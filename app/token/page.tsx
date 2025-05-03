"use client"

import { useState, useRef, useEffect, useCallback } from "react"
import { useWallet, useConnection } from "@solana/wallet-adapter-react"
import { useRouter } from "next/navigation"
import TokenPreview from "@/components/preview/token-preview"
import TokenForm from "@/components/form/tokenForm"
import { Transaction } from "@solana/web3.js"
import { HiOutlineClipboardCopy, HiOutlineShare, HiOutlinePlus, HiX, HiExclamation } from "react-icons/hi"
import { confirmTransaction } from "@/server/transaction"
import { validateText, validateMintAddress, validatePercentage, sanitizeInput } from "@/lib/validation"
import { useErrorHandler } from "@/hooks/useErrorHandler"

type CommissionType = "yes" | "no"

interface ErrorPopupProps {
  message: string;
  isVisible: boolean;
  onClose: () => void;
}

const ErrorPopup: React.FC<ErrorPopupProps> = ({ message, isVisible, onClose }) => {
  if (!isVisible) return null

  return (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4">
      <div className="bg-red-50 border border-red-300 rounded-lg p-4 max-w-md w-full shadow-lg animate-in fade-in duration-200">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <HiExclamation className="h-5 w-5 text-red-400" />
          </div>
          <div className="ml-3 flex-1">
            <h3 className="text-sm font-medium text-red-800">
              Error Occurred
            </h3>
            <div className="mt-2 text-sm text-red-700">
              <p>{message}</p>
            </div>
          </div>
          <div className="ml-auto pl-3">
            <div className="-mx-1.5 -my-1.5">
              <button
                onClick={onClose}
                className="inline-flex rounded-md bg-red-50 p-1.5 text-red-500 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-red-600 focus:ring-offset-2 focus:ring-offset-red-50"
              >
                <HiX className="h-5 w-5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

export default function Page() {
  const { publicKey, connected, sendTransaction } = useWallet()
  const router = useRouter()
  const [icon, setIcon] = useState<string>("")
  const [percentage, setPercentage] = useState<number>(0)
  const [takeCommission, setTakeCommission] = useState<CommissionType>("no")
  const [description, setDescription] = useState<string>("")
  const [title, setTitle] = useState<string>("")
  const [mint, setMint] = useState<string>("")
  const [showPreview, setShowPreview] = useState(false)
  const [loading, setLoading] = useState(false)
  const [loadingText, setLoadingText] = useState("Please Wait!!")
  const [showForm, setShowForm] = useState(true)
  const [blinkLink, setBlinkLink] = useState("")
  const [copied, setCopied] = useState(false)
  

  const { error, showError, hideError, handleAsyncError } = useErrorHandler()
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})
  
  const form = useRef<HTMLDivElement | null>(null)
  const { connection } = useConnection()

  useEffect(() => {
    if (!connected) {
      router.push("/landing")
    }
  }, [connected, router])

  useEffect(() => {
    setShowPreview(false)
  }, [mint])

  useEffect(() => {
    if (takeCommission === "no") {
      setPercentage(0)
    }
  }, [takeCommission])

  const validateForm = useCallback(() => {
    const errors: Record<string, string> = {};
    
    // Validate mint address
    const mintValidation = validateMintAddress(mint);
    if (!mintValidation.isValid) {
      errors.mint = mintValidation.error || 'Invalid mint address';
    }

    // Validate description
    const descValidation = validateText(description, 143, 'Description');
    if (!descValidation.isValid) {
      errors.description = descValidation.error || 'Invalid description';
    }

    // Validate percentage if commission is enabled
    if (takeCommission === "yes") {
      const percentageValidation = validatePercentage(percentage);
      if (!percentageValidation.isValid) {
        errors.percentage = percentageValidation.error || 'Invalid percentage';
      }
    }

    setValidationErrors(errors);
    return Object.keys(errors).length === 0;
  }, [mint, description, percentage, takeCommission]);

  // Show loading or nothing while redirecting
  if (!connected) {
    return null
  }

  const renderLoading = () => (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center">
      <div className="text-white text-lg font-semibold p-4 rounded-lg bg-[var(--card-bg)] border border-[var(--border-color)] shadow-lg">
        {loadingText}
      </div>
    </div>
  )

  const handleSubmit = useCallback(async () => {
    setLoading(true)
    setLoadingText("Waiting for Transaction confirmation!!")
    hideError()
    
    if (!connected || !publicKey) {
      showError("Please connect your wallet first")
      setLoading(false)
      return
    }

    if (!validateForm()) {
      setLoading(false)
      return
    }

    // Sanitize inputs
    const sanitizedDescription = sanitizeInput(description);
    const sanitizedMint = sanitizeInput(mint);

    const result = await handleAsyncError(async () => {
      const response = await fetch("/api/actions/generate-blink/token", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          label: "Buy Token",
          description: sanitizedDescription,
          wallet: publicKey.toString(),
          mint: sanitizedMint,
          commission: takeCommission,
          percentage: percentage,
        }),
      })

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || "Failed to generate blink");
      }

      const data = await response.json()
      const { serializedTransaction, blockhash, lastValidBlockHeight } = data.transaction
      const transaction = Transaction.from(Buffer.from(serializedTransaction, "base64"))

      const signature = await sendTransaction(transaction, connection)
      console.log("Transaction sent:", signature)

      const confirmation = await confirmTransaction(signature, blockhash, lastValidBlockHeight)
      console.log("Transaction confirmed:", confirmation)

      const res = await fetch("/api/actions/order", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          signature,
          orderId: data.id.toString(),
        }),
      })

      if (!res.ok) {
        const errorData = await res.json().catch(() => ({}));
        throw new Error(errorData.error || "Failed to process payment");
      }

      const link = await res.json()
      if (!link.blinkLink) {
        throw new Error("Failed to generate blink link")
      }

      return link;
    }, "Transaction failed. Please try again.");

    if (result) {
      setBlinkLink(result.blinkLink)
      setShowForm(false)
    }
    
    setLoading(false)
  }, [connected, publicKey, validateForm, handleAsyncError, sendTransaction, connection, hideError, showError, description, mint, takeCommission, percentage]);

  const handlePreview = useCallback(async () => {
    setLoading(true)
    setLoadingText("Generating Blink Preview!!")
    hideError()
    
    if (!connected || !publicKey) {
      showError("Please connect your wallet first")
      setLoading(false)
      return
    }

    if (!validateForm()) {
      setLoading(false)
      return
    }

    const result = await handleAsyncError(async () => {
      const response = await fetch(`/api/actions/generate-blink/token?mint=${encodeURIComponent(mint)}`)
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || "Failed to generate blink preview");
      }

      const data = await response.json()
      setShowPreview(true)
      setIcon(data.icon)
      setTitle(data.title)
      return data;
    }, "Invalid Mint Address!!");

    setLoading(false)
  }, [connected, publicKey, validateForm, handleAsyncError, hideError, showError, mint]);

  const handleCopy = () => {
    navigator.clipboard.writeText(`https://dial.to/?action=solana-action:${blinkLink}`)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  const handleTweet = () => {
    const tweetText = `Check out this Blink I just made using blinkgen(dot)xyz : https://dial.to/?action=solana-action:${blinkLink}`
    const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`
    window.open(twitterUrl, "_blank")
  }

  const handleNew = () => {
    setShowForm(true)
  }

  return (
    <div className="bg-[var(--bg-color)] text-[var(--text-color)]">
      {loading && renderLoading()}
      
      <ErrorPopup 
        message={error.message}
        isVisible={error.hasError}
        onClose={hideError}
      />

      <section className="py-8 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="mb-8 text-center">
            <h1 className="text-3xl md:text-4xl font-black gradient-text">Sell/Resell Tokens</h1>
            <p className="text-[var(--text-secondary)] mt-2">Create token Blinks with optional commission</p>
          </div>

          {showForm ? (
            <div className="grid lg:grid-cols-2 gap-8 items-start">
              <div>
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

              <div>
                <div className="mb-4 text-center">
                  <h2 className="text-xl font-semibold gradient-text">Live Preview</h2>
                </div>
                <TokenPreview
                  icon={icon || "solana.jpg"}
                  description={description || "Your Description shows up here, Keep it short and simple"}
                  title={title || "Your Title"}
                />
              </div>
            </div>
          ) : (
            <div className="max-w-2xl mx-auto">
              <div className="text-center fade-in">
                <div className="mb-6">
                  <div className="p-6 rounded-2xl bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white inline-block mb-4">
                    <HiOutlineShare className="w-12 h-12" />
                  </div>
                  <h1 className="text-3xl md:text-4xl font-bold mb-2 gradient-text">Your Token Blink is Ready!</h1>
                  <p className="text-[var(--text-secondary)]">Share this link to let others buy your token</p>
                </div>

                <div className="p-4 rounded-xl bg-[var(--card-bg)] border border-[var(--border-color)] mb-6">
                  <p className="text-sm text-[var(--text-secondary)] mb-2 font-medium">Blink Link:</p>
                  <div className="flex items-center gap-3">
                    <div
                      className="flex-1 p-3 bg-[var(--bg-color)] rounded-lg text-sm overflow-hidden overflow-ellipsis whitespace-nowrap cursor-pointer hover:bg-[var(--hover-bg)] transition-colors duration-300"
                      onClick={() => window.open(`https://dial.to/?action=solana-action:${blinkLink}`, "_blank")}
                    >
                      https://dial.to/?action=solana-action:{blinkLink}
                    </div>
                    <button
                      onClick={handleCopy}
                      className="p-3 rounded-lg bg-[var(--border-color)] hover:bg-[var(--accent-primary)] hover:text-white transition-all duration-300"
                      title="Copy to clipboard"
                    >
                      {copied ? "Copied!" : <HiOutlineClipboardCopy size={20} />}
                    </button>
                  </div>
                </div>

                <div className="grid sm:grid-cols-2 gap-4">
                  <button
                    className="button-primary flex items-center justify-center gap-2 py-3"
                    onClick={handleTweet}
                  >
                    <HiOutlineShare size={18} />
                    Share on X
                  </button>

                  <button
                    className="button-secondary flex items-center justify-center gap-2 py-3"
                    onClick={handleNew}
                  >
                    <HiOutlinePlus size={18} />
                    Create New Blink
                  </button>
                </div>
              </div>
            </div>
          )}
        </div>
      </section>

      
    </div>
  )
}