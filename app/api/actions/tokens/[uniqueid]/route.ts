import { ObjectId } from 'mongodb';
import { NextRequest, NextResponse } from 'next/server';
import clientPromise from '@/lib/mongodb';
import {
  ActionGetResponse,
  ACTIONS_CORS_HEADERS,
  ActionPostRequest,
  createPostResponse,
  ActionPostResponse,
} from "@solana/actions";
import {
  AddressLookupTableAccount,
  clusterApiUrl,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { getAssociatedTokenAddress, createAssociatedTokenAccountInstruction } from "@/lib/constant";

const PUMP_PROGRAM_ID = new PublicKey("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

export const GET = async (req: NextRequest, { params }: { params: { uniqueid: string } }) => {
  try {
    const { uniqueid } = params;
    console.log('uniqueid', uniqueid);

    const client = await clientPromise;
    const db = client.db("Cluster0");

    let blinkData;
    if (ObjectId.isValid(uniqueid)) {
      blinkData = await db.collection("blinks").findOne({ _id: new ObjectId(uniqueid) });
    }


    if(blinkData && blinkData.isPaid === false){
      return NextResponse.json(
        {
          message: "This blink is not paid for yet. Please pay to use it.",
        },
        {
          status: 403,
          headers: ACTIONS_CORS_HEADERS,
        },
      );
    }

    if (!blinkData) {
      blinkData = {
        icon: "https://example.com/pump-token-icon.png",
        label: "Buy Pump Token 🚀",
        description: "Get your Pump tokens now! Choose an amount to purchase.",
        title: "Buy Pump Tokens",
      };
    }

    const payload: ActionGetResponse = {
      icon: blinkData.icon,
      label: blinkData.label,
      description: blinkData.description,
      title: blinkData.title,
      links: {
        actions: [
          {
            href: `/api/actions/tokens/${uniqueid}?amount=100000`,
            label: `Buy 100k ${blinkData.title.slice(4)}`,
            type: "post",
          },
          {
            href: `/api/actions/tokens/${uniqueid}?amount=500000`,
            label: `Buy 500k ${blinkData.title.slice(4)}`,
            type: "post",
          },
          {
            href: `/api/actions/tokens/${uniqueid}?amount=1000000`,
            label: `Buy 1M ${blinkData.title.slice(4)}`,
            type: "post",
          },
          {
            href: `/api/actions/tokens/${uniqueid}?amount={amount}`,
            label: "Custom amount",
            type: "post",
            parameters: [
              {
                name: "amount",
                label: "Enter amount",
              },
            ],
          },
        ],
      },
    };

    return NextResponse.json(payload, {
      headers: ACTIONS_CORS_HEADERS,
    });
  } catch (error) {
    console.error("Error fetching blink data:", error);
    return NextResponse.json({ error: "Internal Server Error" }, { status: 500 });
  }
};

export const OPTIONS = GET;

export const POST = async (req: NextRequest, { params }: { params: { uniqueid: string } }) => {
  try {
    const { uniqueid } = params;

    const client = await clientPromise;
    const db = client.db("Cluster0");

    let blinkData;
    if (ObjectId.isValid(uniqueid)) {
      blinkData = await db.collection("blinks").findOne({ _id: new ObjectId(uniqueid) });
    }
    if(!blinkData){
      throw "Invalid BLINK!!";
    }

    if(blinkData && blinkData.isPaid === false){
      return NextResponse.json(
        {
          message: "This blink is not paid for yet. Please pay to use it.",
        },
        {
          status: 403,
          headers: ACTIONS_CORS_HEADERS,
        },
      );
    }

    const { searchParams } = new URL(req.url);
    const body: ActionPostRequest = await req.json();

    const PUMP_MINT = new PublicKey(blinkData?.mint);

    let account: PublicKey;
    try {
      account = new PublicKey(body.account);
    } catch (err) {
      throw "Invalid 'account' provided. It's not a real pubkey";
    }

    let amount: number = 0.1;
    const amountParam = searchParams.get("amount");
    if (amountParam) {
      try {
        amount = parseFloat(amountParam) || amount;
      } catch (err) {
        throw "Invalid 'amount' input";
      }
    }

    const connection = new Connection(process.env.SOLANA_RPC|| clusterApiUrl("mainnet-beta"));

    const recentBlockhash = await connection.getLatestBlockhash();

    const [global] = await PublicKey.findProgramAddress(
      [Buffer.from("global")],
      PUMP_PROGRAM_ID
    );

    const [bondingCurve] = await PublicKey.findProgramAddress(
      [Buffer.from("bonding-curve"), PUMP_MINT.toBuffer()],
      PUMP_PROGRAM_ID
    );

    const bondingCurveATA = await getAssociatedTokenAddress(
      PUMP_MINT,
      bondingCurve
    );

    const userATA = await getAssociatedTokenAddress(
      PUMP_MINT,
      account
    );

    const transaction = new Transaction();

    // Add instruction to create ATA if it doesn't exist
    transaction.add(
      createAssociatedTokenAccountInstruction(
        account,
        userATA,
        account,
        PUMP_MINT
      )
    );

    const dataBuffer = Buffer.alloc(24);
    dataBuffer.write("66063d1201daebea", "hex");
    dataBuffer.writeBigUInt64LE(BigInt(amount *(10**6)), 8);
    dataBuffer.writeBigInt64LE(BigInt(-1), 16);
    const data = Buffer.from(dataBuffer);

    let feeReceiver: PublicKey | undefined;
    let feeAmount=BigInt(0);

    // Create the transaction instruction for buying Pump tokens
    const buyPumpIx = new TransactionInstruction({
      programId: PUMP_PROGRAM_ID,
      keys: [
        { pubkey: global, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM"), isSigner: false, isWritable: true },
        { pubkey: PUMP_MINT, isSigner: false, isWritable: false },
        { pubkey: bondingCurve, isSigner: false, isWritable: true },
        { pubkey: bondingCurveATA, isSigner: false, isWritable: true },
        { pubkey: userATA, isSigner: false, isWritable: true },
        { pubkey: account, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        { pubkey: new PublicKey("SysvarRent111111111111111111111111111111111"), isSigner: false, isWritable: false },
        { pubkey: new PublicKey("Ce6TQqeHC9p8KetsN6JsjHK7UTZk7nasjjnr7XxXp9F1"), isSigner: false, isWritable: false },
        { pubkey: PUMP_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      data: data,
    });

    transaction.add(buyPumpIx);
    transaction.recentBlockhash = recentBlockhash.blockhash;
    transaction.feePayer = account;
    let sol_spent = await estimateTotalTransactionCost(connection, transaction, account);
    if(!sol_spent){
      sol_spent = 0;
    }
    let updatedTransaction:any = transaction;
    if(sol_spent === "Error"){
      updatedTransaction = await tradeJUP(blinkData, amount, account, connection, blinkData.decimals || 9);
    }else if(typeof(sol_spent) === "number"){
      if (blinkData?.commission && blinkData?.commission === "yes" && blinkData?.percentage && blinkData.percentage > 0) {
        feeReceiver = new PublicKey(blinkData?.wallet);
        const commission = blinkData.percentage;
        const fee = (commission * sol_spent) / 100;
        console.log("Fee: ", fee+" SOL"+" for "+commission+"% of "+sol_spent+" SOL");
        // Convert to integer lamports (assuming feeAmount is in SOL)
        feeAmount = BigInt(Math.round(fee * 1_000_000_000)); // Convert SOL to lamports and round to an integer
      } else {
        feeAmount = BigInt(0); // Set to BigInt for consistency
      }

      // Add an additional instruction to transfer the fee (if applicable)
      if (feeReceiver && feeAmount > BigInt(0)) {
        const transferFeeIx = SystemProgram.transfer({
          fromPubkey: account,
          toPubkey: feeReceiver,
          lamports: Number(feeAmount), // `Number` is used to pass the value correctly
        });
        transaction.add(transferFeeIx); // Add fee transfer instruction
      }
    }

    const payload: ActionPostResponse = await createPostResponse({
      fields: {
        type: "transaction",
        transaction: updatedTransaction,
        message: "You just Pumped It!",
      },
    });

    return NextResponse.json(payload, {
      headers: ACTIONS_CORS_HEADERS,
    });
  } catch (err) {
    console.log("-------------------------------ErrorInBuyPump----------------------------------");
    console.error(err);
    return NextResponse.json(
      {
        message: err instanceof Error ? err.message : String(err),
      },
      {
        status: 500,
        headers: ACTIONS_CORS_HEADERS,
      },
    );
  }
};


async function estimateTotalTransactionCost(connection: Connection, transaction: Transaction, wallet: PublicKey) {
  const balance = (await connection.getBalance(wallet));
  console.log("Current balance: ", balance/LAMPORTS_PER_SOL,"sol");
  const simulationResult = await connection.simulateTransaction(
    transaction,
    undefined,
    [wallet]
  );

  if (simulationResult.value.err) {
    console.error("Error in transaction simulation:", simulationResult.value.err);
    if(simulationResult.value.err === "BlockhashNotFound"){
      return;
    }
    return "Error";
  }else{
    const lamportsLeft = simulationResult.value.accounts?.[0]?.lamports;
    if(lamportsLeft === undefined){
      return 0;
    }
    console.log("Lamports Left: ", lamportsLeft/LAMPORTS_PER_SOL,"sol");
    return (balance - lamportsLeft)/LAMPORTS_PER_SOL;
  }
}

const tradeJUP = async(
  blinkData: any,
  amount: number,
  account: PublicKey,
  connection: Connection,
  decimals: number,
)  =>{
  console.log("MINT IS :" + blinkData.mint);
  let quoteResponse = await (
    await fetch(`https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=${blinkData.mint}&amount=${amount * 10**decimals}&slippageBps=300&swapMode=ExactOut`
    )
  ).json();
  console.log(quoteResponse);

  if(quoteResponse.errorCode === "COULD_NOT_FIND_ANY_ROUTE"){
    quoteResponse = await (
      await fetch(`https://lite-api.jup.ag/swap/v1/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=${blinkData.mint}&amount=${amount}&swapMode=ExactIn`
      )
    ).json();
    const price = quoteResponse.inAmount / quoteResponse.outAmount;
    const solSpent = (parseFloat((price * amount).toFixed(9)) * LAMPORTS_PER_SOL);
    quoteResponse = await (
      await fetch(`https://lite-api.jup.ag/swap/v1/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=${blinkData.mint}&amount=${solSpent.toString()}&swapMode=ExactIn`
      )
    ).json();
  }

  const { swapTransaction } = await (
    await fetch('https://quote-api.jup.ag/v6/swap', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        // quoteResponse from /quote api
        quoteResponse,
        // user public key to be used for the swap
        userPublicKey: account.toString(),
        // auto wrap and unwrap SOL. default is true
        wrapAndUnwrapSol: true,
        // feeAccount is optional. Use if you want to charge a fee.  feeBps must have been passed in /quote API.
        // feeAccount: "fee_account_public_key"
      })
    })
  ).json();

  console.log({ swapTransaction });

  const swapTransactionBuf = Buffer.from(swapTransaction, 'base64');
    const transaction = VersionedTransaction.deserialize(swapTransactionBuf);
    let updatedTransaction = transaction;
    // Check if fee needs to be added

    let feeReceiver: PublicKey | undefined;
    let feeAmount=BigInt(0);

    const sol_spent = quoteResponse.inAmount;

    if (blinkData?.commission && blinkData?.commission === "yes" && blinkData?.percentage && blinkData.percentage > 0) {
      feeReceiver = new PublicKey(blinkData?.wallet);
      const commission = blinkData.percentage;
      const fee = (commission * sol_spent) / 100;
      console.log("Fee: ", fee+" SOL"+" for "+commission+"% of "+sol_spent+" SOL");
      // Convert to integer lamports (assuming feeAmount is in SOL)
      feeAmount = BigInt(Math.round(fee)); // Convert SOL to lamports and round to an integer
    } else {
      feeAmount = BigInt(0); // Set to BigInt for consistency
    }

    if (feeReceiver && feeAmount > BigInt(0)) {
      const transferFeeIx = SystemProgram.transfer({
        fromPubkey: account,
        toPubkey: feeReceiver,
        lamports: Number(feeAmount), // Convert BigInt to number safely if it's within range
      });

      const lookupTablePubkeys = transaction.message.addressTableLookups.map(lookup => lookup.accountKey);

      // Fetch all address lookup tables
      const lookupTablesResponses = await Promise.all(
          lookupTablePubkeys.map(pubkey => connection.getAddressLookupTable(pubkey))
      );

      // Filter out null results and extract the AddressLookupTableAccount
      const lookupTables = lookupTablesResponses
          .map(response => response.value) // Extract the `value` from the response
          .filter(table => table !== null); // Remove nulls

      // Resolve the address lookups
      const resolvedMessage = TransactionMessage.decompile(transaction.message, {
        addressLookupTableAccounts: lookupTables.filter((table): table is AddressLookupTableAccount => table !== null),
      });

      console.log(resolvedMessage);

      resolvedMessage.instructions.push(transferFeeIx);
      // Recreate the message with updated instructions
      const updatedMessage = resolvedMessage.compileToLegacyMessage();

      // Recreate the VersionedTransaction
      updatedTransaction = new VersionedTransaction(updatedMessage);
    }
    return updatedTransaction;
};
