import { Program, Provider } from "@project-serum/anchor";
import * as anchor from "@project-serum/anchor";
import { CoinFlip } from "../target/types/coin_flip";
import { PublicKey, LAMPORTS_PER_SOL, SystemProgram } from "@solana/web3.js";

export const ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID = new anchor.web3.PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);
export const TOKEN_PROGRAM_ID = new anchor.web3.PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

export const findAssociatedAddressForKey = async (
  tokenRecipient: PublicKey,
  mintKey: PublicKey,
  tokenProgramID: PublicKey = new PublicKey(TOKEN_PROGRAM_ID),
  associatedProgramID: PublicKey = new PublicKey(
    ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID
  )
): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [tokenRecipient.toBuffer(), tokenProgramID.toBuffer(), mintKey.toBuffer()],
    associatedProgramID
  );
};

export const addSols = async (
  provider: Provider,
  wallet: PublicKey,
  amount = 2 * anchor.web3.LAMPORTS_PER_SOL
) => {
  await provider.connection.confirmTransaction(
    await provider.connection.requestAirdrop(wallet, amount),
    "confirmed"
  );
};

export const getCoinFlipAddress = async (program: Program<any>) => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("coin-flip")],
    program.programId
  );
};

export const getClaimantAddress = async (
  program: Program<any>,
  payer: PublicKey
) => {
  return await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("claimant"), payer.toBuffer()],
    program.programId
  );
};

export async function claim(
  program: anchor.Program<CoinFlip>,
  claimant: anchor.web3.PublicKey,
  player: anchor.web3.Keypair
) {
  const result = await program.account.claimant.fetch(claimant);

  if (result.success) {
    console.log("You've won!");
    //console.log(result2);
  } else {
    console.log("SORRY you have lost");
  }
  await callClaim(program, claimant, player);
}

type BetType = {
  head?: {};
  tail?: {};
};
async function callClaim(
  program: anchor.Program<CoinFlip>,
  claimant: anchor.web3.PublicKey,
  player: anchor.web3.Keypair
) {
  const instruction = await program.methods
    .claim({})
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
}

export const getBetInstruction = async (
  program: Program<any>,
  player: any,
  bet: BetType
) => {
  const [coinFlip, _1] = await getCoinFlipAddress(program);

  const [claimant, _2] = await getClaimantAddress(program, player.publicKey);

  const amount = new anchor.BN(0.05 * LAMPORTS_PER_SOL);

  //const value: BetType = ;

  return await program.methods
    .bet({
      amount: amount,
      betType: bet,
    })
    .accounts({
      coinFlip: coinFlip,
      claimant: claimant,
      payer: player.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .instruction();
};

export const timeout = (ms) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};
