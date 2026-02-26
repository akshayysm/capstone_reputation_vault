import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { ReputationVault } from "../target/types/reputation_vault";
import { expect } from "chai";

describe("reputation_vault", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .ReputationVault as Program<ReputationVault>;

  const owner = provider.wallet;
  const user = provider.wallet;

  let vaultStatePda: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let reputationPda: anchor.web3.PublicKey;

  const requiredScore = new anchor.BN(10);
  const depositAmount = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL);

  it("Initialize Vault", async () => {
    [vaultStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state"), owner.publicKey.toBuffer()],
      program.programId
    );

    [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), vaultStatePda.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize(requiredScore)
      .accountsStrict({
        owner: owner.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("Initialize Reputation", async () => {
    [reputationPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reputation"), user.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initializeReputation()
      .accountsStrict({
        user: user.publicKey,
        reputation: reputationPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("Deposit SOL", async () => {
    const initialBalance =
      await provider.connection.getBalance(vaultPda);

    await program.methods
      .deposit(depositAmount)
      .accountsStrict({
        user: user.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalBalance =
      await provider.connection.getBalance(vaultPda);

    expect(finalBalance).to.equal(initialBalance + depositAmount.toNumber());
  });

  it("Withdraw should FAIL with low reputation", async () => {
    try {
      await program.methods
        .withdraw(depositAmount)
        .accountsStrict({
          user: user.publicKey,
          vaultState: vaultStatePda,
          vault: vaultPda,
          reputation: reputationPda,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      expect.fail("Withdrawal should have failed");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  it("Increase Reputation", async () => {
    await program.methods
      .increaseReputation(new anchor.BN(10))
      .accountsStrict({
        owner: owner.publicKey,
        vaultState: vaultStatePda,
        reputation: reputationPda,
      })
      .rpc();
  });

  it("Withdraw should SUCCEED", async () => {
    const initialBalance =
      await provider.connection.getBalance(vaultPda);

    await program.methods
      .withdraw(depositAmount)
      .accountsStrict({
        user: user.publicKey,
        vaultState: vaultStatePda,
        vault: vaultPda,
        reputation: reputationPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const finalBalance =
      await provider.connection.getBalance(vaultPda);

    expect(finalBalance).to.equal(initialBalance - depositAmount.toNumber());
  });
});