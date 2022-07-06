import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CoinFlip } from "../target/types/coin_flip";
import { PublicKey, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import {
  addSols,
  findAssociatedAddressForKey,
  getClaimantAddress,
  getCoinFlipAddress,
} from "./utils";
import { BN } from "bn.js";
import { assert } from "chai";

describe("coin-flip", () => {
  const provider = anchor.AnchorProvider.env();
  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const program = anchor.workspace.CoinFlip as Program<CoinFlip>;
  const treasury = anchor.web3.Keypair.generate();
  const player = anchor.web3.Keypair.generate();

  before("Initialize accounts", async () => {
    await addSols(provider, treasury.publicKey, 120 * LAMPORTS_PER_SOL);
    await addSols(provider, player.publicKey, 2 * LAMPORTS_PER_SOL);
  });

  it("Is initialized with balance!", async () => {
    const [coinFlip, _1] = await getCoinFlipAddress(program);

    const amount = new BN(110 * LAMPORTS_PER_SOL);

    // Add your test here.
    const instruction = await program.methods
      .initializeCoinFlip({
        amount: amount,
        minimumTokens: new BN(1 * LAMPORTS_PER_SOL),
      })
      .accounts({
        coinFlip: coinFlip,
        authority: treasury.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const signers = [treasury];

    const tx = new anchor.web3.Transaction();
    tx.add(instruction);
    const result = await program.provider.sendAndConfirm!(tx, signers);

    const bal = await program.provider.connection.getBalance(coinFlip);
    const solBalance = +bal.toString() / LAMPORTS_PER_SOL;

    assert.isOk(solBalance > 110, "Balance should be greater than 110");

    console.log("Your transaction signature", result);
  });

  it("Bet head", async () => {
    await addSols(provider, player.publicKey, 2 * LAMPORTS_PER_SOL);

    const [coinFlip, _1] = await getCoinFlipAddress(program);

    const [claimant, claimantBump] = await getClaimantAddress(
      program,
      player.publicKey
    );

    const amount = new BN(1 * LAMPORTS_PER_SOL);

    const value: BetType = { head: {} };

    const instruction = await program.methods
      .bet({
        amount: amount,
        betType: value,
        claimantBump: claimantBump,
      })
      .accounts({
        coinFlip: coinFlip,
        claimant: claimant,
        payer: player.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const signers = [player];

    const tx = new anchor.web3.Transaction();
    tx.add(instruction);

    try {
      const result = await program.provider.sendAndConfirm!(tx, signers);

      const bal = await program.provider.connection.getBalance(
        player.publicKey
      );
      const solBalance = +bal.toString() / LAMPORTS_PER_SOL;

      //console.log(solBalance);
    } catch (err) {
      console.log(err);
    }

    try {
      const result = await program.account.claimant.fetch(claimant);

      console.log(result);
      if (result.success) {
        const instruction = await program.methods
          .claimPrize({
            claimantBump: claimantBump,
          })
          .accounts({
            claimant: claimant,
            payer: player.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .instruction();

        const signers = [player];

        const tx = new anchor.web3.Transaction();
        tx.add(instruction);
        await program.provider.sendAndConfirm!(tx, signers);
        console.log("You've won!");
        //console.log(result2);
      }
    } catch (err) {
      console.log("sorry you have lost");
    }
  });

  it("Bet tail", async () => {
    await addSols(provider, player.publicKey, 3 * LAMPORTS_PER_SOL);

    const [coinFlip, _1] = await getCoinFlipAddress(program);

    const [claimant, claimantBump] = await getClaimantAddress(
      program,
      player.publicKey
    );

    const amount = new BN(1 * LAMPORTS_PER_SOL);

    const value: BetType = { tail: {} };
    const instruction = await program.methods
      .bet({
        amount: amount,
        betType: value,
        claimantBump: claimantBump,
      })
      .accounts({
        coinFlip: coinFlip,
        claimant: claimant,
        payer: player.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    const signers = [player];

    const tx = new anchor.web3.Transaction();
    tx.add(instruction);

    try {
      const result = await program.provider.sendAndConfirm!(tx, signers);

      const playerBal = await program.provider.connection.getBalance(
        player.publicKey
      );
      const playerBalance = +playerBal.toString() / LAMPORTS_PER_SOL;

      //console.log(playerBalance);

      const treasuryBal = await program.provider.connection.getBalance(
        treasury.publicKey
      );

      const treasuryBalance = +treasuryBal.toString() / LAMPORTS_PER_SOL;

      //console.log(treasuryBalance);
    } catch (err) {
      console.log(err);
    }

    try {
      const result = await program.account.claimant.fetch(claimant);

      console.log(result);
      if (result.success) {
        const instruction = await program.methods
          .claimPrize({
            claimantBump: claimantBump,
          })
          .accounts({
            claimant: claimant,
            payer: player.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .instruction();

        const signers = [player];

        const tx = new anchor.web3.Transaction();
        tx.add(instruction);
        await program.provider.sendAndConfirm!(tx, signers);
        console.log("You've won!");
        //console.log(result2);
      }
    } catch (err) {
      console.log("sorry you have lost");
    }
  });
});

type BetType = {
  head?: {};
  tail?: {};
};
