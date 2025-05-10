"use client"

import { useState } from "react"
import { format } from "date-fns"
import { ArrowUpRight, ArrowDownRight } from "lucide-react"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"

interface Transaction {
  id: string
  type: "buy" | "sell"
  token: string
  amount: number
  price: number
  timestamp: string
}

interface TokenHistoryProps {
  transactions: Transaction[]
}

export function TokenHistory({ transactions }: TokenHistoryProps) {
  const [filter, setFilter] = useState("all")
  const [searchQuery, setSearchQuery] = useState("")

  const filteredTransactions = transactions.filter((tx) => {
    // Filter by type
    if (filter !== "all" && tx.type !== filter) return false

    // Filter by search query
    if (
      searchQuery &&
      !tx.token.toLowerCase().includes(searchQuery.toLowerCase()) &&
      !tx.id.toLowerCase().includes(searchQuery.toLowerCase())
    ) {
      return false
    }

    return true
  })

  if (transactions.length === 0) {
    return (
      <div className="text-center py-8">
        <p className="text-gray-400">No transaction history found</p>
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-col sm:flex-row gap-3">
        <div className="flex-1">
          <Input
            type="text"
            placeholder="Search by token or transaction ID"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <div>
          <Select value={filter} onValueChange={setFilter}>
            <SelectTrigger className="w-[180px]">
              <SelectValue placeholder="Filter by type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">All Transactions</SelectItem>
              <SelectItem value="buy">Buy Only</SelectItem>
              <SelectItem value="sell">Sell Only</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2">
        {filteredTransactions.length > 0 ? (
          filteredTransactions.map((tx) => (
            <div
              key={tx.id}
              className="flex items-center justify-between p-4 rounded-lg border border-white/5 bg-black/20 hover:bg-black/30 transition-colors"
            >
              <div className="flex items-center gap-3">
                <div
                  className={`flex items-center justify-center w-10 h-10 rounded-full ${
                    tx.type === "buy" ? "bg-green-500/20 text-green-500" : "bg-red-500/20 text-red-500"
                  }`}
                >
                  {tx.type === "buy" ? <ArrowDownRight size={20} /> : <ArrowUpRight size={20} />}
                </div>
                <div>
                  <p className="font-medium">
                    {tx.type === "buy" ? "Bought" : "Sold"} {tx.amount} {tx.token}
                  </p>
                  <p className="text-xs text-gray-400">{format(new Date(tx.timestamp), "MMM d, yyyy 'at' h:mm a")}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="font-medium">${(tx.amount * tx.price).toLocaleString()}</p>
                <p className="text-xs text-gray-400">
                  @ ${tx.price.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 6 })}
                </p>
              </div>
            </div>
          ))
        ) : (
          <div className="text-center py-4 text-gray-400">No transactions found matching your filters</div>
        )}
      </div>
    </div>
  )
}
