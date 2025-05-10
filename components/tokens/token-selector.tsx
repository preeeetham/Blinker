"use client"

import { useState } from "react"
import Image from "next/image"
import { Input } from "@/components/ui/input"
import { Search } from "lucide-react"

interface TokenSelectorProps {
  selectedToken: string | null
  setSelectedToken: (token: string) => void
}

// Sample token data - this would come from an API in a real app
const tokens = [
  {
    id: "sol",
    name: "Solana",
    symbol: "SOL",
    icon: "/placeholder.svg?key=sol-icon",
    price: 150.25,
    change: 5.2,
  },
  {
    id: "usdc",
    name: "USD Coin",
    symbol: "USDC",
    icon: "/placeholder.svg?key=usdc-icon",
    price: 1.0,
    change: 0.01,
  },
  {
    id: "bonk",
    name: "Bonk",
    symbol: "BONK",
    icon: "/placeholder.svg?key=bonk-icon",
    price: 0.00002,
    change: -2.5,
  },
  {
    id: "orca",
    name: "Orca",
    symbol: "ORCA",
    icon: "/placeholder.svg?key=orca-icon",
    price: 1.75,
    change: 3.8,
  },
  {
    id: "ray",
    name: "Raydium",
    symbol: "RAY",
    icon: "/placeholder.svg?key=ray-icon",
    price: 0.85,
    change: -1.2,
  },
]

export function TokenSelector({ selectedToken, setSelectedToken }: TokenSelectorProps) {
  const [searchQuery, setSearchQuery] = useState("")

  const filteredTokens = tokens.filter(
    (token) =>
      token.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      token.symbol.toLowerCase().includes(searchQuery.toLowerCase()),
  )

  return (
    <div className="space-y-4">
      <div className="relative">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
        <Input
          type="text"
          placeholder="Search tokens..."
          className="pl-10"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>

      <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2">
        {filteredTokens.map((token) => (
          <div
            key={token.id}
            className={`flex items-center justify-between p-3 rounded-lg cursor-pointer transition-colors duration-200 ${
              selectedToken === token.id
                ? "bg-purple-600/20 border border-purple-500/30"
                : "hover:bg-white/5 border border-transparent"
            }`}
            onClick={() => setSelectedToken(token.id)}
          >
            <div className="flex items-center gap-3">
              <div className="relative w-8 h-8 rounded-full overflow-hidden bg-black/20">
                <Image src={token.icon || "/placeholder.svg"} alt={token.name} fill className="object-cover" />
              </div>
              <div>
                <p className="font-medium">{token.symbol}</p>
                <p className="text-xs text-gray-400">{token.name}</p>
              </div>
            </div>
            <div className="text-right">
              <p className="font-medium">${token.price.toLocaleString()}</p>
              <p className={`text-xs ${token.change >= 0 ? "text-green-400" : "text-red-400"}`}>
                {token.change >= 0 ? "+" : ""}
                {token.change}%
              </p>
            </div>
          </div>
        ))}

        {filteredTokens.length === 0 && (
          <div className="text-center py-4 text-gray-400">No tokens found matching "{searchQuery}"</div>
        )}
      </div>
    </div>
  )
}
