import { ObjectId } from 'mongodb';
import { NextRequest, NextResponse } from 'next/server';
import clientPromise from '@/lib/mongodb';
import { TREASURY_PUBKEY } from "@/lib/constant";
import {
  ActionGetResponse,
  ACTIONS_CORS_HEADERS,
  ActionPostRequest,
  createPostResponse,
  ActionPostResponse,
} from "@solana/actions";
import {
  clusterApiUrl,
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";

// GET endpoint
export const GET = async (req: NextRequest, { params }: { params: { uniqueid: string } }) => {
  try {
    const { uniqueid } = params;

    const client = await clientPromise;
    const db = client.db("YOUR_DB_NAME");

    let data;
    if (ObjectId.isValid(uniqueid)) {
      data = await db.collection("YOUR_COLLECTION_NAME").findOne({ _id: new ObjectId(uniqueid) });
    }

    if (data && data.isPaid === false) {
      return NextResponse.json(
        { message: "This resource is not paid for yet." },
        { status: 403, headers: ACTIONS_CORS_HEADERS },
      );
    }

    if (!data) {
      data = {
        icon: "https://example.com/icon.jpg",
        label: "Default Label",
        description: "Default description text",
        title: "Default Title",
      };
    }

    const payload: ActionGetResponse = {
      icon: data.icon,
      label: data.label,
      description: data.description,
      title: data.title,
      links: {
        actions: [
          {
            href: `/api/actions/pay/${uniqueid}?amount=0.1`,
            label: "0.1 SOL",
            type: "post",
          },
          {
            href: `/api/actions/pay/${uniqueid}?amount=0.5`,
            label: "0.5 SOL",
            type: "post",
          },
          {
            href: `/api/actions/pay/${uniqueid}?amount=1.0`,
            label: "1.0 SOL",
            type: "post",
          },
          {
            href: `/api/actions/pay/${uniqueid}?amount={amount}`,
            label: "Custom SOL",
            type: "post",
            parameters: [{ name: "amount", label: "Enter SOL amount" }],
          },
        ],
      },
    };

    return NextResponse.json(payload, { headers: ACTIONS_CORS_HEADERS });
  } catch (error) {
    console.error("GET Error:", error);
    return NextResponse.json({ error: "Internal Server Error" }, { status: 500 });
  }
};

export const OPTIONS = GET;

// POST endpoint
export const POST = async (req: NextRequest, { params }: { params: { uniqueid: string } }) => {
  try {
    const { uniqueid } = params;

    const client = await clientPromise;
    const db = client.db("YOUR_DB_NAME");

    let data;
    if (ObjectId.isValid(uniqueid)) {
      data = await db.collection("YOUR_COLLECTION_NAME").findOne({ _id: new ObjectId(uniqueid) });
    }

    if (data && data.isPaid === false) {
      return NextResponse.json(
        { message: "This resource is not paid for yet." },
        { status: 403, headers: ACTIONS_CORS_HEADERS },
      );
    }

    const { searchParams } = new URL(req.url);
    const body: ActionPostRequest = await req.json();

    let account: PublicKey;
    try {
      account = new PublicKey(body.account);
    } catch {
      throw "Invalid public key.";
    }

    let amount = 0.1;
    const amountParam = searchParams.get("amount");
    if (amountParam) {
      amount = parseFloat(amountParam) || amount;
    }

    const RPC_URL = clusterApiUrl("mainnet-beta");
    const connection = new Connection(RPC_URL);

    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: account,
        lamports: amount * LAMPORTS_PER_SOL,
        toPubkey: data ? (data.wallet || TREASURY_PUBKEY) : TREASURY_PUBKEY,
      }),
    );
    transaction.feePayer = account;
    transaction.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;

    const payload: ActionPostResponse = await createPostResponse({
      fields: {
        type: "transaction",
        transaction,
        message: "Thanks for your support!",
      },
    });

    return NextResponse.json(payload, { headers: ACTIONS_CORS_HEADERS });
  } catch (err) {
    console.error("POST Error:", err);
    return NextResponse.json(
      { message: err instanceof Error ? err.message : String(err) },
      { status: 500, headers: ACTIONS_CORS_HEADERS },
    );
  }
};
