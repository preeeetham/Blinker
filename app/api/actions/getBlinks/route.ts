import { NextResponse } from 'next/server'
import { prisma } from '@/lib/prisma'

export async function GET(req: Request) {
  try {
    const { searchParams } = new URL(req.url)
    const wallet = searchParams.get('wallet')

    if (!wallet) {
      return NextResponse.json({ error: 'Missing wallet address' }, { status: 400 })
    }

    const blinks = await prisma.blink.findMany({ where: { wallet } })
    return NextResponse.json({ blinks })
  } catch (error) {
    console.error('getBlinks error:', error)
    return NextResponse.json({ error: 'Failed to fetch blinks' }, { status: 500 })
  }
}