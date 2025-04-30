import { NextResponse } from 'next/server'
import { prisma } from '@/core/prisma'
import { createTransaction } from '@/backend/transaction'
import { amounts } from '@/core/constant'

export async function POST(req: Request) {
  try {
    // Parse JSON body in a standard way
    const { icon, label, description, title, wallet } = await req.json()

    if (!icon || !label || !description || !title || !wallet) {
      return NextResponse.json({ error: 'Missing required fields' }, { status: 400 })
    }

    // Persist blink draft
    const created = await prisma.blink.create({
      data: {
        icon,
        label,
        description,
        title,
        wallet,
        endpoint: 'donate',
        isPaid: false,
      },
      select: { id: true },
    })

    // Memo message ensures order authenticity
    const memoMessage = `${wallet}${created.id}`
    const transaction = await createTransaction(memoMessage, amounts.donate, wallet)

    return NextResponse.json({ transaction, id: created.id })
  } catch (error) {
    console.error('generate-blink error:', error)
    return NextResponse.json({ error: 'Failed to generate blink' }, { status: 500 })
  }
}