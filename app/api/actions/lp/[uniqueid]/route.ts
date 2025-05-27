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
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import DLMM, { LbPosition, StrategyType } from '@meteora-ag/dlmm';
import { BN } from '@coral-xyz/anchor';
import { getMint } from '../../../../../node_modules/@solana/spl-token';

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

    if(!blinkData) {
      return NextResponse.json(
        {
          message: "This blink does not exist.",
        },
        {
          status: 404,
          headers: ACTIONS_CORS_HEADERS,
        },
      );
    }

    const blink_Description = `****

**Liq:** $${parseFloat(blinkData.Liquidity).toFixed(3)}        **24h Vol:** $${parseFloat(blinkData.Volume).toFixed(3)}

**APR:** ${parseFloat(blinkData.APR).toFixed(3)}%          **Bin Step:** ${blinkData.BinStep}

**Fee:** ${parseFloat(blinkData.Fee).toFixed(3)}%           **24h Fees:** $${parseFloat(blinkData.DailyFee).toFixed(3)}

**Pool Id:** ${blinkData.poolId}

****

**Token Pair**

**${blinkData.TokenXName}:** ${blinkData.mintX}

**${blinkData.TokenYName}:** ${blinkData.mintY}

****

NOTE: 0.0566 SOL needed to create a position (refundable on closing the position)`;

  const connection = new Connection(process.env.SOLANA_RPC || clusterApiUrl("mainnet-beta"));
  const dlmmPool = await DLMM.create(connection, new PublicKey(blinkData.poolId));
  const bins = await dlmmPool.getBinsAroundActiveBin(20, 20);
  const minBindId = bins.activeBin - 20;
  const maxBindId = bins.activeBin + 20;

  const maxBinPrice = dlmmPool.fromPricePerLamport(
    parseFloat(bins.bins.filter((bin) => bin.binId === maxBindId)[0].price)
  );
  const minBinPrice = dlmmPool.fromPricePerLamport(
    parseFloat(bins.bins.filter((bin) => bin.binId === minBindId)[0].price)
  );


    const payload: ActionGetResponse = {
      icon: process.env.METEORA_IMAGE || blinkData.icon,
      label: `Open a ${blinkData.poolName} Position`,
      description: blink_Description,
      title: `Open a ${blinkData.poolName} Position`,
      links: {
        actions: [
          {
            href: `/api/actions/lp/${uniqueid}?amountX={TokenXamount}&amountY={TokenYamount}&starategy={starategy}&MinPrice={MinPrice}&MaxPrice={MaxPrice}`,
            label: `Open a ${blinkData.poolName} Position`,
            type: "post",
            parameters: [
              {
                type: "radio",
                name: "starategy",
                label: "Select a strategy",
                options: [
                  {
                    label: "Spot",
                    value: "Spot",
                    selected: true,
                  },
                  {
                    label: "Curve",
                    value: "Curve",
                  },
                  {
                    label: "Bid Ask",
                    value: "BidAsk",
                  }
                ],
              },
              {
                name: "TokenXamount",
                label: `Enter amount of ${blinkData.TokenXName}`,
                type: "number",
                required: true,
              },
              {
                name: "TokenYamount",
                label: `Enter amount of ${blinkData.TokenYName}`,
                type: "number",
                required: true,
              },
              {
                name: "MinPrice",
                label: `Min price of ${blinkData.TokenXName} in ${blinkData.TokenYName} (default: ${parseFloat(minBinPrice).toFixed(5)} ${blinkData.TokenYName})`,
                type: "number",
              },
              {
                name: "MaxPrice",
                label: `Max price of ${blinkData.TokenXName} in ${blinkData.TokenYName} (default: ${parseFloat(maxBinPrice).toFixed(5)} ${blinkData.TokenYName})`,
                type: "number",
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
    const connection = new Connection(process.env.SOLANA_RPC || clusterApiUrl("mainnet-beta"));
    const poolId = blinkData?.poolId
    if(!poolId) {
      return NextResponse.json({
        message: "This blink does not exist.",
      }, {
        status: 404,
        headers: ACTIONS_CORS_HEADERS,
      })
    }
    const { searchParams } = new URL(req.url);
    const body: ActionPostRequest = await req.json();

    let account: PublicKey;
    try {
      account = new PublicKey(body.account);
    } catch (err) {
      throw "Invalid 'account' provided. It's not a real pubkey";
    }

    const amountXParam = searchParams.get("amountX");
    const amountYParam = searchParams.get("amountY");
    console.log("--------------------------------------------------------------");
    console.log("amountX", amountXParam);
    console.log("amountY", amountYParam);
    console.log("--------------------------------------------------------------");
    if (!amountXParam || !amountYParam) {
        throw "Invalid 'amount' input";
    }
    const amountX = parseFloat(amountXParam);
    const amountY = parseFloat(amountYParam);
    console.log("--------------------------------------------------------------");
    console.log("amountX", amountX);
    console.log("amountY", amountY);
    console.log("--------------------------------------------------------------");
    if (isNaN(amountX) || isNaN(amountY) || amountX <= 0 || amountY <= 0) {
      throw "Invalid 'amount' input";
    }

    const strategy = searchParams.get("starategy");
    if (!strategy) {
      throw "Invalid 'strategy' input";
    }

    const MaxPriceParam = searchParams.get("MaxPrice");
    const MinPriceParam = searchParams.get("MinPrice");
    let minBinId:number;
    let maxBinId:number;

    const dlmmPool = await DLMM.create(connection, new PublicKey(poolId));

    if(!MaxPriceParam || !MinPriceParam) {
      const bins = await dlmmPool.getBinsAroundActiveBin(20, 20);
      minBinId = bins.activeBin - 20;
      maxBinId = bins.activeBin + 20;
    }else{
      const maxBinPrice = dlmmPool.toPricePerLamport(parseFloat(MaxPriceParam));
      const minBinPrice = dlmmPool.toPricePerLamport(parseFloat(MinPriceParam));

      if (isNaN(parseFloat(maxBinPrice)) || isNaN(parseFloat(minBinPrice))) {
        throw "Invalid 'price' input";
      }

      if (parseFloat(maxBinPrice) <= 0 || parseFloat(minBinPrice) <= 0) {
        throw "Invalid 'price' input";
      }

      maxBinId = dlmmPool.getBinIdFromPrice(parseFloat(maxBinPrice), false);
      minBinId = dlmmPool.getBinIdFromPrice(parseFloat(minBinPrice), true);
    }


    if (!dlmmPool) {
      throw "Unable to find pool";
    }

    const activeBin = await dlmmPool.getActiveBin();
    console.log("--------------------------------------------------------------");
    console.log("activeBin", activeBin);
    console.log("activeBin.binId", activeBin.binId);
    console.log("---------------------------------------------------------------");

    console.log("--------------------------------------------------------------");
    console.log("minBinId", minBinId);
    console.log("maxBinId", maxBinId);
    console.log("--------------------------------------------------------------");


    // Calculate amounts based on current price
    const activeBinPricePerToken = dlmmPool.fromPricePerLamport(
      Number(activeBin.price),
    );

    const tokenXDecimal = (await getMint(connection, new PublicKey(dlmmPool.tokenX.publicKey))).decimals;
    const tokenYDecimal = (await getMint(connection, new PublicKey(dlmmPool.tokenY.publicKey))).decimals;

    console.log("--------------------------------------------------------------");
    console.log("tokenXDecimal", tokenXDecimal);
    console.log("tokenYDecimal", tokenYDecimal);
    console.log("--------------------------------------------------------------");


    const quoteResponse = await (
      await fetch(`https://quote-api.jup.ag/v6/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=${dlmmPool.tokenX.mint.address.toString()}&amount=${amountX * (10**tokenXDecimal)}&slippageBps=300&swapMode=ExactOut`
      )
    ).json();
    console.log(quoteResponse);

    const TotalXTradeValue = quoteResponse.inAmount / Math.pow(10, 9);
    const percentage = blinkData?.percentage || 0;

    const tokenXAmountBN = new BN(
      amountX * Math.pow(10, tokenXDecimal),
      );
    const tokenYAmountBN = new BN(
      amountY *
          Math.pow(10, tokenYDecimal),
      );


    const positionKeypair = new Keypair();
    const createPositionTx =
      await dlmmPool.initializePositionAndAddLiquidityByStrategy({
        positionPubKey: positionKeypair.publicKey,
        user: account,
        totalXAmount: tokenXAmountBN,
        totalYAmount: tokenYAmountBN,
        strategy: {
          maxBinId,
          minBinId,
          strategyType: StrategyType[strategy as keyof typeof StrategyType],
        },
      });

    if(percentage > 0) {
      const fee = (TotalXTradeValue * percentage) / 100;
      createPositionTx.add(
        SystemProgram.transfer({
          fromPubkey: account,
          toPubkey: new PublicKey(TREASURY_PUBKEY),
          lamports: Math.floor(fee * LAMPORTS_PER_SOL),
        })
      )
    }

    createPositionTx.feePayer = account;



    const payload: ActionPostResponse = await createPostResponse({
      fields: {
        type: "transaction",
        transaction: createPositionTx,
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
