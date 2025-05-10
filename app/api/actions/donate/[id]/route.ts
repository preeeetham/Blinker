import { NextResponse } from "next/server"
import clientPromise from "@/lib/mongodb"
import { ObjectId } from "mongodb"
import { ACTIONS_CORS_HEADERS } from "@solana/actions"

export async function GET(req: Request, { params }: { params: { id: string } }) {
  try {
    const id = params.id

    if (!id) {
      return NextResponse.json({ error: "Missing blink ID" }, { status: 400 })
    }

    const client = await clientPromise
    const db = client.db("Cluster0")

    // Find the blink by ID
    const blink = await db.collection("blinks").findOne({
      _id: new ObjectId(id),
    })

    if (!blink) {
      return NextResponse.json({ error: "Blink not found" }, { status: 404 })
    }

    // Format the response according to Solana Actions requirements
    const response = {
      title: blink.title,
      description: blink.description,
      icon: blink.icon,
      buttons: [
        {
          label: "0.05 SOL",
          action: {
            type: "transaction",
            target: `${req.url}/transaction?amount=0.05`,
          },
        },
        {
          label: "1.00 SOL",
          action: {
            type: "transaction",
            target: `${req.url}/transaction?amount=1.0`,
          },
        },
        {
          label: "5.00 SOL",
          action: {
            type: "transaction",
            target: `${req.url}/transaction?amount=5.0`,
          },
        },
      ],
      inputs: [
        {
          id: "amount",
          label: "Custom Amount",
          type: "number",
          placeholder: "Enter SOL amount",
        },
      ],
      actions: [
        {
          label: "Send SOL",
          action: {
            type: "transaction",
            target: `${req.url}/transaction`,
          },
        },
      ],
    }

    return NextResponse.json(response, {
      headers: ACTIONS_CORS_HEADERS,
    })
  } catch (error) {
    console.error("Error fetching blink:", error)
    return NextResponse.json({ error: "Failed to fetch blink" }, { status: 500 })
  }
}
