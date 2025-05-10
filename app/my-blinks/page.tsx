"use client"

import { useEffect, useState } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import { WalletButton } from "@/components/solana/solana-provider"
import Link from "next/link"
import Image from "next/image"
import { HiOutlineClipboardCopy, HiOutlineShare } from "react-icons/hi"
import { ArrowRight } from "lucide-react"

interface Blink {
  _id: string
  icon: string
  title: string
  description: string
  createdAt: string
}

export default function MyBlinksPage() {
  const { publicKey, connected } = useWallet()
  const [blinks, setBlinks] = useState<Blink[]>([])
  const [loading, setLoading] = useState(false)
  const [copied, setCopied] = useState<string | null>(null)

  useEffect(() => {
    if (connected && publicKey) {
      fetchBlinks()
    }
  }, [connected, publicKey])

  const fetchBlinks = async () => {
    if (!publicKey) return

    setLoading(true)
    try {
      const response = await fetch(`/api/actions/my-blinks?wallet=${publicKey.toString()}`)
      const data = await response.json()

      if (data.blinks) {
        setBlinks(data.blinks)
      }
    } catch (error) {
      console.error("Error fetching blinks:", error)
    } finally {
      setLoading(false)
    }
  }

  const handleCopy = (id: string) => {
    navigator.clipboard.writeText(`https://dial.to/?action=solana-action:${id}`)
    setCopied(id)
    setTimeout(() => setCopied(null), 1500)
  }

  const handleTweet = (id: string) => {
    const tweetText = `Check out this Blink I made @getblinkdotfun: https://dial.to/?action=solana-action:${id}`
    const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`
    window.open(twitterUrl, "_blank")
  }

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <h1 className="text-3xl font-bold mb-6 gradient-text">My Blinks</h1>
        <div className="card max-w-md mx-auto p-8">
          <p className="text-gray-300 mb-6">Connect your wallet to view your Blinks</p>
          <WalletButton />
        </div>
      </div>
    )
  }

  return (
    <div className="container mx-auto py-8">
      <div className="flex flex-col md:flex-row items-center justify-between mb-8 gap-4">
        <h1 className="text-3xl font-bold gradient-text">My Blinks</h1>
        <Link href="/create" className="button-primary inline-flex items-center gap-2">
          Create New Blink
          <ArrowRight className="h-4 w-4" />
        </Link>
      </div>

      {loading ? (
        <div className="flex justify-center py-12">
          <div className="relative size-12">
            <div className="absolute inset-0 rounded-full border-4 border-t-purple-500 border-r-transparent border-b-transparent border-l-transparent animate-spin"></div>
            <div className="absolute inset-2 rounded-full border-4 border-t-transparent border-r-indigo-500 border-b-transparent border-l-transparent animate-spin animation-delay-150"></div>
          </div>
        </div>
      ) : blinks.length > 0 ? (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {blinks.map((blink) => (
            <div key={blink._id} className="card overflow-hidden">
              <div className="relative h-48 w-full bg-black/50">
                {blink.icon ? (
                  <Image src={blink.icon || "/placeholder.svg"} alt={blink.title} fill className="object-cover" />
                ) : (
                  <div className="absolute inset-0 flex items-center justify-center">
                    <span className="text-gray-500">No Image</span>
                  </div>
                )}
              </div>
              <div className="p-4">
                <h3 className="text-xl font-bold mb-2">{blink.title}</h3>
                <p className="text-gray-300 text-sm mb-4">{blink.description}</p>

                <div className="flex items-center gap-2 mb-4">
                  <div className="flex-1 p-2 bg-black/30 rounded-lg text-xs overflow-hidden overflow-ellipsis whitespace-nowrap">
                    https://dial.to/?action=solana-action:{blink._id}
                  </div>
                  <button
                    onClick={() => handleCopy(blink._id)}
                    className="p-2 rounded-lg bg-black/30 hover:bg-purple-600/20 transition-colors duration-300"
                    title="Copy to clipboard"
                  >
                    {copied === blink._id ? "Copied!" : <HiOutlineClipboardCopy size={18} />}
                  </button>
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => handleTweet(blink._id)}
                    className="flex-1 py-2 px-3 rounded-lg bg-purple-600/20 hover:bg-purple-600/30 text-purple-300 text-sm font-medium transition-colors duration-300"
                  >
                    <HiOutlineShare size={16} className="inline mr-1" />
                    Share
                  </button>
                  <Link
                    href={`/create?edit=${blink._id}`}
                    className="flex-1 py-2 px-3 rounded-lg bg-black/30 hover:bg-white/10 text-white text-sm font-medium text-center transition-colors duration-300"
                  >
                    View Details
                  </Link>
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="card p-8 text-center">
          <p className="text-gray-300 mb-6">You haven't created any Blinks yet.</p>
          <Link href="/create" className="button-primary">
            Create Your First Blink
          </Link>
        </div>
      )}
    </div>
  )
}
