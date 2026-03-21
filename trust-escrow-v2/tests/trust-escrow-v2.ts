import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("trust-escrow-v2", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);

  const program = anchor.workspace.TrustEscrowV2 as Program;

  // Test accounts
  let admin: anchor.web3.Keypair;
  let client: anchor.web3.Keypair;
  let freelancer: anchor.web3.Keypair;
  let arbiter: anchor.web3.Keypair;

  before(async () => {
    admin = anchor.web3.Keypair.generate();
    client = anchor.web3.Keypair.generate();
    freelancer = anchor.web3.Keypair.generate();
    arbiter = anchor.web3.Keypair.generate();

    // Airdrop SOL to test accounts
    const tx = new anchor.web3.Transaction();
    
    const adminAirdrop = new anchor.web3.Transaction().add(
      SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: admin.publicKey,
        lamports: 10 * anchor.web3.LAMPORTS_PER_SOL,
      })
    );
    
    await provider.sendAndConfirm(adminAirdrop);
  });

  describe("Config", () => {
    it("should initialize config", async () => {
      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );

      const tx = await program.methods
        .initializeConfig(
          [admin.publicKey],
          1, // threshold
          admin.publicKey,
          5 // fee percent
        )
        .accounts({
          authority: admin.publicKey,
          config: configPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      console.log("Config initialized, tx:", tx);

      // Fetch and verify the config
      const config = await program.account.config.fetch(configPda);
      expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
      expect(config.feePercent).to.equal(5);
      expect(config.paused).to.equal(false);
    });
  });

  describe("User", () => {
    it("should create user", async () => {
      const [userPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), client.publicKey.toBuffer()],
        program.programId
      );

      const tx = await program.methods
        .createUser("testuser")
        .accounts({
          payer: client.publicKey,
          user: userPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      console.log("User created, tx:", tx);

      const user = await program.account.user.fetch(userPda);
      expect(user.username).to.equal("testuser");
      expect(user.walletPrincipal.toBase58()).to.equal(client.publicKey.toBase58());
      expect(user.activeWallet.toBase58()).to.equal(client.publicKey.toBase58());
    });

    it("should add secondary wallet", async () => {
      const [userPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), client.publicKey.toBuffer()],
        program.programId
      );

      const newWallet = anchor.web3.Keypair.generate();

      const tx = await program.methods
        .addWallet(newWallet.publicKey)
        .accounts({
          user: userPda,
          authority: client.publicKey,
          newWallet: newWallet.publicKey,
        })
        .signers([client])
        .rpc();

      console.log("Wallet added, tx:", tx);

      const user = await program.account.user.fetch(userPda);
      expect(user.walletsAsociadas.length).to.equal(2);
    });

    it("should set active wallet", async () => {
      const [userPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("user"), client.publicKey.toBuffer()],
        program.programId
      );

      const user = await program.account.user.fetch(userPda);
      const newWallet = user.walletsAsociadas[1];

      const tx = await program.methods
        .setActiveWallet(newWallet)
        .accounts({
          user: userPda,
          authority: client.publicKey,
        })
        .signers([client])
        .rpc();

      console.log("Active wallet set, tx:", tx);

      const updatedUser = await program.account.user.fetch(userPda);
      expect(updatedUser.activeWallet.toBase58()).to.equal(newWallet.toBase58());
    });
  });

  describe("Job", () => {
    let configPda: PublicKey;
    let jobPda: PublicKey;

    before(async () => {
      [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("config")],
        program.programId
      );
    });

    it("should create job", async () => {
      const jobId = new anchor.BN(1);
      [jobPda] = PublicKey.findProgramAddressSync(
        [Buffer.from("job"), client.publicKey.toBuffer(), jobId.toArrayLike(Buffer, "le", 8)],
        program.programId
      );

      const deadline = Math.floor(Date.now() / 1000) + 86400 * 7; // 7 days from now

      const tx = await program.methods
        .createJob(
          jobId,
          "Test Job",
          "Test Description",
          new anchor.BN(1000000000), // 1 SOL
          deadline,
          arbiter.publicKey
        )
        .accounts({
          client: client.publicKey,
          job: jobPda,
          config: configPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      console.log("Job created, tx:", tx);

      const job = await program.account.job.fetch(jobPda);
      expect(job.title).to.equal("Test Job");
      expect(job.amount.toNumber()).to.equal(1000000000);
      expect(job.client.toBase58()).to.equal(client.publicKey.toBase58());
    });

    it("should accept job", async () => {
      const jobId = new anchor.BN(1);

      const tx = await program.methods
        .acceptJob(jobId)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPda,
        })
        .signers([freelancer])
        .rpc();

      console.log("Job accepted, tx:", tx);

      const job = await program.account.job.fetch(jobPda);
      expect(job.freelancer.toBase58()).to.equal(freelancer.publicKey.toBase58());
    });

    it("should submit work", async () => {
      const jobId = new anchor.BN(1);

      const tx = await program.methods
        .submitWork(jobId)
        .accounts({
          freelancer: freelancer.publicKey,
          job: jobPda,
        })
        .signers([freelancer])
        .rpc();

      console.log("Work submitted, tx:", tx);

      const job = await program.account.job.fetch(jobPda);
      expect(job.status.submitted).to.not.be.undefined;
    });

    it("should approve work", async () => {
      const jobId = new anchor.BN(1);

      // Get initial balances
      const initialFreelancerBalance = await provider.connection.getBalance(freelancer.publicKey);
      const initialTreasuryBalance = await provider.connection.getBalance(admin.publicKey);

      const tx = await program.methods
        .approveWork(jobId)
        .accounts({
          client: client.publicKey,
          job: jobPda,
          freelancer: freelancer.publicKey,
          treasury: admin.publicKey,
          config: configPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([client])
        .rpc();

      console.log("Work approved, tx:", tx);

      // Verify funds transferred
      const finalFreelancerBalance = await provider.connection.getBalance(freelancer.publicKey);
      expect(finalFreelancerBalance).to.be.greaterThan(initialFreelancerBalance);
    });
  });
});