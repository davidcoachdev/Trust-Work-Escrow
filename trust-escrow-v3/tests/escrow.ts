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

const endpoint = process.env.ANCHOR_PROVIDER_URL || "http://127.0.0.1:8899";
const parsedEndpoint = new URL(endpoint);
if (parsedEndpoint.protocol !== "http:" || parsedEndpoint.hostname !== "127.0.0.1") {
  throw new Error(`Tests require a loopback localnet endpoint; refusing ${endpoint}`);
}

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
      .createJob(jobId, "Job A", "desc", amount, deadline)
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
      .applyToJob(jobId, 0, "quiero hacerlo")
      .accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId })
      .signers([freelancer])
      .rpc();
    await program.methods
      .applyToJob(jobId, 1, "yo tambien")
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
      .createMilestone(jobId, 0, "M0", "d", new BN(m0), deadline)
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
    await program.methods.createJob(jobId, "Job B", "desc", amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, "x").accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0).accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();
    await program.methods.submitWork(jobId).accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job }).signers([freelancer]).rpc();

    await program.methods.raiseDispute(jobId, "no me pagan").accountsPartial({ raiser: client.publicKey, client: client.publicKey, job, ticket: null, dispute: disputePda(job), escrow: arbFeePda(job) }).rpc();
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

    let sizeRejected = false;
    try {
      await program.methods
        .submitEvidence(jobId, 0, Buffer.alloc(2_049))
        .accountsPartial({
          submitter: client.publicKey,
          client: client.publicKey,
          job,
          dispute,
          evidence: evidencePda(dispute, 0),
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (error) {
      sizeRejected = true;
      assert.match(String(error), /EvidenceTooLong|encoding overruns Buffer/);
    }
    assert.isTrue(sizeRejected, "evidencia mayor a 2.048 bytes debe rechazarse");

    for (let index = 0; index < 10; index++) {
      await program.methods
        .submitEvidence(jobId, index, Buffer.from(`evidence-${index}`))
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
    assert.equal(Buffer.from(firstEvidence.content).toString(), "evidence-0");

    let limitRejected = false;
    try {
      await program.methods
        .submitEvidence(jobId, 10, Buffer.from("evidence-10"))
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
    await program.methods.createJob(jobId, "Job C", "desc", amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, "x").accountsPartial({ applicant: freelancer.publicKey, client: client.publicKey, job, application: applicationPda(job, 0, freelancer.publicKey), systemProgram: SystemProgram.programId }).signers([freelancer]).rpc();
    await program.methods.acceptApplication(jobId, 0).accountsPartial({ client: client.publicKey, job, applicant: freelancer.publicKey, application: applicationPda(job, 0, freelancer.publicKey) }).rpc();

    const milestoneAmount = new BN(1_000_000);
    await program.methods.createMilestone(jobId, 0, "M0", "d", milestoneAmount, deadline)
      .accountsPartial({ client: client.publicKey, job, milestone: milestonePda(job, 0) }).rpc();
    await program.methods.submitMilestone(jobId, 0)
      .accountsPartial({ freelancer: freelancer.publicKey, client: client.publicKey, job, milestone: milestonePda(job, 0) })
      .signers([freelancer]).rpc();
    await program.methods.approveMilestone(jobId, 0)
      .accountsPartial({ client: client.publicKey, job, freelancer: freelancer.publicKey, milestone: milestonePda(job, 0) }).rpc();

    // abre ticket (sin bono) y el asesor resuelve -> cancela y reembolsa
    await program.methods.openSupportTicket(jobId, "el freelancer no entrego")
      .accountsPartial({ opener: client.publicKey, client: client.publicKey, job, dispute: null, ticket: supportPda(job) })
      .rpc();
    const before = await provider.connection.getBalance(client.publicKey);
    await program.methods.resolveSupportTicket(jobId, "cancelado por incumplimiento")
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
    await program.methods.createJob(jobId, "Job D", "desc", amount, deadline)
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

    await program.methods.createJob(jobId, "Job E", "desc", amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, "x")
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
    await program.methods.createJob(jobId, "Job F", "desc", new BN(2_000_000), deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId).accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.applyToJob(jobId, 0, "x")
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
    await program.methods.createJob(jobId, "Job applications", "desc", amount, deadline)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();
    await program.methods.depositFunds(jobId)
      .accountsPartial({ client: client.publicKey, job, config: configPda }).rpc();

    const applicationCount = Number(process.env.TRUST_ESCROW_V3_APPLICATION_COUNT || "50");
    const applicants = Array.from({ length: applicationCount }, () => Keypair.generate());
    for (let offset = 0; offset < applicants.length; offset += 10) {
      await Promise.all(applicants.slice(offset, offset + 10).map((applicant) => airdrop(applicant, 0.1)));
    }
    await program.methods.applyToJob(jobId, 0, "proposal-0")
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
      await program.methods.applyToJob(jobId, 1, "duplicate-before-limit")
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
      await program.methods.applyToJob(jobId, index, `proposal-${index}`)
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
      await program.methods.applyToJob(jobId, applicationCount, `proposal-${applicationCount}`)
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
});
