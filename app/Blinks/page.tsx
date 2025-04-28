"use client"
import { useState, useEffect } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import { useRouter } from "next/navigation"
import DataCard from "../../components/DataCard/dataCard"
import { WalletButton } from "@/components/solana/solana-provider"
import { HiOutlineCollection, HiArrowRight } from "react-icons/hi"

export default function Page() {
  const { publicKey, connected } = useWallet()
  const router = useRouter()
  const [data, setData] = useState<Array<Record<string, any>>>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!connected) {
      router.push("/landing")
    }
  }, [connected, router])

  useEffect(() => {
    const getBlinks = async () => {
      if (!publicKey) return

      try {
        const response = await fetch(`/api/actions/getBlinks?wallet=${encodeURIComponent(publicKey.toString())}`)
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const { blinks } = await response.json()
        setData(blinks || [])
        console.log("Blinks:", blinks)
        setLoading(false)
      } catch (error) {
        setLoading(false)
        console.error("Error fetching blinks:", error)
        // Set empty array on error to prevent undefined issues
        setData([])
      }
    }

    if (publicKey) {
      getBlinks()
    } else {
      setLoading(false)
    }
  }, [publicKey])

  // Show loading or nothing while redirecting
  if (!connected) {
    return null
  }

  return (
    <div className="bg-[var(--bg-color)] text-[var(--text-color)]">
      <section className="py-8 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="mb-6 text-center">
            <h1 className="text-3xl md:text-4xl font-black gradient-text">My Blinks</h1>
            <p className="text-[var(--text-secondary)] mt-2">Manage and share your created Blinks</p>
          </div>

          {!publicKey ? (
            <div className="flex flex-col items-center justify-center py-12 space-y-4">
              <p className="text-[var(--text-secondary)] text-center">Connect your wallet to view your Blinks</p>
              <WalletButton />
            </div>
          ) : loading ? (
            <div className="grid gap-4">
              <div className="shimmer h-24 rounded-xl"></div>
              <div className="shimmer h-24 rounded-xl"></div>
              <div className="shimmer h-24 rounded-xl"></div>
            </div>
          ) : (
            <div>
              {data && data.length > 0 ? (
                <div className="grid gap-4">
                  {data
                    .slice()
                    .reverse()
                    .map((blink) => (
                      <DataCard
                        key={blink["_id"]}
                        code={blink.id || blink["_id"]}
                        base={"https://dial.to/?action=solana-action:"}
                        title={blink.title || `Open a ${blink.poolName || 'Token'} Position`}
                        endpoint={blink.endpoint}
                      />
                    ))}
                </div>
              ) : (
                <div className="text-center py-12">
                  <h2 className="text-xl font-semibold text-[var(--text-color)] mb-2">No Blinks Yet</h2>
                  <p className="text-[var(--text-secondary)] mb-4">Create your first Blink to get started</p>
                </div>
              )}
            </div>
          )}
        </div>
      </section>
    </div>
  )
}
