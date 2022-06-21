import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CoinFlip } from "../target/types/coin_flip";
import { PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { addSols, findAssociatedAddressForKey } from "./utils";
import { BN } from "bn.js";

describe("coin-flip", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;
  const wallet = anchor.web3.Keypair.generate();

  before("Initialize accounts", async () => {
    await addSols(provider, wallet.publicKey, 120 * LAMPORTS_PER_SOL);
  });

  it("Is initialized!", async () => {
    const [coinFlip, _] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("coin-flip")],
      program.programId
    );

    const [pool, _1] = await PublicKey.findProgramAddress(
      [Buffer.from("pool"), coinFlip.toBuffer()],
      program.programId
    );

    const amount = new BN(100 * LAMPORTS_PER_SOL);

    // Add your test here.
    const instruction = await program.methods
      .initializeCoinFlip({
        amount: amount,
      })
      .accounts({
        pool: pool,
        coinFlip: coinFlip,
        authority: wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();

    const signers = [wallet];

    const tx = new anchor.web3.Transaction();
    tx.add(instruction);
    const result = await program.provider.sendAndConfirm!(tx, signers);

    console.log("Your transaction signature", result);
  });
});
