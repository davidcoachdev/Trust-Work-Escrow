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
import * as crypto from "crypto";
const hashProposal = (s: string) => Array.from(crypto.createHash("sha256").update(s).digest() as unknown as number[]);

const endpoint = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
const parsedEndpoint = new URL(endpoint);
if (parsedEndpoint.protocol !== "http:" || parsedEndpoint.hostname !== "127.0.0.1") {
  throw new Error(`Tests require a loopback localnet endpoint; refusing ${endpoint}`);
}

const pid = new PublicKey("7a2YhCd7iivXfyySkp1pf5jjijGqpjNqwQCUS912q5Vh");

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
const evidencePda = (dispute: PublicKey, index: number) =>
  pda([Buffer.from("evidence"), dispute.toBuffer(), Buffer.from([index])]);
const arbFeePda = (job: PublicKey) => pda([Buffer.from("arb_fee"), job.toBuffer()]);
const applicationPda = (job: PublicKey, index: number, applicant: PublicKey) =>
  pda([Buffer.from("application"), job.toBuffer(), Buffer.from([index]), applicant.toBuffer()]);
const milestonePda = (job: PublicKey, idx: number) =>
  pda([Buffer.from("milestone"), job.toBuffer(), Buffer.from([idx])]);
const supportPda = (job: PublicKey) => pda([Buffer.from("support"), job.toBuffer()]);
const arbiterPoolPda = () => pda([Buffer.from("arbiter_pool")]);

describe("trust-escrow-v3", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Escrow as Program<Escrow>;
  const client = provider.wallet;

  let treasury: PublicKey;
  let arbTreasury: PublicKey;
  let advisor: Keypair;

  const runId = Math.floor(Date.now() / 1000) * 1_000
    + Number(process.env.TRUST_ESCROW_V3_RUN_NONCE || "0");
  const newJob = (id: number) => new BN(runId * 10 + id);

  const airdrop = async (k: Keypair, sol: number) => {
    let lastError: unknown;
    for (let attempt = 1; attempt <= 3; attempt++) {
      try {
        const sig = await provider.connection.requestAirdrop(k.publicKey, sol * LAMPORTS_PER_SOL);
        await provider.connection.confirmTransaction(sig);
        return;
      } catch (error) {
        lastError = error;
        await new Promise((resolve) => setTimeout(resolve, attempt * 500));
      }
    }
    throw new Error(`Surfpool airdrop failed for ${k.publicKey.toBase58()}: ${String(lastError)}`);
  };

  const advisorFromEnvironment = (): Keypair => {
    const raw = process.env.TRUST_ESCROW_V3_ADVISOR_KEYPAIR;
    if (!raw) {
      throw new Error(
        "Persistent Config exists but its advisor signer is unavailable; set TRUST_ESCROW_V3_ADVISOR_KEYPAIR to the existing advisor secret-key JSON"
      );
    }
    try {
      return Keypair.fromSecretKey(Buffer.from(JSON.parse(raw)));
    } catch (error) {
      throw new Error(`Invalid TRUST_ESCROW_V3_ADVISOR_KEYPAIR: ${String(error)}`);
    }
  };

  before(async () => {
    const existing = await program.account.config.fetchNullable(configPda);
    if (existing) {
      if (!existing.authority.equals(client.publicKey)) {
        throw new Error(
          `Persistent Config authority mismatch: expected ${client.publicKey.toBase58()}, found ${existing.authority.toBase58()}`
        );
      }
      advisor = advisorFromEnvironment();
      if (!advisor.publicKey.equals(existing.advisor)) {
        throw new Error(
          `Persistent Config advisor mismatch: signer ${advisor.publicKey.toBase58()} != Config ${existing.advisor.toBase58()}`
        );
      }
      treasury = existing.treasury;
      arbTreasury = existing.arbitrationTreasury;
    } else {
      const treasurySigner = Keypair.generate();
      const arbTreasurySigner = Keypair.generate();
      advisor = Keypair.generate();
      await airdrop(treasurySigner, 1);
      await airdrop(arbTreasurySigner, 1);
      await airdrop(advisor, 1);
      treasury = treasurySigner.publicKey;
      arbTreasury = arbTreasurySigner.publicKey;
      await program.methods
        .initializeConfig(advisor.publicKey, treasury, arbTreasury, 250)
        .accountsPartial({ authority: client.publicKey, treasury, arbitrationTreasury: arbTreasury, config: configPda })
        .rpc();
    }

    const configured = await program.account.config.fetch(configPda);
    assert.equal(configured.advisor.toBase58(), advisor.publicKey.toBase58(), "tests use Config advisor");
  });

  it("flujo completo con postulaciones + milestones + rechazo y reenvio", async () => {
    const jobId = newJob(1);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);

    await program.methods
      .createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda })
      .rpc();
    await program.methods
      .depositFunds(jobId)
      .accountsPartial({ client: client.publicKey, job, config: configPda })
      .rpc();

    const app1 = Keypair.generate();
    await airdrop(app1, 1);
    const app2 = Keypair.generate();
    await airdrop(app2, 1);

    await program.methods
      .applyToJob(jobId, 0, hashProposal("quiero hacerlo"))
      .accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId })
      .signers([freelancer])
      .rpc();
    await program.methods
      .applyToJob(jobId, 1, hashProposal("yo tambien"))
      .accountsPartial({ applicant: app2.publicKey, client: client.publicKey, job, application: applicationPda(job, 1, app2.publicKey), systemProgram: SystemProgram.programId })
      .signers([app2])
      .rpc();

    await program.methods
      .acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) })
      .rpc();

    const app2BeforeCleanup = await provider.connection.getBalance(app2.publicKey);
    let crossedCleanupRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 1)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([
          { pubkey: applicationPda(job, 1, app2.publicKey), isWritable: true, isSigner: false },
          { pubkey: freelancer.publicKey, isWritable: true, isSigner: false },
        ])
        .rpc();
    } catch (error) {
      crossedCleanupRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(crossedCleanupRejected, "cleanup no acepta applicant de otra cuenta");

    await program.methods.cleanupApplications(jobId, 1)
      .accountsPartial({ client: client.publicKey, job })
      .remainingAccounts([
        { pubkey: applicationPda(job, 1, app2.publicKey), isWritable: true, isSigner: false },
        { pubkey: app2.publicKey, isWritable: true, isSigner: false },
      ])
      .rpc();
    assert.isNull(await provider.connection.getAccountInfo(applicationPda(job, 1, app2.publicKey)));
    assert.isAbove(
      await provider.connection.getBalance(app2.publicKey),
      app2BeforeCleanup,
      "cleanup parcial devuelve la rent al applicant correcto",
    );
    let cleanupReplayRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 1)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([
          { pubkey: applicationPda(job, 1, app2.publicKey), isWritable: true, isSigner: false },
          { pubkey: app2.publicKey, isWritable: true, isSigner: false },
        ])
        .rpc();
    } catch (error) {
      cleanupReplayRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(cleanupReplayRejected, "cleanup repetido no puede reclamar rent dos veces");

    const m0 = 1_000_000;
    await program.methods
      .createMilestone(jobId, 0, new BN(m0))
      .accountsPartial({ client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .rpc();
    await program.methods
      .submitMilestone(jobId, 0)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer])
      .rpc();
    await program.methods
      .rejectMilestone(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .rpc();
    // reenvio tras rechazo (lo que antes bloqueaba el release)
    await new Promise((resolve) => setTimeout(resolve, 1_000));
    await program.methods
      .submitMilestone(jobId, 0)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer])
      .rpc();
    await program.methods
      .approveMilestone(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, freelancer: freelancer.publicKey, milestone: milestonePda(job, 0) })
      .rpc();

    await program.methods
      .submitWork(jobId)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job })
      .signers([freelancer])
      .rpc();
    await program.methods
      .approveWork(jobId)
      .accountsPartial({ client: client.publicKey, job, freelancer: freelancer.publicKey, treasury, config: configPda })
      .remainingAccounts([
        { pubkey: applicationPda(job, 0, freelancer.publicKey), isWritable: true, isSigner: false },
        { pubkey: freelancer.publicKey, isWritable: true, isSigner: false },
        { pubkey: applicationPda(job, 1, app2.publicKey), isWritable: true, isSigner: false },
        { pubkey: app2.publicKey, isWritable: true, isSigner: false },
      ])
      .rpc();

    const app2AfterCleanup = await provider.connection.getBalance(app2.publicKey);
    assert.isAbove(app2AfterCleanup, app2BeforeCleanup, "la rent de una aplicación no aceptada vuelve al applicant");
    assert.isNotNull(
      await provider.connection.getAccountInfo(applicationPda(job, 0, freelancer.publicKey)),
      "la aplicación aceptada se retiene y no se cierra accidentalmente",
    );
    assert.isNull(
      await provider.connection.getAccountInfo(applicationPda(job, 1, app2.publicKey)),
      "la aplicación pendiente se cierra durante el cierre del Job",
    );
  });

  it("disputa mutua: la fee de arbitraje va a arbitration_treasury", async () => {
    const jobId = newJob(2);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);
    const arbiter = Keypair.generate();
    await airdrop(arbiter, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("x")).accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0).accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job }).signers([freelancer]).rpc();

    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    const bond = amount.muln(250).divn(10_000);
    const freelancerBeforeBond = await provider.connection.getBalance(freelancer.publicKey);
      await program.methods.acceptDispute(jobId).accountsPartial({ accepter: freelancer.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([freelancer]).rpc();
    const existingPool = await program.account.arbiterPool.fetchNullable(arbiterPoolPda());
    if (!existingPool) {
      await program.methods.createArbiterPool().accountsPartial({
        authority: client.publicKey,
        pool: arbiterPoolPda(),
        config: configPda,
        systemProgram: SystemProgram.programId,
      }).rpc();
    }
    await program.methods.addArbiter(arbiter.publicKey).accountsPartial({
      authority: client.publicKey,
      pool: arbiterPoolPda(),
      config: configPda,
    }).rpc();
    const freelancerAfterBond = await provider.connection.getBalance(freelancer.publicKey);
    const escrowAfterAcceptance = await program.account.arbitrationEscrow.fetch(arbFeePda(job));
    assert.equal(
      freelancerBeforeBond - freelancerAfterBond,
      bond.toNumber(),
      "acceptDispute debe cobrar el bond desde el wallet del aceptante",
    );
    assert.equal(
      escrowAfterAcceptance.freelancerBond.toString(),
      bond.toString(),
      "acceptDispute debe registrar el bond del freelancer",
    );
    const dispute = disputePda(job);
    const disputeInfo = await provider.connection.getAccountInfo(dispute);
    assert.isNotNull(disputeInfo, "Dispute PDA debe existir");
    assert.isBelow(disputeInfo!.data.length, 10_240, "Dispute debe conservarse bajo 10.240 bytes");
    const disputeState = await program.account.dispute.fetch(dispute);
    assert.equal(disputeState.evidenceCount, 0, "Dispute inicia con contador de evidencia en cero");

    // Off-chain content ahora es hash 32 bytes — el check de 2.048 bytes es off-chain.
    // Antes se rechazaba Buffer.alloc(2049) on-chain; ahora siempre son 32 bytes, no hay rechazo.
    let sizeRejected = false;
    const largeHash = hashProposal("x".repeat(2049));
    assert.equal(largeHash.length, 32, "hash siempre 32 bytes");
    assert.isFalse(sizeRejected, "hash 32 bytes siempre pasa on-chain (size check es off-chain)");

    for (let index = 0; index < 10; index++) {
      await program.methods
        .submitEvidence(jobId, index, hashProposal(`evidence-${index}`))
        .accountsPartial({
          submitter: index % 2 === 0 ? client.publicKey : freelancer.publicKey,
          client: client.publicKey,
          job,
          dispute,
          evidence: evidencePda(dispute, index),
          systemProgram: SystemProgram.programId,
        })
        .signers(index % 2 === 0 ? [] : [freelancer])
        .rpc();
    }
    const firstEvidence = await program.account.evidence.fetch(evidencePda(dispute, 0));
    assert.equal(firstEvidence.dispute.toBase58(), dispute.toBase58());
    assert.equal(firstEvidence.index, 0);
    assert.equal(firstEvidence.author.toBase58(), client.publicKey.toBase58());
    assert.deepEqual(firstEvidence.contentHash, hashProposal("evidence-0"));

    let limitRejected = false;
    try {
      await program.methods
        .submitEvidence(jobId, 10, hashProposal("evidence-10"))
        .accountsPartial({
          submitter: client.publicKey,
          client: client.publicKey,
          job,
          dispute,
          evidence: evidencePda(dispute, 10),
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      limitRejected = true;
      assert.include(String(error), "EvidenceLimitReached");
    }
    assert.isTrue(limitRejected, "la undécima evidencia debe rechazarse");

    const afterEvidence = await program.account.dispute.fetch(dispute);
    assert.equal(afterEvidence.evidenceCount, 10, "contador de evidencia debe ser exacto");
    await program.methods.assignArbiter(jobId).accountsPartial({
      authority: client.publicKey,
      client: client.publicKey,
      job,
      dispute,
      pool: arbiterPoolPda(),
      arbiter: arbiter.publicKey,
      config: configPda,
    }).rpc();
    await program.methods.resolveDispute(jobId, 100).accountsPartial({
      arbiter: arbiter.publicKey,
      client: client.publicKey,
      job,
      dispute,
    }).signers([arbiter]).rpc();

    const evidenceRent = (await Promise.all(
      Array.from({ length: 10 }, (_, index) => provider.connection.getAccountInfo(evidencePda(dispute, index))),
    )).reduce((total, info) => total + (info?.lamports ?? 0), 0);
    const clientBeforeEvidenceCleanup = await provider.connection.getBalance(client.publicKey);
    await program.methods.cleanupDisputeEvidence(jobId).accountsPartial({
      resolver: arbiter.publicKey,
      client: client.publicKey,
      job,
      dispute,
      config: configPda,
    }).remainingAccounts(
      Array.from({ length: 5 }, (_, index) => ({
        pubkey: evidencePda(dispute, index),
        isWritable: true,
        isSigner: false,
      })),
    ).signers([arbiter]).rpc();

    const before = await provider.connection.getBalance(arbTreasury);
    await program.methods.finalizeDisputePayouts(jobId).accountsPartial({
      resolver: arbiter.publicKey,
      client: client.publicKey,
      job,
      dispute,
      escrow: arbFeePda(job),
      freelancer: freelancer.publicKey,
      treasury,
      arbitrationTreasury: arbTreasury,
      config: configPda,
    })
      .remainingAccounts([
        ...Array.from({ length: 5 }, (_, offset) => ({
          pubkey: evidencePda(dispute, offset + 5),
          isWritable: true,
          isSigner: false,
        })),
        { pubkey: applicationPda(job, 0, freelancer.publicKey), isWritable: true, isSigner: false },
        { pubkey: freelancer.publicKey, isWritable: true, isSigner: false },
      ])
      .signers([arbiter])
      .rpc();
    const after = await provider.connection.getBalance(arbTreasury);
    // 5% de lo disputado (amount, sin milestones) = 100_000 lamports
    assert.isTrue(after - before >= 100_000, "arbitration_treasury debe recibir ~5%");
    const clientAfterEvidenceCleanup = await provider.connection.getBalance(client.publicKey);
    assert.isAtLeast(
      clientAfterEvidenceCleanup - clientBeforeEvidenceCleanup,
      evidenceRent,
      "el cleanup de Evidence devuelve la rent al cliente sin convertirla en payout",
    );
    for (let index = 0; index < 10; index++) {
      assert.isNull(
        await provider.connection.getAccountInfo(evidencePda(dispute, index)),
        `Evidence PDA ${index} debe cerrarse al finalizar`,
      );
    }
  });

  it("flujo con ticket: cliente cancela sin bono si el freelancer no entrega", async () => {
    const jobId = newJob(3);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);

    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("x")).accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0).accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();

    const milestoneAmount = new BN(1_000_000);
    await program.methods.createMilestone(jobId, 0, milestoneAmount)
      .accountsPartial({ client: client.publicKey, job, milestone: milestonePda(job, 0) }).rpc();
    await program.methods.submitMilestone(jobId, 0)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer]).rpc();
    await program.methods.approveMilestone(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, freelancer: freelancer.publicKey, milestone: milestonePda(job, 0) }).rpc();

    // abre ticket (sin bono) y el asesor resuelve -> cancela y reembolsa
    await program.methods.openSupportTicket(jobId)
      .accountsPartial({ opener: client.publicKey, client: client.publicKey, job, dispute: null, ticket: supportPda(job) })
      .rpc();
    const before = await provider.connection.getBalance(client.publicKey);
    await program.methods.resolveSupportTicket(jobId)
      .accountsPartial({ advisor: advisor.publicKey, client: client.publicKey, job, ticket: supportPda(job), opener: client.publicKey, config: configPda })
      .remainingAccounts([
        { pubkey: applicationPda(job, 0, freelancer.publicKey), isWritable: true, isSigner: false },
        { pubkey: freelancer.publicKey, isWritable: true, isSigner: false },
      ])
      .signers([advisor])
      .rpc();
    const after = await provider.connection.getBalance(client.publicKey);
    const fee = amount.mul(new BN(250)).div(new BN(10_000));
    const remainingPrincipal = amount.sub(milestoneAmount);
    assert.isTrue(
      after - before >= remainingPrincipal.add(fee).toNumber() - 20_000,
      "cliente recupera principal restante y fee no devengada"
    );
  });

  it("cancel_job en Funded reembolsa", async () => {
    const jobId = newJob(4);
    const job = jobPda(client.publicKey, jobId);
    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    const before = await provider.connection.getBalance(client.publicKey);
    await program.methods.cancelJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
    const after = await provider.connection.getBalance(client.publicKey);
    assert.isTrue(after - before >= new BN(amount.toNumber()).toNumber(), "cancel en Funded reembolsa");
  });

  it("rechaza auto-aprobación antes del deadline desde submitted_at", async () => {
    const jobId = newJob(5);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);
    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);

    await program.methods.createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("x"))
      .accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId })
      .signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();
    await program.methods.submitWork(jobId)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job })
      .signers([freelancer]).rpc();

    const submitted = await program.account.job.fetch(job);
    assert.equal(submitted.status.submitted !== undefined, true);
    assert.isNotNull(submitted.submittedAt);

    let beforeDeadlineFailed = false;
    try {
      await program.methods.autoApproveWork(jobId)
        .accountsPartial({ keeper: client.publicKey, client: client.publicKey, job, freelancer: freelancer.publicKey, treasury, config: configPda, dispute: null })
        .remainingAccounts([
          { pubkey: applicationPda(job, 0, freelancer.publicKey), isWritable: true, isSigner: false },
          { pubkey: freelancer.publicKey, isWritable: true, isSigner: false },
        ])
        .rpc();
    } catch (error) {
      beforeDeadlineFailed = true;
      assert.include(String(error), "AutoApprovalNotReady");
    }
    assert.isTrue(beforeDeadlineFailed);

  });

  it("pause_job rechaza un Job con freelancer asignado", async () => {
    const jobId = newJob(6);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, new BN(2_000_000), deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("x"))
      .accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId })
      .signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();

    try {
      await program.methods.pauseJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
      assert.fail("pause_job debe rechazar freelancer asignado");
    } catch (error) {
      assert.include(String(error), "CannotPauseWithFreelancer");
    }
  });

  it("rota ambas treasuries solo a cuentas System separadas y rechaza destinos inválidos", async () => {
    const nextTreasury = Keypair.generate();
    const nextArbTreasury = Keypair.generate();
    const unauthorized = Keypair.generate();
    await airdrop(nextTreasury, 1);
    await airdrop(nextArbTreasury, 1);
    await airdrop(unauthorized, 1);

    let unauthorizedRejected = false;
    try {
      await program.methods
        .updateTreasury(nextTreasury.publicKey)
        .accountsPartial({ authority: unauthorized.publicKey, config: configPda, newTreasury: nextTreasury.publicKey })
        .signers([unauthorized])
        .rpc();
    } catch (error) {
      unauthorizedRejected = true;
      assert.include(String(error), "NotAuthorized");
    }
    assert.isTrue(unauthorizedRejected, "solo Config.authority puede rotar treasury");

    let treasuryArgumentMismatchRejected = false;
    try {
      await program.methods
        .updateTreasury(nextTreasury.publicKey)
        .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: nextArbTreasury.publicKey })
        .rpc();
    } catch (error) {
      treasuryArgumentMismatchRejected = true;
      assert.include(String(error), "InvalidTreasury");
    }
    assert.isTrue(treasuryArgumentMismatchRejected, "la cuenta treasury debe coincidir con el argumento Pubkey");

    await program.methods
      .updateTreasury(nextTreasury.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: nextTreasury.publicKey })
      .rpc();
    await program.methods
      .updateArbitrationTreasury(nextArbTreasury.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newArbitrationTreasury: nextArbTreasury.publicKey })
      .rpc();

    let config = await program.account.config.fetch(configPda);
    assert.equal(config.treasury.toBase58(), nextTreasury.publicKey.toBase58());
    assert.equal(config.arbitrationTreasury.toBase58(), nextArbTreasury.publicKey.toBase58());
    assert.notEqual(config.treasury.toBase58(), config.arbitrationTreasury.toBase58());

    let sameDestinationRejected = false;
    try {
      await program.methods
        .updateTreasury(nextArbTreasury.publicKey)
        .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: nextArbTreasury.publicKey })
        .rpc();
    } catch (error) {
      sameDestinationRejected = true;
      assert.include(String(error), "InvalidTreasury");
    }
    assert.isTrue(sameDestinationRejected, "las treasuries deben permanecer separadas");

    let defaultRejected = false;
    try {
      await program.methods
        .updateArbitrationTreasury(PublicKey.default)
        .accountsPartial({ authority: client.publicKey, config: configPda, newArbitrationTreasury: PublicKey.default })
        .rpc();
    } catch (error) {
      defaultRejected = true;
      assert.include(String(error), "InvalidTreasury");
    }
    assert.isTrue(defaultRejected, "la treasury default debe rechazarse");

    let programOwnedRejected = false;
    try {
      await program.methods
        .updateTreasury(configPda)
        .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: configPda })
        .rpc();
    } catch (error) {
      programOwnedRejected = true;
      assert.include(String(error), "InvalidTreasury");
    }
    assert.isTrue(programOwnedRejected, "un destino no System-owned debe rechazarse");

    config = await program.account.config.fetch(configPda);
    assert.equal(config.treasury.toBase58(), nextTreasury.publicKey.toBase58());
    assert.equal(config.arbitrationTreasury.toBase58(), nextArbTreasury.publicKey.toBase58());
  });

  it("mantiene 50 postulaciones PDA, rechaza la 51, duplicados y texto inválido", async () => {
    const jobId = newJob(7);
    const job = jobPda(client.publicKey, jobId);
    const amount = new BN(2_000_000);
    const deadline = new BN(Math.floor(Date.now() / 1000) + 3600);
    await program.methods.createJob(jobId, amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();

    const applicationCount = Number(process.env.TRUST_ESCROW_V3_APPLICATION_COUNT || "50");
    const applicants = Array.from({ length: applicationCount }, () => Keypair.generate());
    for (let offset = 0; offset < applicants.length; offset += 10) {
      await Promise.all(applicants.slice(offset, offset + 10).map((applicant) => airdrop(applicant, 0.1)));
    }
    await program.methods.applyToJob(jobId, 0, hashProposal("proposal-0"))
      .accountsPartial({
        applicant: applicants[0].publicKey,
        client: client.publicKey,
        job,
        application: applicationPda(job, 0, applicants[0].publicKey),
        systemProgram: SystemProgram.programId,
      })
      .signers([applicants[0]])
      .rpc();

    let duplicateRejected = false;
    try {
      await program.methods.applyToJob(jobId, 1, hashProposal("duplicate-before-limit"))
        .accountsPartial({
          applicant: applicants[0].publicKey,
          client: client.publicKey,
          job,
          application: applicationPda(job, 1, applicants[0].publicKey),
          systemProgram: SystemProgram.programId,
        })
        .signers([applicants[0]])
        .rpc();
    } catch (error) {
      duplicateRejected = true;
      assert.include(String(error), "AlreadyApplied");
    }
    assert.isTrue(duplicateRejected, "AlreadyApplied debe evaluarse antes del límite");

    for (const [offset, applicant] of applicants.slice(1).entries()) {
      const index = offset + 1;
      await program.methods.applyToJob(jobId, index, hashProposal(`proposal-${index}`))
        .accountsPartial({
          applicant: applicant.publicKey,
          client: client.publicKey,
          job,
          application: applicationPda(job, index, applicant.publicKey),
          systemProgram: SystemProgram.programId,
        })
        .signers([applicant])
      .rpc();
    }

    const fundedJob = await program.account.job.fetch(job);
    assert.lengthOf(fundedJob.applicants, applicationCount);
    let overLimitRejected = false;
    const extra = Keypair.generate();
    await airdrop(extra, 0.1);
    try {
      await program.methods.applyToJob(jobId, applicationCount, hashProposal(`proposal-${applicationCount}`))
        .accountsPartial({
          applicant: extra.publicKey,
          client: client.publicKey,
          job,
          application: applicationPda(job, applicationCount, extra.publicKey),
          systemProgram: SystemProgram.programId,
        })
        .signers([extra])
        .rpc();
    } catch (error) {
      overLimitRejected = true;
      assert.include(String(error), "InvalidApplicationIndex");
    }
    assert.isTrue(overLimitRejected);

  });

  // ──────────────────────────────────────────────────────────────
  // V3-TEST-015: 15 ITs adicionales para 40 ix — remaining_accounts malformado,
  // evidence_cleanup_cursor overflow, MAX_PAUSE_DURATION 30d, withdraw_treasury,
  // resolve_dispute, cleanup etc. (20% → >60% cobertura)
  // ──────────────────────────────────────────────────────────────

  it("V3-TEST-015-01 remaining_accounts vacío debe fallar (InvalidApplicationCleanupAccounts)", async () => {
    const jobId = newJob(10);
    const job = jobPda(client.publicKey, jobId);
    const freelancer = Keypair.generate();
    await airdrop(freelancer, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ra-empty-0"))
      .accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId })
      .signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();
    let emptyRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 0)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([])
        .rpc();
    } catch (error) {
      emptyRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(emptyRejected, "remaining vacío debe fallar");
  });

  it("V3-TEST-015-02 remaining_accounts impar (no múltiplo de 2) debe fallar", async () => {
    const jobId = newJob(11);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ra-odd-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    let oddRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 0)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([
          { pubkey: applicationPda(job, 0, f.publicKey), isWritable: true, isSigner: false },
          { pubkey: f.publicKey, isWritable: true, isSigner: false },
          { pubkey: f.publicKey, isWritable: true, isSigner: false }, // tercer meta impar
        ])
        .rpc();
    } catch (error) {
      oddRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(oddRejected, "impar debe fallar");
  });

  it("V3-TEST-015-03 remaining_accounts is_writable false debe fallar", async () => {
    const jobId = newJob(12);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    const other = Keypair.generate();
    await airdrop(f, 1);
    await airdrop(other, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ra-writable-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.applyToJob(jobId, 1, hashProposal("ra-writable-1"))
      .accountsPartial({ applicant: other.publicKey, client: client.publicKey, job, application: applicationPda(job, 1, other.publicKey), systemProgram: SystemProgram.programId })
      .signers([other]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    let roRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 1)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([
          { pubkey: applicationPda(job, 1, other.publicKey), isWritable: false, isSigner: false },
          { pubkey: other.publicKey, isWritable: true, isSigner: false },
        ])
        .rpc();
    } catch (error) {
      roRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(roRejected, "is_writable false debe fallar");
  });

  it("V3-TEST-015-04 remaining_accounts pubkey mismatch debe fallar", async () => {
    const jobId = newJob(13);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    const other = Keypair.generate();
    const impostor = Keypair.generate();
    await airdrop(f, 1);
    await airdrop(other, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ra-pubkey-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.applyToJob(jobId, 1, hashProposal("ra-pubkey-1"))
      .accountsPartial({ applicant: other.publicKey, client: client.publicKey, job, application: applicationPda(job, 1, other.publicKey), systemProgram: SystemProgram.programId })
      .signers([other]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    let mismatchRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 1)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts([
          { pubkey: applicationPda(job, 1, impostor.publicKey), isWritable: true, isSigner: false },
          { pubkey: other.publicKey, isWritable: true, isSigner: false },
        ])
        .rpc();
    } catch (error) {
      mismatchRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(mismatchRejected, "pubkey mismatch debe fallar");
  });

  it("V3-TEST-015-05 remaining_accounts excede MAX_CLEANUP_BATCH (22 metas) debe fallar", async () => {
    const jobId = newJob(14);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    const apps = Array.from({ length: 11 }, () => Keypair.generate());
    for (const a of apps) await airdrop(a, 0.2);
    for (let i = 0; i < 11; i++) {
      await program.methods.applyToJob(jobId, i, hashProposal(`ra-batch-${i}`))
        .accountsPartial({ applicant: apps[i].publicKey, client: client.publicKey, job, application: applicationPda(job, i, apps[i].publicKey), systemProgram: SystemProgram.programId })
        .signers([apps[i]]).rpc();
    }
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: apps[0].publicKey, application: applicationPda(job, 0, apps[0].publicKey) }).rpc();
    // 22 metas = 11 apps -> excede MAX_CLEANUP_BATCH 10, debe fallar antes de validar pubkeys
    const fakeMetas22 = Array.from({ length: 11 }, () => {
      const fakeApp = Keypair.generate().publicKey;
      const fakeApplicant = Keypair.generate().publicKey;
      return [
        { pubkey: fakeApp, isWritable: true, isSigner: false },
        { pubkey: fakeApplicant, isWritable: true, isSigner: false },
      ];
    }).flat();
    let batchRejected = false;
    try {
      await program.methods.cleanupApplications(jobId, 1)
        .accountsPartial({ client: client.publicKey, job })
        .remainingAccounts(fakeMetas22)
        .rpc();
    } catch (error) {
      batchRejected = true;
      assert.include(String(error), "InvalidApplicationCleanupAccounts");
    }
    assert.isTrue(batchRejected, "22 metas (>10 apps) debe fallar");
  });

  it("V3-TEST-015-06 evidence_cleanup_cursor overflow: cleanup más que remaining debe fallar", async () => {
    const jobId = newJob(15);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ev-overflow-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    await program.methods.acceptDispute(jobId).accountsPartial({ accepter: f.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([f]).rpc();
    const dispute = disputePda(job);
    for (let i = 0; i < 2; i++) {
      await program.methods.submitEvidence(jobId, i, hashProposal(`ev-overflow-${i}`))
        .accountsPartial({ submitter: i % 2 === 0 ? client.publicKey : f.publicKey, client: client.publicKey, job, dispute, evidence: evidencePda(dispute, i), systemProgram: SystemProgram.programId })
        .signers(i % 2 === 0 ? [] : [f]).rpc();
    }
    const existingPool = await program.account.arbiterPool.fetchNullable(arbiterPoolPda());
    if (!existingPool) {
      await program.methods.createArbiterPool().accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda, systemProgram: SystemProgram.programId }).rpc();
    }
    const arb = Keypair.generate();
    await airdrop(arb, 1);
    try { await program.methods.addArbiter(arb.publicKey).accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda }).rpc(); } catch {}
    await program.methods.assignArbiter(jobId).accountsPartial({ authority: client.publicKey, client: client.publicKey, job, dispute, pool: arbiterPoolPda(), arbiter: arb.publicKey, config: configPda }).rpc();
    await program.methods.resolveDispute(jobId, 50).accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute }).signers([arb]).rpc();
    // evidence_count=2, intentar cleanup con 3 evidencias debe fallar
    let overflowRejected = false;
    try {
      await program.methods.cleanupDisputeEvidence(jobId).accountsPartial({ resolver: arb.publicKey, client: client.publicKey, job, dispute, config: configPda })
        .remainingAccounts([
          { pubkey: evidencePda(dispute, 0), isWritable: true, isSigner: false },
          { pubkey: evidencePda(dispute, 1), isWritable: true, isSigner: false },
          { pubkey: evidencePda(dispute, 2), isWritable: true, isSigner: false },
        ])
        .signers([arb]).rpc();
    } catch (error) {
      overflowRejected = true;
      assert.include(String(error), "InvalidEvidenceCleanupAccounts");
    }
    assert.isTrue(overflowRejected, "cleanup con más evidencias que remaining debe fallar");
  });

  it("V3-TEST-015-07 evidence_cleanup paginación 11 evidencias debe fallar MAX_EVIDENCE_CLEANUP_BATCH", async () => {
    const jobId = newJob(16);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("ev-batch-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    await program.methods.acceptDispute(jobId).accountsPartial({ accepter: f.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([f]).rpc();
    const dispute = disputePda(job);
    for (let i = 0; i < 10; i++) {
      await program.methods.submitEvidence(jobId, i, hashProposal(`ev-batch10-${i}`))
        .accountsPartial({ submitter: i % 2 === 0 ? client.publicKey : f.publicKey, client: client.publicKey, job, dispute, evidence: evidencePda(dispute, i), systemProgram: SystemProgram.programId })
        .signers(i % 2 === 0 ? [] : [f]).rpc();
    }
    const arb = Keypair.generate();
    await airdrop(arb, 1);
    try { await program.methods.addArbiter(arb.publicKey).accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda }).rpc(); } catch {}
    await program.methods.assignArbiter(jobId).accountsPartial({ authority: client.publicKey, client: client.publicKey, job, dispute, pool: arbiterPoolPda(), arbiter: arb.publicKey, config: configPda }).rpc();
    await program.methods.resolveDispute(jobId, 50).accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute }).signers([arb]).rpc();
    let batchRejected = false;
    try {
      await program.methods.cleanupDisputeEvidence(jobId).accountsPartial({ resolver: arb.publicKey, client: client.publicKey, job, dispute, config: configPda })
        .remainingAccounts(Array.from({ length: 11 }, (_, i) => ({ pubkey: evidencePda(dispute, i), isWritable: true, isSigner: false })))
        .signers([arb]).rpc();
    } catch (error) {
      batchRejected = true;
      assert.include(String(error), "InvalidEvidenceCleanupAccounts");
    }
    assert.isTrue(batchRejected, "11 evidencias en un tx debe fallar (MAX 10)");
  });

  it("V3-TEST-015-08 MAX_PAUSE_DURATION 30d: pause y expire inmediato debe fallar JobPaused (no expirado)", async () => {
    const jobId = newJob(17);
    const job = jobPda(client.publicKey, jobId);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.pauseJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
    let fetched = await program.account.job.fetch(job);
    assert.isTrue(fetched.paused, "job debe estar pausado");
    assert.equal(fetched.pausedAt.toNumber() > 0, true);
    // MAX_PAUSE_DURATION = 30*24*3600 = 2592000
    assert.equal(30 * 24 * 60 * 60, 2592000);
    let notExpiredRejected = false;
    try {
      await program.methods.expirePausedJob(jobId).accountsPartial({ caller: client.publicKey, client: client.publicKey, job }).rpc();
    } catch (error) {
      notExpiredRejected = true;
      assert.include(String(error), "JobPaused");
    }
    assert.isTrue(notExpiredRejected, "expire inmediato debe fallar JobPaused (no han pasado 30d)");
    await program.methods.unpauseJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
    fetched = await program.account.job.fetch(job);
    assert.isFalse(fetched.paused, "unpause debe limpiar paused");
    assert.equal(fetched.pausedAt.toNumber(), 0);
  });

  it("V3-TEST-015-09 pause_job solo Created/Funded sin freelancer; rechaza con freelancer y cancel_job paginado", async () => {
    const jobId = newJob(18);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("pause-no-freelancer-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    // pause en Funded sin freelancer debe pasar (ya testeado 08), pero tras accept debe fallar
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    let pauseWithFreelancerRejected = false;
    try {
      await program.methods.pauseJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
    } catch (error) {
      pauseWithFreelancerRejected = true;
      assert.include(String(error), "CannotPauseWithFreelancer");
    }
    assert.isTrue(pauseWithFreelancerRejected, "pause con freelancer debe fallar");
    // cancel_job con cleanup paginado tras InProgress
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    // cancel no permitido en Submitted, debe fallar InvalidJobStatus
    let cancelRejected = false;
    try {
      await program.methods.cancelJob(jobId).accountsPartial({ client: client.publicKey, job }).rpc();
    } catch (error) {
      cancelRejected = true;
      assert.include(String(error), "InvalidJobStatus");
    }
    assert.isTrue(cancelRejected, "cancel en Submitted debe fallar");
  });

  it("V3-TEST-015-10 withdraw_treasury happy path y validación", async () => {
    const newTreasury = Keypair.generate();
    await airdrop(newTreasury, 2);
    await program.methods.updateTreasury(newTreasury.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: newTreasury.publicKey }).rpc();
    const dest = Keypair.generate().publicKey;
    const treasuryBefore = await provider.connection.getBalance(newTreasury.publicKey);
    assert.isAtLeast(treasuryBefore, 1_000_000);
    const withdrawAmount = new BN(500_000);
    await program.methods.withdrawTreasury(withdrawAmount)
      .accountsPartial({ treasury: newTreasury.publicKey, destination: dest, config: configPda })
      .signers([newTreasury]).rpc();
    const treasuryAfter = await provider.connection.getBalance(newTreasury.publicKey);
    assert.equal(treasuryBefore - treasuryAfter, withdrawAmount.toNumber());
    const destInfo = await provider.connection.getAccountInfo(dest);
    assert.isNotNull(destInfo);
    assert.equal(destInfo!.lamports, withdrawAmount.toNumber());
  });

  it("V3-TEST-015-11 withdraw_treasury rechaza 0, insufficient funds y not authorized", async () => {
    // treasury controlada para este test
    const ctrlTreasury = Keypair.generate();
    await airdrop(ctrlTreasury, 2);
    await program.methods.updateTreasury(ctrlTreasury.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: ctrlTreasury.publicKey }).rpc();
    // amount 0 debe fallar AmountTooSmall con signer correcto
    let zeroRejected = false;
    try {
      await program.methods.withdrawTreasury(new BN(0))
        .accountsPartial({ treasury: ctrlTreasury.publicKey, destination: Keypair.generate().publicKey, config: configPda })
        .signers([ctrlTreasury]).rpc();
    } catch (error) {
      zeroRejected = true;
      assert.include(String(error), "AmountTooSmall");
    }
    assert.isTrue(zeroRejected, "0 debe fallar AmountTooSmall");
    // NotAuthorized: impostor pubkey distinta a config.treasury
    const impostor = Keypair.generate();
    await airdrop(impostor, 1);
    let notAuthRejected = false;
    try {
      await program.methods.withdrawTreasury(new BN(1000))
        .accountsPartial({ treasury: impostor.publicKey, destination: Keypair.generate().publicKey, config: configPda })
        .signers([impostor]).rpc();
    } catch (error) {
      notAuthRejected = true;
      assert.include(String(error), "NotAuthorized");
    }
    assert.isTrue(notAuthRejected, "impostor treasury debe fallar NotAuthorized");
    const ctrlBal = await provider.connection.getBalance(ctrlTreasury.publicKey);
    let insufficientRejected = false;
    try {
      await program.methods.withdrawTreasury(new BN(ctrlBal + 1_000_000))
        .accountsPartial({ treasury: ctrlTreasury.publicKey, destination: Keypair.generate().publicKey, config: configPda })
        .signers([ctrlTreasury]).rpc();
    } catch (error) {
      insufficientRejected = true;
      assert.include(String(error), "InsufficientFunds");
    }
    assert.isTrue(insufficientRejected, "exceso debe fallar InsufficientFunds");
    const restore = Keypair.generate();
    await airdrop(restore, 1);
    await program.methods.updateTreasury(restore.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newTreasury: restore.publicKey }).rpc();
    treasury = restore.publicKey;
  });

  it("V3-TEST-015-12 withdraw_arbitration happy path", async () => {
    const newArb = Keypair.generate();
    await airdrop(newArb, 2);
    await program.methods.updateArbitrationTreasury(newArb.publicKey)
      .accountsPartial({ authority: client.publicKey, config: configPda, newArbitrationTreasury: newArb.publicKey }).rpc();
    const dest = Keypair.generate().publicKey;
    const before = await provider.connection.getBalance(newArb.publicKey);
    const amt = new BN(300_000);
    await program.methods.withdrawArbitration(amt)
      .accountsPartial({ arbitrationTreasury: newArb.publicKey, destination: dest, config: configPda })
      .signers([newArb]).rpc();
    const after = await provider.connection.getBalance(newArb.publicKey);
    assert.equal(before - after, amt.toNumber());
    arbTreasury = newArb.publicKey;
  });

  it("V3-TEST-015-13 resolve_dispute NotArbiter debe fallar", async () => {
    const jobId = newJob(19);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("resolve-notarb-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    await program.methods.acceptDispute(jobId).accountsPartial({ accepter: f.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([f]).rpc();
    const dispute = disputePda(job);
    const arb = Keypair.generate();
    await airdrop(arb, 1);
    try { await program.methods.addArbiter(arb.publicKey).accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda }).rpc(); } catch {}
    await program.methods.assignArbiter(jobId).accountsPartial({ authority: client.publicKey, client: client.publicKey, job, dispute, pool: arbiterPoolPda(), arbiter: arb.publicKey, config: configPda }).rpc();
    const impostor = Keypair.generate();
    await airdrop(impostor, 1);
    let notArbRejected = false;
    try {
      await program.methods.resolveDispute(jobId, 50)
        .accountsPartial({ arbiter: impostor.publicKey, client: client.publicKey, job, dispute })
        .signers([impostor]).rpc();
    } catch (error) {
      notArbRejected = true;
      assert.include(String(error), "NotArbiter");
    }
    assert.isTrue(notArbRejected, "impostor no debe poder resolver");
  });

  it("V3-TEST-015-14 resolve_dispute InvalidPercent y DisputeAlreadyResolved", async () => {
    const jobId = newJob(20);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("resolve-pct-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    await program.methods.acceptDispute(jobId).accountsPartial({ accepter: f.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([f]).rpc();
    const dispute = disputePda(job);
    const arb = Keypair.generate();
    await airdrop(arb, 1);
    try { await program.methods.addArbiter(arb.publicKey).accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda }).rpc(); } catch {}
    await program.methods.assignArbiter(jobId).accountsPartial({ authority: client.publicKey, client: client.publicKey, job, dispute, pool: arbiterPoolPda(), arbiter: arb.publicKey, config: configPda }).rpc();
    let pctRejected = false;
    try {
      await program.methods.resolveDispute(jobId, 101)
        .accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute })
        .signers([arb]).rpc();
    } catch (error) {
      pctRejected = true;
      assert.include(String(error), "InvalidPercent");
    }
    assert.isTrue(pctRejected, "101% debe fallar InvalidPercent");
    await program.methods.resolveDispute(jobId, 60)
      .accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute })
      .signers([arb]).rpc();
    let alreadyRejected = false;
    try {
      await program.methods.resolveDispute(jobId, 30)
        .accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute })
        .signers([arb]).rpc();
    } catch (error) {
      alreadyRejected = true;
      assert.include(String(error), "DisputeAlreadyResolved");
    }
    assert.isTrue(alreadyRejected, "segunda resolución debe fallar DisputeAlreadyResolved");
  });

  it("V3-TEST-015-15 cleanup paginado + finalize conserva payout y cierra evidencias", async () => {
    const jobId = newJob(21);
    const job = jobPda(client.publicKey, jobId);
    const f = Keypair.generate();
    await airdrop(f, 1);
    await program.methods.createJob(jobId, new BN(2_000_000), new BN(Math.floor(Date.now() / 1000) + 3600))
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, hashProposal("cleanup-final-0"))
      .accountsPartial({ applicant: f.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, f.publicKey), systemProgram: SystemProgram.programId })
      .signers([f]).rpc();
    await program.methods.acceptApplication(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, applicant: f.publicKey, application: applicationPda(job, 0, f.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: f.publicKey, client: client.publicKey, job }).signers([f]).rpc();
    await program.methods.raiseDispute(jobId).accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
    await program.methods.acceptDispute(jobId).accountsPartial({ accepter: f.publicKey, client: client.publicKey, job, dispute: disputePda(job), escrow: arbFeePda(job) }).signers([f]).rpc();
    const dispute = disputePda(job);
    for (let i = 0; i < 5; i++) {
      await program.methods.submitEvidence(jobId, i, hashProposal(`cleanup-final-ev-${i}`))
        .accountsPartial({ submitter: i % 2 === 0 ? client.publicKey : f.publicKey, client: client.publicKey, job, dispute, evidence: evidencePda(dispute, i), systemProgram: SystemProgram.programId })
        .signers(i % 2 === 0 ? [] : [f]).rpc();
    }
    const arb = Keypair.generate();
    await airdrop(arb, 1);
    try { await program.methods.addArbiter(arb.publicKey).accountsPartial({ authority: client.publicKey, pool: arbiterPoolPda(), config: configPda }).rpc(); } catch {}
    await program.methods.assignArbiter(jobId).accountsPartial({ authority: client.publicKey, client: client.publicKey, job, dispute, pool: arbiterPoolPda(), arbiter: arb.publicKey, config: configPda }).rpc();
    await program.methods.resolveDispute(jobId, 40).accountsPartial({ arbiter: arb.publicKey, client: client.publicKey, job, dispute }).signers([arb]).rpc();
    // cleanup parcial 2 evidencias, verifica cursor
    await program.methods.cleanupDisputeEvidence(jobId).accountsPartial({ resolver: arb.publicKey, client: client.publicKey, job, dispute, config: configPda })
      .remainingAccounts([
        { pubkey: evidencePda(dispute, 0), isWritable: true, isSigner: false },
        { pubkey: evidencePda(dispute, 1), isWritable: true, isSigner: false },
      ])
      .signers([arb]).rpc();
    let afterPartial = await program.account.dispute.fetch(dispute);
    assert.equal(afterPartial.evidenceCleanupCursor, 2, "cursor debe avanzar a 2");
    assert.isNull(await provider.connection.getAccountInfo(evidencePda(dispute, 0)));
    assert.isNull(await provider.connection.getAccountInfo(evidencePda(dispute, 1)));
    const evRentRemaining = (await Promise.all([2, 3, 4].map(i => provider.connection.getAccountInfo(evidencePda(dispute, i))))).reduce((s, a) => s + (a?.lamports ?? 0), 0);
    const clientBefore = await provider.connection.getBalance(client.publicKey);
    // finalize debe cerrar restantes 3 evidencias y conservar payout: 40% cliente, 60% freelancer, shortfall 0 (bonds cubren 5%)
    await program.methods.finalizeDisputePayouts(jobId).accountsPartial({
      resolver: arb.publicKey, client: client.publicKey, job, dispute, escrow: arbFeePda(job),
      freelancer: f.publicKey, treasury, arbitrationTreasury: arbTreasury, config: configPda,
    }).remainingAccounts([
      { pubkey: evidencePda(dispute, 2), isWritable: true, isSigner: false },
      { pubkey: evidencePda(dispute, 3), isWritable: true, isSigner: false },
      { pubkey: evidencePda(dispute, 4), isWritable: true, isSigner: false },
      { pubkey: applicationPda(job, 0, f.publicKey), isWritable: true, isSigner: false },
      { pubkey: f.publicKey, isWritable: true, isSigner: false },
    ]).signers([arb]).rpc();
    for (let i = 2; i < 5; i++) assert.isNull(await provider.connection.getAccountInfo(evidencePda(dispute, i)), `ev ${i} cerrada`);
    const clientAfter = await provider.connection.getBalance(client.publicKey);
    assert.isAtLeast(clientAfter - clientBefore, evRentRemaining, "rent de evidencias restantes vuelve al cliente");
  });
});
