import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { CoinFlip } from "../target/types/coin_flip";
import { LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";
import {
  addSols,
  claim,
  getBetInstruction,
  getClaimantAddress,
  getCoinFlipAddress,
  timeout,
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
    try {
      const signers = [player];
      const instruction = await getBetInstruction(program, player, {
        head: {},
      });
      const tx = new anchor.web3.Transaction();
      tx.add(instruction);
      await program.provider.sendAndConfirm!(tx, signers);

      const [claimant, _2] = await getClaimantAddress(
        program,
        player.publicKey
      );

      await claim(program, claimant, player);

      const instruction1 = await getBetInstruction(program, player, {
        tail: {},
      });
      const tx1 = new anchor.web3.Transaction();
      tx1.add(instruction1);
      const signers1 = [player];
      await program.provider.sendAndConfirm!(tx, signers1);

      const [claimant1, _3] = await getClaimantAddress(
        program,
        player.publicKey
      );
      await claim(program, claimant1, player);
    } catch (err) {
      console.log(err);
    }
  });

  it("Bet tail", async () => {
    await timeout(100);
    try {
      const instruction = await getBetInstruction(program, player, {
        tail: {},
      });
      const tx = new anchor.web3.Transaction();
      tx.add(instruction);
      const signers = [player];
      await program.provider.sendAndConfirm!(tx, signers);

      const [claimant, _2] = await getClaimantAddress(
        program,
        player.publicKey
      );
      await claim(program, claimant, player);

      // const playerBal = await program.provider.connection.getBalance(
      //   player.publicKey
      // );
      //const playerBalance = +playerBal.toString() / LAMPORTS_PER_SOL;
      //console.log(playerBalance);

      // const treasuryBal = await program.provider.connection.getBalance(
      //   treasury.publicKey
      // );
      //const treasuryBalance = +treasuryBal.toString() / LAMPORTS_PER_SOL;
      //console.log(treasuryBalance);
    } catch (err) {
      console.log(err);
    }
  });
});
