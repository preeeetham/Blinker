import { NextResponse } from 'next/server';
import clientPromise from '@/lib/mongodb';

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
    const { poolName, Liquidity, Volume, APR, Fee,
            DailyFee, BinStep, TokenXName, TokenYName,
            mintX, mintY, wallet, poolId, percentage } = data;

    if (!poolName || !Liquidity || Volume==null || APR==null || !Fee
        || DailyFee==null || BinStep==null || !TokenXName || !TokenYName
        || !mintX || !mintY || !wallet || !poolId
        || !percentage) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    const client = await clientPromise;
    const db = client.db("Cluster0");

    const result = await db.collection("blinks").insertOne({
      icon : "https://www.getblink.fun/meteora.jpg",
      poolName,
      Liquidity,
      Volume,
      APR,
      Fee,
      DailyFee,
      BinStep,
      TokenXName,
      TokenYName,
      mintX,
      mintY,
      createdAt: new Date(),
      isPaid: false,
      wallet,
      endpoint: "lp",
      poolId,
      percentage,
    });

    console.log(result);

    const blinkLink = `https://www.getblink.fun/api/actions/lp/${result.insertedId}`;
    return NextResponse.json({ blinkLink, id: result.insertedId.toString() });
  } catch (error) {
    console.error('Error generating blink:', error);
    return NextResponse.json({ error: 'Failed to generate blink' }, { status: 500 });
  }
}
