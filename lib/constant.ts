import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { PublicKey, SystemProgram, TransactionInstruction } from "@solana/web3.js";

export const TREASURY_PUBKEY = new PublicKey("8twrkXxvDzuUezvbkgg3LxpTEZ59KiFx2VxPFDkucLk3");

export async function getAssociatedTokenAddress(
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  return (
    await PublicKey.findProgramAddress(
      [
        owner.toBuffer(),
        TOKEN_PROGRAM_ID.toBuffer(),
        mint.toBuffer(),
      ],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  )[0];
}

export function createAssociatedTokenAccountInstruction(
  payer: PublicKey,        // The payer of the transaction (usually the user)
  associatedTokenAddress: PublicKey,  // The address of the associated token account
  owner: PublicKey,        // The owner of the associated token account
  mint: PublicKey          // The token mint address
): TransactionInstruction {
  return new TransactionInstruction({
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: ASSOCIATED_TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    programId: ASSOCIATED_TOKEN_PROGRAM_ID,
    data: Buffer.alloc(0),  // The instruction data (empty in this case)
  });
}

export enum BlinkType {
  lp = "lp",
  tokens = "tokens",
  donate = "donate"
}

export enum amounts {
  lp =  parseFloat(process.env.LP_PRICE|| "0.01"),
  tokens = parseFloat(process.env.TOKEN_PRICE || "0.01"),
  donate = parseFloat(process.env.DONATE_PRICE || "0.001")
}
