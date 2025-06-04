"use server";
import { clusterApiUrl, Connection, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";

export const createTransaction = async (messageString: string, rawAmount: number, UserPublicKey: string) => {
  const connection = new Connection(process.env.SOLANA_RPC|| clusterApiUrl("mainnet-beta"));
  const recipientPubKey = new PublicKey(process.env.WALLET || "8twrkXxvDzuUezvbkgg3LxpTEZ59KiFx2VxPFDkucLk3");
  const MEMO_PROGRAM_ID = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");
  const amount = rawAmount * LAMPORTS_PER_SOL;

  const transaction = new Transaction().add(
    new TransactionInstruction({
      programId: MEMO_PROGRAM_ID,
      keys: [],
      data: Buffer.from(messageString, "utf8"),
    })
  );
  transaction.add(
    SystemProgram.transfer({
      fromPubkey: new PublicKey(UserPublicKey),
      toPubkey: recipientPubKey,
      lamports: amount,
    })
  );

  const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
  transaction.recentBlockhash = blockhash;
  transaction.feePayer = new PublicKey(UserPublicKey);

  const serializedTransaction =  Buffer.from(transaction.serialize({
    requireAllSignatures: false,
    verifySignatures: false
  })).toString('base64');

  return {serializedTransaction, blockhash, lastValidBlockHeight};
}

export const confirmTransaction = async (signature: string, blockhash: string, lastValidBlockHeight: number) => {
  try{
    const connection = new Connection(process.env.SOLANA_RPC || clusterApiUrl("mainnet-beta"));
    const confirmation = await connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature,
    }, 'finalized');

    if (confirmation.value.err) {
      throw new Error('Transaction confirmation failed');
    }

    return true;
  }catch(e){
    console.error("Error confirming transaction:", e);
    return false;
  }
}

