import { NextResponse } from "next/server"
import clientPromise from "@/lib/mongodb"
import { ObjectId } from "mongodb"

export async function POST(req: Request) {
  try {
    const data = await req.json()
    const { signature, orderId } = data

    if (!signature || !orderId) {
      return NextResponse.json({ error: "Missing signature or orderId" }, { status: 400 })
    }

    const client = await clientPromise
    const db = client.db("Cluster0")

    // Update the blink to mark it as paid
    const result = await db
      .collection("blinks")
      .updateOne({ _id: new ObjectId(orderId) }, { $set: { isPaid: true, signature, updatedAt: new Date() } })

    if (result.matchedCount === 0) {
      return NextResponse.json({ error: "Blink not found" }, { status: 404 })
    }

    // Return the blink link
    return NextResponse.json({ blinkLink: orderId })
  } catch (error) {
    console.error("Error processing order:", error)
    return NextResponse.json({ error: "Failed to process order" }, { status: 500 })
  }
}
