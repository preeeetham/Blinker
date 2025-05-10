"use client"

import type React from "react"

import { useState, useEffect } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { ArrowRight, AlertCircle } from "lucide-react"
import { Alert, AlertDescription } from "@/components/ui/alert"

interface BuySellFormProps {
  type: "buy" | "sell"
  selectedToken: string | null
  setSelectedToken: (token: string) => void
  setIsLoading: (isLoading: boolean) => void
}

// Sample token data - this would come from an API in a real app
const tokens = {
  sol: {
    name: "Solana",
    symbol: "SOL",
    price: 150.25,
    balance: 2.5,
  },
  usdc: {
    name: "USD Coin",
    symbol: "USDC",
    price: 1.0,
    balance: 500,
  },
  bonk: {
    name: "Bonk",
    symbol: "BONK",
    price: 0.00002,
    balance: 1000000,
  },
  orca: {
    name: "Orca",
    symbol: "ORCA",
    price: 1.75,
    balance: 10,
  },
  ray: {
    name: "Raydium",
    symbol: "RAY",
    price: 0.85,
    balance: 25,
  },
}

export function BuySellForm({ type, selectedToken, setSelectedToken, setIsLoading }: BuySellFormProps) {
  const { publicKey, connected, sendTransaction } = useWallet()
  const [amount, setAmount] = useState("")
  const [usdValue, setUsdValue] = useState(0)
  const [error, setError] = useState<string | null>(null)

  // Reset form when token changes
  useEffect(() => {
    setAmount("")
    setUsdValue(0)
    setError(null)
  }, [selectedToken, type])

  // Calculate USD value when amount changes
  useEffect(() => {
    if (selectedToken && amount) {
      const token = tokens[selectedToken as keyof typeof tokens]
      if (token) {
        setUsdValue(Number.parseFloat(amount) * token.price)
      }
    } else {
      setUsdValue(0)
    }
  }, [amount, selectedToken])

  const handleAmountChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value

    // Only allow numbers and decimals
    if (value === "" || /^\d*\.?\d*$/.test(value)) {
      setAmount(value)

      // Validate amount
      if (selectedToken) {
        const token = tokens[selectedToken as keyof typeof tokens]

        if (type === "sell" && Number.parseFloat(value) > token.balance) {
          setError(`Insufficient ${token.symbol} balance`)
        } else {
          setError(null)
        }
      }
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (!selectedToken || !amount || Number.parseFloat(amount) <= 0) {
      setError("Please enter a valid amount")
      return
    }

    if (error) return

    setIsLoading(true)

    try {
      // This would be replaced with actual transaction logic
      console.log(`${type === "buy" ? "Buying" : "Selling"} ${amount} ${selectedToken.toUpperCase()}`)

      // Simulate transaction delay
      await new Promise((resolve) => setTimeout(resolve, 2000))

      // Success
      setAmount("")
      setUsdValue(0)

      // Show success message
      alert(`Successfully ${type === "buy" ? "bought" : "sold"} ${amount} ${selectedToken.toUpperCase()}`)
    } catch (error) {
      console.error(`Error ${type}ing token:`, error)
      setError(`Failed to ${type} token. Please try again.`)
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {!selectedToken ? (
        <div className="text-center py-8">
          <p className="text-gray-400 mb-4">Please select a token from the list</p>
          <Button variant="outline" type="button" onClick={() => document.getElementById("token-selector")?.focus()}>
            Select Token
          </Button>
        </div>
      ) : (
        <>
          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-300">
              {type === "buy" ? "Amount to Buy" : "Amount to Sell"}
            </label>
            <div className="relative">
              <Input type="text" value={amount} onChange={handleAmountChange} placeholder="0.00" className="pr-20" />
              <div className="absolute right-3 top-1/2 -translate-y-1/2 text-sm font-medium text-gray-400">
                {tokens[selectedToken as keyof typeof tokens]?.symbol}
              </div>
            </div>
            <div className="flex justify-between text-xs text-gray-400">
              <span>
                ≈ ${usdValue.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </span>
              {type === "sell" && (
                <span>
                  Balance: {tokens[selectedToken as keyof typeof tokens]?.balance.toLocaleString()}{" "}
                  {tokens[selectedToken as keyof typeof tokens]?.symbol}
                </span>
              )}
            </div>
          </div>

          {error && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <label className="block text-sm font-medium text-gray-300">Transaction Summary</label>
            <div className="bg-black/20 rounded-lg p-4 space-y-3">
              <div className="flex justify-between">
                <span className="text-gray-400">Token Price</span>
                <span>
                  $
                  {tokens[selectedToken as keyof typeof tokens]?.price.toLocaleString(undefined, {
                    minimumFractionDigits: 2,
                    maximumFractionDigits: 6,
                  })}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Amount</span>
                <span>
                  {amount || "0"} {tokens[selectedToken as keyof typeof tokens]?.symbol}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Total Value</span>
                <span>
                  ${usdValue.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Network Fee</span>
                <span>~0.000005 SOL</span>
              </div>
            </div>
          </div>

          <Button
            type="submit"
            className={`w-full ${type === "buy" ? "bg-green-600 hover:bg-green-700" : "bg-red-600 hover:bg-red-700"}`}
            disabled={!amount || Number.parseFloat(amount) <= 0 || !!error}
          >
            {type === "buy" ? "Buy" : "Sell"} {tokens[selectedToken as keyof typeof tokens]?.symbol}
            <ArrowRight className="ml-2 h-4 w-4" />
          </Button>
        </>
      )}
    </form>
  )
}
