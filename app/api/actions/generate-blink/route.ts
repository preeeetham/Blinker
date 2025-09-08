import { NextResponse } from 'next/server';
import { prisma } from '@/lib/prisma';
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

    const result = await prisma.blink.create({
      data: {
        icon,
        label,
        description,
        title,
        wallet,
        endpoint: "donate",
        isPaid: false
      }
    });

    const messageString = `${wallet + result.id}`;
    const transaction = await  createTransaction(messageString, amounts.donate, wallet);

    console.log(result);

    return NextResponse.json({ transaction, id: result.id });
  } catch (error) {
    console.error('Error generating blink:', error);
    return NextResponse.json({ error: 'Failed to generate blink' }, { status: 500 });
  }
}