import { NextResponse } from 'next/server';
import clientPromise from '@/lib/mongodb';
export async function GET(req: Request) {
  try {
    const { searchParams } = new URL(req.url);
    const wallet = searchParams.get('wallet');

    if (!wallet) {
      return NextResponse.json({ error: 'Missing wallet address in query params' }, { status: 400 });
    }

    const client = await clientPromise;
    const db = client.db("Cluster0");

    const blinks = await db.collection("blinks").find({
      wallet,
      $or: [
        { isPaid: true },
        { isPaid: { $exists: false } }
      ]
    }).toArray();

    if (!blinks.length) {
      return NextResponse.json({ blinks: [] });
    }

    return NextResponse.json({ blinks });
  } catch (error) {
    console.error('Error fetching blinks:', error);
    return NextResponse.json({ error: 'Failed to fetch blinks' }, { status: 500 });
  }
}
