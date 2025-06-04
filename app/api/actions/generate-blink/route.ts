import { NextResponse } from 'next/server';
import clientPromise from '@/lib/mongodb';
import { createTransaction } from '@/server/transaction';
import { amounts } from '@/lib/constant';

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
    const { icon, label, description, title, wallet } = data;
    if (!icon || !label || !description || !title || !wallet) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 });
    }

    const client = await clientPromise;
    const db = client.db("Cluster0");

    const result = await db.collection("blinks").insertOne({
      icon,
      label,
      description,
      title,
      wallet,
      endpoint: "donate",
      createdAt: new Date(),
      isPaid: false
    });

    const messageString = `${wallet + result.insertedId.toString()}`;
    const transaction = await  createTransaction(messageString, amounts.donate, wallet);

    console.log(result);

    return NextResponse.json({ transaction, id: result.insertedId.toString() });
  } catch (error) {
    console.error('Error generating blink:', error);
    return NextResponse.json({ error: 'Failed to generate blink' }, { status: 500 });
  }
}
