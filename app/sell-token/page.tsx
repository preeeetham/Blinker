"use client"

import { useState } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import TokenForm from "@/components/tokens/token-form"
import TokenPreview from "@/components/tokens/token-preview"
import LoadingScreen from "@/components/loading/loading"
import { HiOutlineClipboardCopy, HiOutlineShare } from "react-icons/hi"

export default function SellTokenPage() {
  const { publicKey, connected } = useWallet()
  const [mint, setMint] = useState("")
  const [description, setDescription] = useState("")
  const [percentage, setPercentage] = useState(0.5)
  const [takeCommission, setTakeCommission] = useState<"yes" | "no">("yes")
  const [showPreview, setShowPreview] = useState(false)
  const [loading, setLoading] = useState(false)
  const [tokenData, setTokenData] = useState<{ icon: string; title: string } | null>(null)
  const [blinkLink, setBlinkLink] = useState("")
  const [copied, setCopied] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)

  const handlePreview = async () => {
    if (!mint) {
      window.alert("Please enter a mint address")
      return
    }

    setLoading(true)
    try {
      const response = await fetch(`/api/actions/token-info?mint=${mint}`)
      if (!response.ok) {
        throw new Error("Failed to fetch token info")
      }

      const data = await response.json()
      setTokenData(data)
      setShowPreview(true)
    } catch (error) {
      console.error("Error fetching token info:", error)
      window.alert("Failed to fetch token info. Please check the mint address and try again.")
    } finally {
      setLoading(false)
    }
  }

  const handleSubmit = async () => {
    if (!connected || !publicKey) {
      console.error("Wallet not connected")
      return
    }

    if (!mint || !description) {
      console.error("Please fill all required fields")
      window.alert("Please fill all required fields")
      return
    }

    setLoading(true)
    try {
      const walletAddress = publicKey.toString()
      const response = await fetch("/api/actions/token-blink", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          label: "Buy Token",
          description,
          wallet: walletAddress,
          mint,
          commission: takeCommission,
          percentage: takeCommission === "yes" ? percentage : 0,
        }),
      })

      if (!response.ok) {
        throw new Error("Failed to generate token blink")
      }

      const data = await response.json()
      setBlinkLink(data.blinkLink)
      setShowSuccess(true)
    } catch (error) {
      console.error("Failed to generate token blink:", error)
      window.alert("Failed to generate token blink")
    } finally {
      setLoading(false)
    }
  }

  const handleCopy = () => {
    navigator.clipboard.writeText(`https://dial.to/?action=solana-action:${blinkLink}`)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleTweet = () => {
    const tweetText = `Check out this Token Blink I just made @getblinkdotfun: https://dial.to/?action=solana-action:${blinkLink}`
    const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`
    window.open(twitterUrl, "_blank")
  }

  const handleNew = () => {
    setMint("")
    setDescription("")
    setPercentage(0.5)
    setTakeCommission("yes")
    setShowPreview(false)
    setTokenData(null)
    setBlinkLink("")
    setShowSuccess(false)
  }

  return (
    <div className="container mx-auto py-8">
      {loading && <LoadingScreen subtext="Processing your request..." />}

      {showSuccess ? (
        <div className="max-w-2xl mx-auto card p-8">
          <h1 className="text-2xl md:text-3xl font-bold mb-6 gradient-text">Your Token Blink is Ready!</h1>

          <div className="p-4 rounded-xl bg-black/30 border border-white/10 mb-6">
            <p className="text-sm text-gray-400 mb-2">Blink Link:</p>
            <div className="flex items-center gap-2">
              <div className="flex-1 p-3 bg-black/50 rounded-lg text-sm overflow-hidden overflow-ellipsis whitespace-nowrap">
                https://dial.to/?action=solana-action:{blinkLink}
              </div>
              <button
                onClick={handleCopy}
                className="p-3 rounded-lg bg-black/30 hover:bg-purple-600/20 transition-colors duration-300"
                title="Copy to clipboard"
              >
                {copied ? "Copied!" : <HiOutlineClipboardCopy size={20} />}
              </button>
            </div>
          </div>

          <div className="flex flex-col sm:flex-row gap-4">
            <button className="button-primary flex-1" onClick={handleTweet}>
              <HiOutlineShare size={18} className="mr-2" />
              Share on X
            </button>

            <button className="button-secondary flex-1" onClick={handleNew}>
              Create New Token Blink
            </button>
          </div>
        </div>
      ) : (
        <div className="flex flex-col lg:flex-row gap-8 items-start">
          <div className="w-full lg:w-3/5">
            <div className="card md:p-8">
              <TokenForm
                mint={mint}
                description={description}
                percentage={percentage}
                takeCommission={takeCommission}
                showPreview={showPreview}
                setMint={setMint}
                setDescription={setDescription}
                setPercentage={setPercentage}
                setTakeCommission={setTakeCommission}
                handlePreview={handlePreview}
                handleSubmit={handleSubmit}
                connected={connected}
                publicKey={publicKey}
              />
            </div>
          </div>

          <div className="w-full lg:w-2/5 sticky top-24">
            <div className="card p-6">
              <h2 className="text-xl font-bold mb-4">Preview</h2>
              <div className="flex justify-center">
                {showPreview && tokenData ? (
                  <TokenPreview
                    icon={tokenData.icon}
                    description={description}
                    title={tokenData.title || "Your Title"}
                  />
                ) : (
                  <div className="text-center py-8 text-gray-400">
                    <p>Enter a token mint address and click "Preview Blink" to see how your token blink will look.</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
