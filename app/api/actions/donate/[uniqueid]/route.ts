import { NextRequest, NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
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

export const GET = async (req: NextRequest, { params }: { params: { uniqueid: string } }) => {
  try {
    const { uniqueid } = params;
    console.log('uniqueid', uniqueid);

    let blinkData;
    try {
      blinkData = await prisma.blink.findUnique({ 
        where: { id: uniqueid } 
      });
    } catch (error) {
      // Invalid ID format, blinkData will remain undefined
      blinkData = null;
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
        icon: "https://www.vegrecipesofindia.com/wp-content/uploads/2018/02/cafe-style-hot-coffee-recipe-1.jpg",
        label: "Buy me a coffee ☕️",
        description: "👋 If you are interested in helping to support my work, buy me a coffee with SOL using this super sweet blink of mine :)",
        title: "Buy Me a Coffee? ☕️",
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
            href: `/api/actions/donate/${uniqueid}?amount=0.1`,
            label: "0.1 SOL",
            type: "post",
          },
          {
            href: `/api/actions/donate/${uniqueid}?amount=0.5`,
            label: "0.5 SOL",
            type: "post",
          },
          {
            href: `/api/actions/donate/${uniqueid}?amount=1.0`,
            label: "1.0 SOL",
            type: "post",
          },
          {
            href: `/api/actions/donate/${uniqueid}?amount={amount}`,
            label: "Send SOL",
            type: "post",
            parameters: [
              {
                name: "amount",
                label: "Enter a SOL amount",
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

    let blinkData;
    try {
      blinkData = await prisma.blink.findUnique({ 
        where: { id: uniqueid } 
      });
    } catch (error) {
      // Invalid ID format, blinkData will remain undefined
      blinkData = null;
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

    const SOLANA_RPC_URL = clusterApiUrl("mainnet-beta", false);
    if (!SOLANA_RPC_URL) throw "Unable to find RPC url...awkward...";
    const connection = new Connection(SOLANA_RPC_URL);

    const toPubkey = blinkData?.wallet ? new PublicKey(blinkData.wallet) : TREASURY_PUBKEY;
    
    const transaction = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: account,
        lamports: amount * LAMPORTS_PER_SOL,
        toPubkey: toPubkey,
      }),
    );
    transaction.feePayer = account;
    transaction.recentBlockhash = (
      await connection.getLatestBlockhash()
    ).blockhash;

    const payload: ActionPostResponse = await createPostResponse({
      fields: {
        type: "transaction",
        transaction,
        message: "Thanks for the coffee fren :)",
      },
    });

    return NextResponse.json(payload, {
      headers: ACTIONS_CORS_HEADERS,
    });
  } catch (err) {
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