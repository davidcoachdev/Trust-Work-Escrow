import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import {
  PublicKey,
  Keypair,
  LAMPORTS_PER_SOL,
  SystemProgram,
} from "@solana/web3.js";
import { assert } from "chai";

const pid = new PublicKey("J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h");

const pda = (seeds: Buffer[]) =>
  PublicKey.findProgramAddressSync(seeds, pid)[0];

const configPda = pda([Buffer.from("config")]);
const jobPda = (client: PublicKey, jobId: BN) =>
  pda([
    Buffer.from("job"),
    client.toBuffer(),
    jobId.toArrayLike(Buffer, "le", 8),
  ]);
const disputePda = (job: PublicKey) => pda([Buffer.from("dispute"), job.toBuffer()]);
const arbFeePda = (job: PublicKey) => pda([Buffer.from("arb_fee"), job.toBuffer()]);
const milestonePda = (job: PublicKey, idx: number) =>
  pda([Buffer.from("milestone"), job.toBuffer(), Buffer.from([idx])]);
const supportPda = (job: PublicKey) => pda([Buffer.from("support"), job.toBuffer()]);

describe("trust-escrow-v3", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Escrow as Program<Escrow>;
  const client = provider.wallet;

  const treasury = Keypair.generate();
  const arbTreasury = Keypair.generate();
  const advisor = Keypair.generate();

  const newJob = (id: number) => new BN(id);

  const airdrop = async (k: Keypair, sol: number) => {
    const sig = await provider.connection.requestAirdrop(k.publicKey, sol * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);
  };

  before(async () => {
    await airdrop(treasury, 1);
    await airdrop(arbTreasury, 1);
    await airdrop(advisor, 1);
    try {
      await program.methods
        .initializeConfig(advisor.publicKey, treasury.publicKey, arbTreasury.publicKey, 250)
        .accounts({ authority: client.publicKey, treasury: treasury.publicKey, config: configPda })
        .rpc();
    } catch (_) {}
  });

  it("flujo completo con postulaciones + milestones + rechazo y reenvio", async () => {
    const jobId = newJob(1);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);

    await program.methods
      .createJob(jobId, "Job A", "desc", amount, deadline)
      .accounts({ client: client.publicKey, job, config: configPda })
      .rpc();
    await program.methods
      .depositFunds(jobId)
      .accounts({ client: client.publicKey, job, config: configPda })
      .rpc();

    const app1 = Keypair.generate();
    await airdrop(app1, 1);
    const app2 = Keypair.generate();
    await airdrop(app2, 1);

    await program.methods
      .applyToJob(jobId, "quiero hacerlo")
      .accounts({ applicant: app1.publicKey, client: client.publicKey, job })
      .signers([app1])
      .rpc();
    await program.methods
      .applyToJob(jobId, "yo tambien")
      .accounts({ applicant: app2.publicKey, client: client.publicKey, job })
      .signers([app2])
      .rpc();

    await program.methods
      .acceptApplication(jobId, new BN(0))
      .accounts({ client: client.publicKey, job })
      .rpc();

    const m0 = 1_000_000;
    await program.methods
      .createMilestone(jobId, new BN(0), "M0", "d", new BN(m0), deadline)
      .accounts({ client: client.publicKey, job })
      .rpc();
    await program.methods
      .submitMilestone(jobId, new BN(0))
      .accounts({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer])
      .rpc();
    await program.methods
      .rejectMilestone(jobId, new BN(0))
      .accounts({ client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .rpc();
    // reenvio tras rechazo (lo que antes bloqueaba el release)
    await program.methods
      .submitMilestone(jobId, new BN(0))
      .accounts({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer])
      .rpc();
    await program.methods
      .approveMilestone(jobId, new BN(0))
      .accounts({ client: client.publicKey, job, freelancer: freelancer.publicKey, milestone: milestonePda(job, 0) })
      .rpc();

    const before = await provider.connection.getBalance(freelancer.publicKey);
    await program.methods
      .submitWork(jobId)
      .accounts({ freelancer: freelancer.publicKey, client: client.publicKey, job })
      .signers([freelancer])
      .rpc();
    await program.methods
      .approveWork(jobId)
      .accounts({ client: client.publicKey, job, freelancer: freelancer.publicKey, treasury: treasury.publicKey, config: configPda })
      .rpc();

    const after = await provider.connection.getBalance(freelancer.publicKey);
    assert.isTrue(after - before >= new BN(m0).toNumber(), "freelancer debe recibir el resto + milestone");
  });

  it("disputa mutua: la fee de arbitraje va a arbitration_treasury", async () => {
    const jobId = newJob(2);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, "Job B", "desc", amount, deadline)
      .accounts({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accounts({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, "x").accounts({ applicant: freelancer.publicKey, client: client.publicKey, job }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, new BN(0)).accounts({ client: client.publicKey, job }).rpc();
    await program.methods.submitWork(jobId).accounts({ freelancer: freelancer.publicKey, client: client.publicKey, job }).signers([freelancer]).rpc();

    await program.methods.raiseDispute(jobId, "no me pagan").accounts({ raiser: client.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job), config: configPda }).rpc();
    await program.methods.acceptDispute(jobId).accounts({ accepter: freelancer.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job), config: configPda }).signers([freelancer]).rpc();
    await program.methods.resolvePlatformCase(jobId, new BN(100)).accounts({ advisor: advisor.publicKey, client: client.publicKey, job, dispute: disputePda(job), config: configPda }).signers([advisor]).rpc();

    const before = await provider.connection.getBalance(arbTreasury.publicKey);
    await program.methods.finalizeDisputePayouts(jobId).accounts({
      resolver: advisor.publicKey,
      client: client.publicKey,
      job,
      dispute: disputePda(job),
      escrow: arbFeePda(job),
      freelancer: freelancer.publicKey,
      treasury: treasury.publicKey,
      arbitrationTreasury: arbTreasury.publicKey,
      config: configPda,
    }).signers([advisor]).rpc();
    const after = await provider.connection.getBalance(arbTreasury.publicKey);
    // 5% de lo disputado (amount, sin milestones) = 100_000 lamports
    assert.isTrue(after - before >= 100_000, "arbitration_treasury debe recibir ~5%");
  });

  it("flujo con ticket: cliente cancela sin bono si el freelancer no entrega", async () => {
    const jobId = newJob(3);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, "Job C", "desc", amount, deadline)
      .accounts({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accounts({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, "x").accounts({ applicant: freelancer.publicKey, client: client.publicKey, job }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, new BN(0)).accounts({ client: client.publicKey, job }).rpc();

    // abre ticket (sin bono) y el asesor resuelve -> cancela y reembolsa
    await program.methods.openSupportTicket(jobId, "el freelancer no entrego")
      .accounts({ opener: client.publicKey, client: client.publicKey, job, dispute: disputePda(job), ticket: supportPda(job) })
      .rpc();
    const before = await provider.connection.getBalance(client.publicKey);
    await program.methods.resolveSupportTicket(jobId, "cancelado por incumplimiento")
      .accounts({ advisor: advisor.publicKey, client: client.publicKey, job, ticket: supportPda(job), opener: client.publicKey, config: configPda })
      .signers([advisor])
      .rpc();
    const after = await provider.connection.getBalance(client.publicKey);
    assert.isTrue(after - before >= new BN(amount.toNumber()).toNumber(), "cliente recupera su deposito");
  });

  it("cancel_job en Funded reembolsa", async () => {
    const jobId = newJob(4);
    const job = jobPda(client.publicKey, jobId);
    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, "Job D", "desc", amount, deadline)
      .accounts({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accounts({ client: client.publicKey, job, config: configPda }).rpc();
    const before = await provider.connection.getBalance(client.publicKey);
    await program.methods.cancelJob(jobId).accounts({ client: client.publicKey, job, config: configPda }).rpc();
    const after = await provider.connection.getBalance(client.publicKey);
    assert.isTrue(after - before >= new BN(amount.toNumber()).toNumber(), "cancel en Funded reembolsa");
  });
});
