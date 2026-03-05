import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TrustEscrow } from "../target/types/trust_escrow";
import { expect } from "chai";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import BN from "bn.js";

describe("trust-escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.trustEscrow as Program<TrustEscrow>;

  const authority = provider.wallet;
  const treasury = Keypair.generate();
  const client = Keypair.generate();
  const freelancer = Keypair.generate();
  const arbiter = Keypair.generate();

  const JOB_ID = new BN(1);
  const JOB_AMOUNT = new BN(1_000_000_000); // 1 SOL
  const FEE_PERCENT = 5;

  const getConfigPDA = (): PublicKey => {
    return PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId,
    )[0];
  };

  const getJobPDA = (clientKey: PublicKey, jobId: BN): PublicKey => {
    return PublicKey.findProgramAddressSync(
      [
        Buffer.from("job"),
        clientKey.toBuffer(),
        jobId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId,
    )[0];
  };

  before(async () => {
    const airdropAmount = 10 * LAMPORTS_PER_SOL;
    for (const kp of [client, freelancer, arbiter]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        airdropAmount,
      );
      await provider.connection.confirmTransaction(sig);
    }
  });

  // =========================================================================
  // initialize_config
  // =========================================================================
  describe("initialize_config", () => {
    it("initializes config correctly", async () => {
      const configPDA = getConfigPDA();

      await program.methods
        .initializeConfig()
        .accounts({
          authority: authority.publicKey,
          treasury: treasury.publicKey,
          config: configPDA,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      const config = await program.account.config.fetch(configPDA);
      expect(config.authority.toString()).to.equal(
        authority.publicKey.toString(),
      );
      expect(config.treasury.toString()).to.equal(
        treasury.publicKey.toString(),
      );
      expect(config.feePercent).to.equal(FEE_PERCENT);
      expect(config.paused).to.equal(false);
    });

    it("cannot initialize config twice", async () => {
      try {
        await program.methods
          .initializeConfig()
          .accounts({
            authority: authority.publicKey,
            treasury: treasury.publicKey,
            config: getConfigPDA(),
            systemProgram: SystemProgram.programId,
          })
          .rpc();
        expect.fail("Should have thrown");
      } catch (_err) {
        // Expected — account already exists
      }
    });
  });

  // =========================================================================
  // create_job
  // =========================================================================
  describe("create_job", () => {
    it("creates a job correctly", async () => {
      const jobPDA = getJobPDA(client.publicKey, JOB_ID);
      const deadline = Math.floor(Date.now() / 1000) + 86400;

      await program.methods
        .createJob(
          JOB_ID,
          "Build a website",
          "Create a landing page with React",
          JOB_AMOUNT,
          new BN(deadline),
        )
        .accounts({
          client: client.publicKey,
          arbiter: arbiter.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      expect(job.client.toString()).to.equal(client.publicKey.toString());
      expect(job.arbiter.toString()).to.equal(arbiter.publicKey.toString());
      expect(job.freelancer).to.be.null;
      expect(job.amount.toNumber()).to.equal(JOB_AMOUNT.toNumber());
      expect(job.feePercent).to.equal(FEE_PERCENT);
      expect(job.feeAmount.toNumber()).to.equal(
        (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100,
      );
      expect(job.title).to.equal("Build a website");
      expect(job.description).to.equal("Create a landing page with React");
      expect(Object.keys(job.status)[0]).to.equal("created");
    });

    it("fails with empty title", async () => {
      const jobId = new BN(99);
      const jobPDA = getJobPDA(client.publicKey, jobId);

      try {
        await program.methods
          .createJob(
            jobId,
            "",
            "desc",
            JOB_AMOUNT,
            new BN(Math.floor(Date.now() / 1000) + 86400),
          )
          .accounts({
            client: client.publicKey,
            arbiter: arbiter.publicKey,
            job: jobPDA,
            config: getConfigPDA(),
            systemProgram: SystemProgram.programId,
          })
          .signers([client])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("EmptyTitle");
      }
    });

    it("fails with amount too small", async () => {
      const jobId = new BN(98);
      const jobPDA = getJobPDA(client.publicKey, jobId);

      try {
        await program.methods
          .createJob(
            jobId,
            "Tiny job",
            "desc",
            new BN(100),
            new BN(Math.floor(Date.now() / 1000) + 86400),
          )
          .accounts({
            client: client.publicKey,
            arbiter: arbiter.publicKey,
            job: jobPDA,
            config: getConfigPDA(),
            systemProgram: SystemProgram.programId,
          })
          .signers([client])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("AmountTooSmall");
      }
    });
  });

  // =========================================================================
  // deposit_funds
  // =========================================================================
  describe("deposit_funds", () => {
    it("deposits funds correctly", async () => {
      const jobPDA = getJobPDA(client.publicKey, JOB_ID);
      const jobBefore = await program.account.job.fetch(jobPDA);
      const clientBefore = await provider.connection.getBalance(
        client.publicKey,
      );

      await program.methods
        .depositFunds(JOB_ID)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      expect(Object.keys(job.status)[0]).to.equal("funded");

      const clientAfter = await provider.connection.getBalance(
        client.publicKey,
      );
      const expectedDeposit =
        JOB_AMOUNT.toNumber() + jobBefore.feeAmount.toNumber();
      // Client should lose at least the deposit (plus some tx fees)
      expect(clientBefore - clientAfter).to.be.greaterThanOrEqual(
        expectedDeposit,
      );
    });
  });

  // =========================================================================
  // accept_job
  // =========================================================================
  describe("accept_job", () => {
    it("freelancer accepts job", async () => {
      const jobPDA = getJobPDA(client.publicKey, JOB_ID);

      await program.methods
        .acceptJob(JOB_ID)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([freelancer])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      expect(Object.keys(job.status)[0]).to.equal("inProgress");
      expect(job.freelancer.toString()).to.equal(
        freelancer.publicKey.toString(),
      );
    });

    it("client cannot accept own job", async () => {
      const jobId = new BN(51);
      const jobPDA = getJobPDA(client.publicKey, jobId);
      const deadline = Math.floor(Date.now() / 1000) + 86400;

      await program.methods
        .createJob(jobId, "Self-test", "desc", JOB_AMOUNT, new BN(deadline))
        .accounts({
          client: client.publicKey,
          arbiter: arbiter.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .depositFunds(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      try {
        await program.methods
          .acceptJob(jobId)
          .accounts({
            freelancer: client.publicKey,
            job: jobPDA,
            config: getConfigPDA(),
          })
          .signers([client])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("CannotWorkOnOwnJob");
      }
    });
  });

  // =========================================================================
  // submit_work
  // =========================================================================
  describe("submit_work", () => {
    it("freelancer submits work", async () => {
      const jobPDA = getJobPDA(client.publicKey, JOB_ID);

      await program.methods
        .submitWork(JOB_ID)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([freelancer])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      expect(Object.keys(job.status)[0]).to.equal("submitted");
    });
  });

  // =========================================================================
  // approve_work (critical bug fix verified here)
  // =========================================================================
  describe("approve_work", () => {
    it("client approves and freelancer gets paid, treasury gets fee", async () => {
      const jobPDA = getJobPDA(client.publicKey, JOB_ID);

      const freelancerBefore = await provider.connection.getBalance(
        freelancer.publicKey,
      );
      const treasuryBefore = await provider.connection.getBalance(
        treasury.publicKey,
      );

      await program.methods
        .approveWork(JOB_ID)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          freelancer: freelancer.publicKey,
          treasury: treasury.publicKey,
          config: getConfigPDA(),
        })
        .signers([client])
        .rpc();

      const freelancerAfter = await provider.connection.getBalance(
        freelancer.publicKey,
      );
      const treasuryAfter = await provider.connection.getBalance(
        treasury.publicKey,
      );

      const expectedPayment = JOB_AMOUNT.toNumber();
      const expectedFee = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;

      // Freelancer receives payment
      expect(freelancerAfter - freelancerBefore).to.equal(expectedPayment);
      // Treasury receives fee
      expect(treasuryAfter - treasuryBefore).to.equal(expectedFee);

      // Job account should be closed
      try {
        await program.account.job.fetch(jobPDA);
        expect.fail("Job account should be closed");
      } catch (_err) {
        // Expected — account closed by Anchor
      }
    });
  });

  // =========================================================================
  // dispute flow: reject → resolve
  // =========================================================================
  describe("dispute flow", () => {
    const DISPUTE_JOB_ID = new BN(2);

    before(async () => {
      const jobPDA = getJobPDA(client.publicKey, DISPUTE_JOB_ID);
      const deadline = Math.floor(Date.now() / 1000) + 86400;

      // Create → Deposit → Accept → Submit
      await program.methods
        .createJob(
          DISPUTE_JOB_ID,
          "Dispute Test",
          "Testing disputes",
          JOB_AMOUNT,
          new BN(deadline),
        )
        .accounts({
          client: client.publicKey,
          arbiter: arbiter.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .depositFunds(DISPUTE_JOB_ID)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .acceptJob(DISPUTE_JOB_ID)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([freelancer])
        .rpc();

      await program.methods
        .submitWork(DISPUTE_JOB_ID)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([freelancer])
        .rpc();
    });

    it("client rejects work → dispute opens", async () => {
      const jobPDA = getJobPDA(client.publicKey, DISPUTE_JOB_ID);

      await program.methods
        .rejectWork(DISPUTE_JOB_ID, "Work is incomplete")
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([client])
        .rpc();

      const job = await program.account.job.fetch(jobPDA);
      expect(Object.keys(job.status)[0]).to.equal("disputed");
      expect(job.disputeReason).to.equal("Work is incomplete");
    });

    it("non-arbiter cannot resolve dispute", async () => {
      const jobPDA = getJobPDA(client.publicKey, DISPUTE_JOB_ID);
      const fakeArbiter = Keypair.generate();
      const sig = await provider.connection.requestAirdrop(
        fakeArbiter.publicKey,
        LAMPORTS_PER_SOL,
      );
      await provider.connection.confirmTransaction(sig);

      try {
        await program.methods
          .resolveDispute(DISPUTE_JOB_ID, 70)
          .accounts({
            arbiter: fakeArbiter.publicKey,
            client: client.publicKey,
            job: jobPDA,
            freelancer: freelancer.publicKey,
            treasury: treasury.publicKey,
            config: getConfigPDA(),
          })
          .signers([fakeArbiter])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        // Expected — wrong arbiter
      }
    });

    it("arbiter resolves dispute (70% freelancer, 30% client)", async () => {
      const jobPDA = getJobPDA(client.publicKey, DISPUTE_JOB_ID);

      const freelancerBefore = await provider.connection.getBalance(
        freelancer.publicKey,
      );
      const clientBefore = await provider.connection.getBalance(
        client.publicKey,
      );
      const treasuryBefore = await provider.connection.getBalance(
        treasury.publicKey,
      );

      await program.methods
        .resolveDispute(DISPUTE_JOB_ID, 70)
        .accounts({
          arbiter: arbiter.publicKey,
          client: client.publicKey,
          job: jobPDA,
          freelancer: freelancer.publicKey,
          treasury: treasury.publicKey,
          config: getConfigPDA(),
        })
        .signers([arbiter])
        .rpc();

      const freelancerAfter = await provider.connection.getBalance(
        freelancer.publicKey,
      );
      const clientAfter = await provider.connection.getBalance(
        client.publicKey,
      );
      const treasuryAfter = await provider.connection.getBalance(
        treasury.publicKey,
      );

      const expectedFreelancer = Math.floor((JOB_AMOUNT.toNumber() * 70) / 100);
      const expectedFee = (JOB_AMOUNT.toNumber() * FEE_PERCENT) / 100;

      expect(freelancerAfter - freelancerBefore).to.equal(expectedFreelancer);
      expect(treasuryAfter - treasuryBefore).to.equal(expectedFee);
      // Client gets 30% of amount + rent via close = client
      expect(clientAfter).to.be.greaterThan(clientBefore);
    });
  });

  // =========================================================================
  // cancel_job
  // =========================================================================
  describe("cancel_job", () => {
    it("cancels a created (unfunded) job", async () => {
      const jobId = new BN(3);
      const jobPDA = getJobPDA(client.publicKey, jobId);
      const deadline = Math.floor(Date.now() / 1000) + 86400;

      await program.methods
        .createJob(jobId, "Cancel Test", "desc", JOB_AMOUNT, new BN(deadline))
        .accounts({
          client: client.publicKey,
          arbiter: arbiter.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .cancelJob(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([client])
        .rpc();

      try {
        await program.account.job.fetch(jobPDA);
        expect.fail("Job should be closed");
      } catch (_err) {
        // Expected
      }
    });

    it("cancels a funded job and refunds client", async () => {
      const jobId = new BN(4);
      const jobPDA = getJobPDA(client.publicKey, jobId);
      const deadline = Math.floor(Date.now() / 1000) + 86400;

      await program.methods
        .createJob(jobId, "Funded Cancel", "desc", JOB_AMOUNT, new BN(deadline))
        .accounts({
          client: client.publicKey,
          arbiter: arbiter.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      await program.methods
        .depositFunds(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      const clientBefore = await provider.connection.getBalance(
        client.publicKey,
      );

      await program.methods
        .cancelJob(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPDA,
          config: getConfigPDA(),
        })
        .signers([client])
        .rpc();

      const clientAfter = await provider.connection.getBalance(
        client.publicKey,
      );
      // Client should receive refund (amount + fee + rent - tx fee)
      expect(clientAfter).to.be.greaterThan(clientBefore);
    });
  });

  // =========================================================================
  // pause / unpause
  // =========================================================================
  describe("pause/unpause", () => {
    it("authority pauses program", async () => {
      await program.methods
        .pauseProgram()
        .accounts({
          authority: authority.publicKey,
          config: getConfigPDA(),
        })
        .rpc();

      const config = await program.account.config.fetch(getConfigPDA());
      expect(config.paused).to.equal(true);
    });

    it("cannot create job while paused", async () => {
      const jobId = new BN(100);
      const jobPDA = getJobPDA(client.publicKey, jobId);

      try {
        await program.methods
          .createJob(
            jobId,
            "Paused",
            "desc",
            JOB_AMOUNT,
            new BN(Math.floor(Date.now() / 1000) + 86400),
          )
          .accounts({
            client: client.publicKey,
            arbiter: arbiter.publicKey,
            job: jobPDA,
            config: getConfigPDA(),
            systemProgram: SystemProgram.programId,
          })
          .signers([client])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("ProgramPaused");
      }
    });

    it("authority unpauses program", async () => {
      await program.methods
        .unpauseProgram()
        .accounts({
          authority: authority.publicKey,
          config: getConfigPDA(),
        })
        .rpc();

      const config = await program.account.config.fetch(getConfigPDA());
      expect(config.paused).to.equal(false);
    });

    it("non-authority cannot pause", async () => {
      try {
        await program.methods
          .pauseProgram()
          .accounts({
            authority: client.publicKey,
            config: getConfigPDA(),
          })
          .signers([client])
          .rpc();
        expect.fail("Should have thrown");
      } catch (err) {
        expect(err.toString()).to.include("NotAuthorized");
      }
    });
  });
});
