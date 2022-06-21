import { Provider } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import * as anchor from "@project-serum/anchor";

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
