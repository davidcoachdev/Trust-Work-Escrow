import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { Escrow } from "../../target/types/escrow";
import { Keypair, PublicKey } from "@solana/web3.js";
import { assert } from "chai";

const programId = new PublicKey("J1c4QsjbV9bFEPrFQZZGe8GrGWFxNhtAhhrxJFK2xc1h");
const pda = (...seeds: Buffer[]) => PublicKey.findProgramAddressSync(seeds, programId)[0];
const jobPda = (client: PublicKey, jobId: BN) =>
  pda(Buffer.from("job"), client.toBuffer(), jobId.toArrayLike(Buffer, "le", 8));
const applicationPda = (job: PublicKey, index: number, applicant: PublicKey) =>
  pda(Buffer.from("application"), job.toBuffer(), Buffer.from([index]), applicant.toBuffer());

describe("individual application PDAs", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Escrow as Program<Escrow>;
  const client = provider.wallet.publicKey;

  it("derives each application from job, index and applicant and keeps the job compact", async () => {
    const jobId = new BN(Date.now());
    const job = jobPda(client, jobId);
    const applicant = Keypair.generate();
    const application = applicationPda(job, 0, applicant.publicKey);

    const info = await program.account.job.fetchNullable(job);
    assert.isNull(info);
    const applicationInfo = await program.account.application.fetchNullable(application);
    assert.isNull(applicationInfo);
    assert.notEqual(application.toBase58(), job.toBase58());
  });
});
