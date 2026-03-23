import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { assert } from "chai";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/pubkey";

// Configure the provider
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

// Get the program from the workspace
const program = anchor.workspace.TrustEscrowV2 as Program;

// Constants matching lib.rs
const PROGRAM_ID = new PublicKey("TesT3XPqD3WFFVTY4BTwZ3sJpY7C7hF3Z6K2oX3i7jB");
const FEE_PERCENT = 5;

describe("Trust Work Escrow v2 - Integration Tests", () => {
  // Test accounts
  const admin = Keypair.generate();
  const client = Keypair.generate();
  const freelancer = Keypair.generate();
  const arbiter = Keypair.generate();
  const treasury = Keypair.generate();

  // PDAs
  let configPDA: PublicKey;
  let userClientPDA: PublicKey;
  let userFreelancerPDA: PublicKey;
  let jobPDA: PublicKey;
  let arbiterPoolPDA: PublicKey;
  let disputePDA: PublicKey;
  let milestonePDA: PublicKey;

  const jobId = new anchor.BN(1);

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(client.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(freelancer.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(arbiter.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(treasury.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);

    // Wait for confirmations
    await new Promise((r) => setTimeout(r, 1000));

    // Derive PDAs
    [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      PROGRAM_ID
    );

    [userClientPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), client.publicKey.toBuffer()],
      PROGRAM_ID
    );

    [userFreelancerPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user"), freelancer.publicKey.toBuffer()],
      PROGRAM_ID
    );

    [jobPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("job"), client.publicKey.toBuffer(), jobId.toBuffer("little", 8)],
      PROGRAM_ID
    );

    [arbiterPoolPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("arbiter_pool")],
      PROGRAM_ID
    );

    [disputePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("dispute"), jobPDA.toBuffer()],
      PROGRAM_ID
    );

    [milestonePDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("milestone"), jobPDA.toBuffer(), Buffer.from([0])],
      PROGRAM_ID
    );
  });

  describe("Config", () => {
    it("should initialize config", async () => {
      const multisigOwners = [admin.publicKey];

      await program.methods
        .initializeConfig(multisigOwners, 1, treasury.publicKey, FEE_PERCENT)
        .accounts({
          authority: admin.publicKey,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      const config = await program.account.config.fetch(configPDA);
      assert.equal(config.admin.toString(), admin.publicKey.toString());
      assert.equal(config.treasury.toString(), treasury.publicKey.toString());
      assert.equal(config.feePercent, FEE_PERCENT);
      assert.equal(config.paused, false);
    });

    it("should pause and unpause program", async () => {
      // Pause
      await program.methods
        .pause()
        .accounts({
          authority: admin.publicKey,
          config: configPDA,
        })
        .signers([admin])
        .rpc();

      let config = await program.account.config.fetch(configPDA);
      assert.equal(config.paused, true);

      // Unpause
      await program.methods
        .unpause()
        .accounts({
          authority: admin.publicKey,
          config: configPDA,
        })
        .signers([admin])
        .rpc();

      config = await program.account.config.fetch(configPDA);
      assert.equal(config.paused, false);
    });
  });

  describe("User", () => {
    it("should create user for client", async () => {
      await program.methods
        .createUser("client_user")
        .accounts({
          authority: client.publicKey,
          user: userClientPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const user = await program.account.user.fetch(userClientPDA);
      assert.equal(user.username, "client_user");
      assert.equal(user.walletPrincipal.toString(), client.publicKey.toString());
      assert.equal(user.activeWallet.toString(), client.publicKey.toString());
    });

    it("should create user for freelancer", async () => {
      await program.methods
        .createUser("freelancer_user")
        .accounts({
          authority: freelancer.publicKey,
          user: userFreelancerPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([freelancer])
        .rpc();

      const user = await program.account.user.fetch(userFreelancerPDA);
      assert.equal(user.username, "freelancer_user");
    });

    it("should update user bio", async () => {
      await program.methods
        .updateUser("New bio for client")
        .accounts({
          authority: client.publicKey,
          user: userClientPDA,
        })
        .signers([client])
        .rpc();

      const user = await program.account.user.fetch(userClientPDA);
      assert.equal(user.bio, "New bio for client");
    });

    it("should add secondary wallet", async () => {
      const secondaryWallet = Keypair.generate().publicKey;

      await program.methods
        .addWallet(secondaryWallet)
        .accounts({
          authority: client.publicKey,
          user: userClientPDA,
        })
        .signers([client])
        .rpc();

      const user = await program.account.user.fetch(userClientPDA);
      assert.isTrue(user.wallets.some((w: PublicKey) => w.toString() === secondaryWallet.toString()));
    });
  });

  describe("Arbiter Pool", () => {
    it("should create arbiter pool", async () => {
      await program.methods
        .createArbiterPool()
        .accounts({
          admin: admin.publicKey,
          pool: arbiterPoolPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.arbiterPool.fetch(arbiterPoolPDA);
      assert.equal(pool.authority.toString(), admin.publicKey.toString());
      assert.equal(pool.arbiters.length, 0);
    });

    it("should add arbiter", async () => {
      await program.methods
        .addArbiter(arbiter.publicKey)
        .accounts({
          admin: admin.publicKey,
          pool: arbiterPoolPDA,
        })
        .signers([admin])
        .rpc();

      const pool = await program.account.arbiterPool.fetch(arbiterPoolPDA);
      assert.isTrue(pool.arbiters.some((a: PublicKey) => a.toString() === arbiter.publicKey.toString()));
    });
  });

  describe("Job Lifecycle", () => {
    const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400); // 1 day from now
    const jobAmount = new anchor.BN(2 * anchor.web3.LAMPORTS_PER_SOL); // 2 SOL

    it("should create job", async () => {
      await program.methods
        .createJob(jobId, "Build Landing Page", "Create a landing page for my startup", jobAmount, deadline)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      assert.equal(job.title, "Build Landing Page");
      assert.equal(job.amount.toString(), jobAmount.toString());
      assert.equal(job.client.toString(), client.publicKey.toString());
      assert.isNull(job.freelancer);
    });

    it("should deposit funds", async () => {
      const fee = jobAmount.muln(FEE_PERCENT).divn(100);
      const total = jobAmount.add(fee);

      await program.methods
        .depositFunds(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      // Status 1 = ApplicationsOpen
      assert.equal(job.status, 1);
    });

    it("should apply to job", async () => {
      await program.methods
        .applyToJob(jobId, "I can build this landing page with React and Tailwind", false)
        .accounts({
          applicant: freelancer.publicKey,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      assert.equal(job.applications.length, 1);
      assert.equal(job.applications[0].proposal, "I can build this landing page with React and Tailwind");
      assert.equal(job.applications[0].status, 0); // Pending
    });

    it("should accept application", async () => {
      await program.methods
        .acceptApplication(jobId, freelancer.publicKey, false)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      assert.isNotNull(job.freelancer);
      assert.equal(job.freelancer.toString(), freelancer.publicKey.toString());
      // Status 2 = InProgress
      assert.equal(job.status, 2);
    });

    it("should submit work", async () => {
      await program.methods
        .submitWork(jobId)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      // Status 3 = Submitted
      assert.equal(job.status, 3);
      assert.isNotNull(job.submittedAt);
    });

    it("should approve work and transfer funds", async () => {
      const freelancerBalanceBefore = await provider.connection.getBalance(freelancer.publicKey);

      await program.methods
        .approveWork(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: configPDA,
          freelancer: freelancer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      // Status 4 = Approved
      assert.equal(job.status, 4);

      const freelancerBalanceAfter = await provider.connection.getBalance(freelancer.publicKey);
      assert.isAbove(freelancerBalanceAfter, freelancerBalanceBefore);
    });

    it("should reject - test dispute flow", async () => {
      // Create another job for reject test
      const jobId2 = new anchor.BN(2);
      const [jobPDA2] = PublicKey.findProgramAddressSync(
        [Buffer.from("job"), client.publicKey.toBuffer(), jobId2.toBuffer("little", 8)],
        PROGRAM_ID
      );

      const fee = jobAmount.muln(FEE_PERCENT).divn(100);
      const total = jobAmount.add(fee);

      // Create and fund job
      await program.methods
        .createJob(jobId2, "Another Job", "Test reject flow", jobAmount, deadline)
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .depositFunds(jobId2)
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .applyToJob(jobId2, "My proposal", false)
        .accounts({
          applicant: freelancer.publicKey,
          job: jobPDA2,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      await program.methods
        .acceptApplication(jobId2, freelancer.publicKey, false)
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
        })
        .signers([client])
        .rpc();

      await program.methods
        .submitWork(jobId2)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA2,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      // Reject
      await program.methods
        .rejectWork(jobId2, "Work does not meet requirements")
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA2);
      // Status 5 = Disputed
      assert.equal(job.status, 5);
    });
  });

  describe("Cancel Job", () => {
    const jobIdCancel = new anchor.BN(99);
    const jobAmount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
    const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

    it("should cancel unfunded job", async () => {
      const [jobPDA2] = PublicKey.findProgramAddressSync(
        [Buffer.from("job"), client.publicKey.toBuffer(), jobIdCancel.toBuffer("little", 8)],
        PROGRAM_ID
      );

      await program.methods
        .createJob(jobIdCancel, "Cancel Test Job", "Job to be cancelled", jobAmount, deadline)
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .cancelJob(jobIdCancel)
        .accounts({
          client: client.publicKey,
          job: jobPDA2,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA2);
      // Status 6 = Cancelled
      assert.equal(job.status, 6);
    });
  });

  describe("Dispute Flow", () => {
    it("should raise dispute", async () => {
      const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);

      await program.methods
        .raiseDispute(jobId, "Quality dispute - not satisfied", deadline)
        .accounts({
          raiser: client.publicKey,
          job: jobPDA,
          dispute: disputePDA,
          client: client.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const dispute = await program.account.dispute.fetch(disputePDA);
      assert.equal(dispute.raisedBy.toString(), client.publicKey.toString());
      // Status 0 = Open
      assert.equal(dispute.status, 0);
    });

    it("should submit evidence", async () => {
      await program.methods
        .submitEvidence(jobId, "Here is evidence of incomplete work")
        .accounts({
          submitter: freelancer.publicKey,
          dispute: disputePDA,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      const dispute = await program.account.dispute.fetch(disputePDA);
      assert.equal(dispute.evidence.length, 1);
      assert.equal(dispute.evidence[0].content, "Here is evidence of incomplete work");
    });

    it("should assign arbiter", async () => {
      await program.methods
        .assignArbiter(jobId)
        .accounts({
          arbiter: arbiter.publicKey,
          dispute: disputePDA,
          pool: arbiterPoolPDA,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([arbiter])
        .rpc();

      const dispute = await program.account.dispute.fetch(disputePDA);
      assert.isNotNull(dispute.arbiter);
      assert.equal(dispute.arbiter.toString(), arbiter.publicKey.toString());
      // Status 2 = ArbiterAssigned
      assert.equal(dispute.status, 2);
    });

    it("should resolve dispute", async () => {
      await program.methods
        .resolveDispute(jobId, "Freelancer partially fulfilled requirements", 70)
        .accounts({
          arbiter: arbiter.publicKey,
          dispute: disputePDA,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([arbiter])
        .rpc();

      const dispute = await program.account.dispute.fetch(disputePDA);
      assert.equal(dispute.clientPayoutPercent, 30);
      assert.equal(dispute.freelancerPayoutPercent, 70);
      // Status 3 = Resolved
      assert.equal(dispute.status, 3);
    });
  });

  describe("Milestone Flow", () => {
    const milestoneIndex = 0;
    const milestoneAmount = new anchor.BN(0.5 * anchor.web3.LAMPORTS_PER_SOL);
    const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400 * 7);

    it("should create milestone", async () => {
      await program.methods
        .createMilestone(
          jobId,
          "Phase 1: Design",
          "Create wireframes and mockups",
          milestoneAmount,
          deadline,
          milestoneIndex
        )
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          milestone: milestonePDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const milestone = await program.account.milestone.fetch(milestonePDA);
      assert.equal(milestone.title, "Phase 1: Design");
      assert.equal(milestone.amount.toString(), milestoneAmount.toString());
      assert.equal(milestone.index, 0);
    });

    it("should submit milestone", async () => {
      await program.methods
        .submitMilestone(jobId, milestoneIndex)
        .accounts({
          freelancer: freelancer.publicKey,
          milestone: milestonePDA,
          job: jobPDA,
          client: client.publicKey,
        })
        .signers([freelancer])
        .rpc();

      const milestone = await program.account.milestone.fetch(milestonePDA);
      // Status 1 = Submitted
      assert.equal(milestone.status, 1);
      assert.isNotNull(milestone.submittedAt);
    });

    it("should approve milestone and transfer funds", async () => {
      const freelancerBalanceBefore = await provider.connection.getBalance(freelancer.publicKey);

      await program.methods
        .approveMilestone(jobId, milestoneIndex)
        .accounts({
          client: client.publicKey,
          milestone: milestonePDA,
          job: jobPDA,
          freelancer: freelancer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const milestone = await program.account.milestone.fetch(milestonePDA);
      // Status 2 = Approved
      assert.equal(milestone.status, 2);
      assert.isNotNull(milestone.approvedAt);

      const freelancerBalanceAfter = await provider.connection.getBalance(freelancer.publicKey);
      assert.isAbove(freelancerBalanceAfter, freelancerBalanceBefore);
    });
  });

  describe("Team", () => {
    it("should create team", async () => {
      const [teamPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("team"), client.publicKey.toBuffer()],
        PROGRAM_ID
      );

      await program.methods
        .createTeam("Freelancer Team Alpha", "Best frontend team")
        .accounts({
          owner: client.publicKey,
          team: teamPDA,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const team = await program.account.team.fetch(teamPDA);
      assert.equal(team.name, "Freelancer Team Alpha");
      assert.equal(team.members.length, 1);
      assert.equal(team.owner.toString(), client.publicKey.toString());
    });

    it("should add team member", async () => {
      const [teamPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("team"), client.publicKey.toBuffer()],
        PROGRAM_ID
      );
      const newMember = Keypair.generate().publicKey;

      await program.methods
        .addTeamMember(newMember, { contributor: {} }) // <--- Nota: Ajusta este objeto si tu enum 'MemberRole' en Rust es diferente
        .accounts({ owner: client.publicKey, team: teamPDA })
        .signers([client])
        .rpc();

      const team = await program.account.team.fetch(teamPDA);
      assert.equal(team.members.length, 2);
    });
  });

  describe("Treasury", () => {
    it("should update treasury", async () => {
      const newTreasury = Keypair.generate().publicKey;

      await program.methods
        .updateTreasury(newTreasury)
        .accounts({
          admin: admin.publicKey,
          config: configPDA,
        })
        .signers([admin])
        .rpc();

      const config = await program.account.config.fetch(configPDA);
      assert.equal(config.treasury.toString(), newTreasury.toString());
    });

    it("should withdraw treasury funds", async () => {
      const config = await program.account.config.fetch(configPDA);
      await program.methods
        .withdrawTreasury(new anchor.BN(1000))
        .accounts({
          admin: admin.publicKey,
          config: configPDA,
          treasury: config.treasury,
        })
        .signers([admin])
        .rpc();
    });
  });
});
