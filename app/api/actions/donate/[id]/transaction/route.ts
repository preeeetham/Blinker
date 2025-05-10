import { NextResponse } from "next/server"
import clientPromise from "@/lib/mongodb"
import { ACTIONS_CORS_HEADERS } from "@solana/actions"
import { PublicKey, SystemProgram, Transaction, LAMPORTS_PER_SOL } from "@solana/web3.js"
import { TREASURY_PUBKEY } from "@/lib/constants"

export async function POST(req: Request, { params }: { params: { id: string } }) {
  try {
    const id = params.id
    const { searchParams } = new URL(req.url)
    const amountParam = searchParams.get("amount")

    const data = await req.json()
    const { account, input } = data

    if (!account) {
      return NextResponse.json({ error: "Missing account" }, { status: 400 })
    }

    // Determine the amount to send
    let amount: number
    if (amountParam) {
      amount = Number.parseFloat(amountParam)
    } else if (input && input.amount) {
      amount = Number.parseFloat(input.amount)
    } else {
      amount = 0.1 // Default amount
    }

    // Convert amount to lamports
    const lamports = Math.floor(amount * LAMPORTS_PER_SOL)

    // Create a transaction
    const transaction = new Transaction()

    // Add a transfer instruction
    transaction.add(
      SystemProgram.transfer({
        fromPubkey: new PublicKey(account),
        toPubkey: TREASURY_PUBKEY,
        lamports,
      }),
    )

    // Serialize the transaction
    const serializedTransaction = transaction
      .serialize({
        requireAllSignatures: false,
        verifySignatures: false,
      })
      .toString("base64")

    // Update the blink in the database to record this transaction attempt
    const client = await clientPromise
    const db = client.db("Cluster0")

    await db.collection("transactions").insertOne({
      blinkId: id,
      sender: account,
      amount,
      createdAt: new Date(),
      status: "pending",
    })

    return NextResponse.json(
      {
        transaction: serializedTransaction,
        message: `Sending ${amount} SOL to the creator`,
      },
      {
        headers: ACTIONS_CORS_HEADERS,
      },
    )
  } catch (error) {
    console.error("Error creating transaction:", error)
    return NextResponse.json(
      { error: "Failed to create transaction" },
      {
        status: 500,
        headers: ACTIONS_CORS_HEADERS,
      },
    )
  }
}
