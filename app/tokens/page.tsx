"use client"

import { useState, useEffect } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import { WalletButton } from "@/components/solana/solana-provider"
import { TokenSelector } from "@/components/tokens/token-selector"
import { BuySellForm } from "@/components/tokens/buy-sell-form"
import { TokenHistory } from "@/components/tokens/token-history"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card } from "@/components/ui/card"
import LoadingScreen from "@/components/loading/loading"

export default function TokensPage() {
  const { publicKey, connected } = useWallet()
  const [selectedToken, setSelectedToken] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [transactions, setTransactions] = useState<any[]>([])

  // Fetch transaction history when wallet is connected
  useEffect(() => {
    if (connected && publicKey) {
      fetchTransactionHistory()
    }
  }, [connected, publicKey])

  const fetchTransactionHistory = async () => {
    if (!publicKey) return

    try {
      // This would be replaced with actual API call
      setTransactions([
        {
          id: "tx1",
          type: "buy",
          token: "SOL",
          amount: 1.5,
          price: 150.25,
          timestamp: new Date().toISOString(),
        },
        {
          id: "tx2",
          type: "sell",
          token: "USDC",
          amount: 100,
          price: 1.0,
          timestamp: new Date(Date.now() - 86400000).toISOString(),
        },
      ])
    } catch (error) {
      console.error("Error fetching transaction history:", error)
    }
  }

  if (!connected) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <h1 className="text-3xl font-bold mb-6 gradient-text">Token Trading</h1>
        <div className="card max-w-md mx-auto p-8">
          <p className="text-gray-300 mb-6">Connect your wallet to buy and sell tokens</p>
          <WalletButton />
        </div>
      </div>
    )
  }

  return (
    <div className="container mx-auto py-8">
      {isLoading && <LoadingScreen subtext="Processing your transaction..." />}

      <h1 className="text-3xl font-bold mb-8 gradient-text">Token Trading</h1>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <div className="lg:col-span-2">
          <Tabs defaultValue="buy" className="w-full">
            <Card className="mb-6 p-6">
              <TabsList className="grid w-full grid-cols-2 mb-6">
                <TabsTrigger value="buy">Buy Tokens</TabsTrigger>
                <TabsTrigger value="sell">Sell Tokens</TabsTrigger>
              </TabsList>

              <TabsContent value="buy">
                <BuySellForm
                  type="buy"
                  selectedToken={selectedToken}
                  setSelectedToken={setSelectedToken}
                  setIsLoading={setIsLoading}
                />
              </TabsContent>

              <TabsContent value="sell">
                <BuySellForm
                  type="sell"
                  selectedToken={selectedToken}
                  setSelectedToken={setSelectedToken}
                  setIsLoading={setIsLoading}
                />
              </TabsContent>
            </Card>
          </Tabs>

          <Card className="p-6">
            <h2 className="text-xl font-bold mb-4">Transaction History</h2>
            <TokenHistory transactions={transactions} />
          </Card>
        </div>

        <div>
          <Card className="p-6 sticky top-24">
            <h2 className="text-xl font-bold mb-4">Select Token</h2>
            <TokenSelector selectedToken={selectedToken} setSelectedToken={setSelectedToken} />
          </Card>
        </div>
      </div>
    </div>
  )
}
