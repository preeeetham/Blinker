import { NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
import { Connection, PublicKey, TransactionMessage, clusterApiUrl, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { amounts, BlinkType } from '@/lib/constant';

export async function POST(req: Request) {
  try {
    // Log the raw request body
    const rawBody = await req.text();
    console.log('Raw request body:', rawBody);

    // Try to parse the JSON
    let data;
    try {
      data = JSON.parse(rawBody);
    } catch (parseError) {
      console.error('Error parsing JSON:', parseError);
      return NextResponse.json({ error: 'Invalid JSON in request body' }, { status: 400 });
    }

    // Destructure and validate the parsed data
    const { signature, orderId } = data;
    if (!signature || !orderId) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    const MEMO_PROGRAM_ID = new PublicKey("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

    const connection = new Connection(process.env.SOLANA_RPC || clusterApiUrl("mainnet-beta"));

    const tx = await connection.getTransaction(signature, {
      commitment: 'finalized'
    });

    if (!tx || !tx.transaction || !tx.transaction.message) {
      throw new Error("Transaction not found or not yet confirmed.");
    }

    const msg = TransactionMessage.decompile(tx.transaction.message);
    const inx = msg.instructions.filter(
      (inx) => inx.programId.equals(MEMO_PROGRAM_ID)
    )[0];
    if(inx === undefined) {
      throw new Error('No memo instruction found');
    }
    if (!inx.programId.equals(MEMO_PROGRAM_ID)) {
      throw new Error('Transaction not using memo program');
    }

    const order = await prisma.blink.findUnique({ 
      where: { id: orderId } 
    });
    if(!order){
      throw new Error('Order not found');
    }

    const nonce = order.wallet + order.id;
    if (inx.data.toString() !== nonce) {
      console.log('Transaction memo data does not match expected nonce', inx.data.toString(), nonce);
      return NextResponse.json({ error: 'Transaction memo data does not match expected nonce' }, { status: 400 });
    }
    const SYSTEM_PROGRAM_ID = new PublicKey('11111111111111111111111111111111');
    const TREASURY_WALLET = new PublicKey(process.env.WALLET || 'HMdzGauLv7s8LQbuEwJvqFEM3Za1VRC2h6Jdq7nVT7YX');

    const EXPECTED_AMOUNT = amounts[order.endpoint as BlinkType] * LAMPORTS_PER_SOL;

    const transferInx = msg.instructions.find(
      (ix) =>
        ix.programId.equals(SYSTEM_PROGRAM_ID)
    );

    if(transferInx === undefined) {
      throw new Error('No treasury payment instruction found');
    }

    const amount = transferInx.data.slice(4).readBigUInt64LE(0);
    const filterKeys = transferInx.keys.filter((key) => key.pubkey.equals(TREASURY_WALLET))[0];
    if(!filterKeys) {
      throw new Error('Treasury wallet not found in transfer instruction');
    }
    if (!filterKeys.isWritable) {
      throw new Error('Treasury wallet is not writable in transfer instruction');
    }


    if(amount !== BigInt(EXPECTED_AMOUNT)) {
      console.log('Payment amount does not match expected amount', amount.toString(), EXPECTED_AMOUNT.toString());
      return NextResponse.json({ error: 'Payment amount does not match expected amount' }, { status: 400 });
    }

    const result = await prisma.blink.update({
      where: { id: orderId },
      data: { isPaid: true }
    });

    const blinkLink = `https://www.blinkgen.xyz/api/actions/${order.endpoint}/${orderId}`;
    return NextResponse.json({ blinkLink });
  } catch (error) {
    console.error('Error generating blink:', error);
    return NextResponse.json({ error: 'Failed to generate blink' }, { status: 500 });
  }}