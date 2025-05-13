import Link from "next/link"
import Image from "next/image"
import { ArrowRight, Zap, Shield, Globe, Coins } from "lucide-react"

export default function Home() {
  return (
    <div className="flex flex-col gap-12">
      {/* Hero Section */}
      <section className="w-full py-8 md:py-12 lg:py-16">
        <div className="container px-4 md:px-6 pt-4">
          <div className="grid gap-6 lg:grid-cols-[1fr_400px] lg:gap-12 xl:grid-cols-[1fr_400px]">
            <div className="flex flex-col justify-center space-y-4">
              <div className="space-y-2">
                <h1 className="text-3xl font-bold tracking-tighter sm:text-5xl xl:text-6xl/none">
                  Create Shareable <span className="gradient-text">Solana Actions</span> in Seconds
                </h1>
                <p className="max-w-[600px] text-gray-400 md:text-xl">
                  Generate custom Blinks that let anyone send SOL with just a click. Perfect for creators, developers,
                  and businesses.
                </p>
              </div>
              <div className="flex flex-col gap-2 min-[400px]:flex-row">
                <Link href="/create" className="button-primary inline-flex items-center gap-2">
                  Create Your Blink
                  <ArrowRight className="h-4 w-4" />
                </Link>
                <Link href="/tokens" className="button-secondary">
                  Trade Tokens
                </Link>
              </div>
            </div>
            <div className="flex items-center justify-center">
              <div className="relative w-full max-w-sm overflow-hidden rounded-xl border border-white/10 bg-black/20 p-2 shadow-2xl">
                <Image
                  src="/placeholder.svg?key=qilm9"
                  width={400}
                  height={600}
                  alt="Solana Blink Preview"
                  className="rounded-lg"
                />
                <div className="absolute bottom-4 left-4 right-4 rounded-lg bg-black/80 backdrop-blur-sm p-4">
                  <h3 className="text-lg font-semibold text-white">Support Your Favorite Creator</h3>
                  <p className="text-sm text-gray-300 mb-3">Send SOL directly with one click</p>
                  <div className="grid grid-cols-3 gap-2">
                    <button className="bg-purple-600 hover:bg-purple-700 text-white text-sm py-1 rounded-md">
                      0.1 SOL
                    </button>
                    <button className="bg-purple-600 hover:bg-purple-700 text-white text-sm py-1 rounded-md">
                      1 SOL
                    </button>
                    <button className="bg-purple-600 hover:bg-purple-700 text-white text-sm py-1 rounded-md">
                      5 SOL
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="w-full py-12 md:py-24 lg:py-32">
        <div className="container px-4 md:px-6">
          <div className="flex flex-col items-center justify-center space-y-4 text-center">
            <div className="space-y-2">
              <h2 className="text-3xl font-bold tracking-tighter sm:text-5xl">Features</h2>
              <p className="max-w-[900px] text-gray-400 md:text-xl/relaxed lg:text-base/relaxed xl:text-xl/relaxed">
                Everything you need to create and share Solana Actions Blinks
              </p>
            </div>
          </div>
          <div className="mx-auto grid max-w-5xl items-center gap-6 py-12 lg:grid-cols-4 lg:gap-12">
            <div className="flex flex-col justify-center space-y-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/10 text-purple-600">
                <Zap className="h-6 w-6" />
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Fast & Easy</h3>
                <p className="text-gray-400">
                  Create a Blink in seconds with our intuitive interface. No coding required.
                </p>
              </div>
            </div>
            <div className="flex flex-col justify-center space-y-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/10 text-purple-600">
                <Shield className="h-6 w-6" />
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Secure</h3>
                <p className="text-gray-400">Built on Solana's secure blockchain with wallet authentication.</p>
              </div>
            </div>
            <div className="flex flex-col justify-center space-y-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/10 text-purple-600">
                <Globe className="h-6 w-6" />
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Shareable</h3>
                <p className="text-gray-400">Share your Blink anywhere with a simple link. Works on any device.</p>
              </div>
            </div>
            <div className="flex flex-col justify-center space-y-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/10 text-purple-600">
                <Coins className="h-6 w-6" />
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Token Trading</h3>
                <p className="text-gray-400">Buy and sell tokens directly from our platform with low fees.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Token Trading Section */}
      <section className="w-full py-12 md:py-24 lg:py-32 bg-black/30 backdrop-blur-sm rounded-3xl border border-white/5">
        <div className="container px-4 md:px-6">
          <div className="flex flex-col items-center justify-center space-y-4 text-center mb-12">
            <div className="space-y-2">
              <h2 className="text-3xl font-bold tracking-tighter sm:text-5xl">Token Trading</h2>
              <p className="max-w-[900px] text-gray-400 md:text-xl/relaxed lg:text-base/relaxed xl:text-xl/relaxed">
                Buy and sell Solana tokens with our easy-to-use platform
              </p>
            </div>
          </div>

          <div className="grid md:grid-cols-2 gap-8">
            <div className="card p-8 flex flex-col items-center text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-green-600 text-white mb-6">
                <Coins className="h-8 w-8" />
              </div>
              <h3 className="text-2xl font-bold mb-4">Buy Tokens</h3>
              <p className="text-gray-400 mb-6">
                Purchase your favorite Solana tokens directly through our platform. Low fees and fast transactions.
              </p>
              <Link href="/tokens" className="button-primary">
                Buy Tokens
              </Link>
            </div>

            <div className="card p-8 flex flex-col items-center text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-purple-600 text-white mb-6">
                <Coins className="h-8 w-8" />
              </div>
              <h3 className="text-2xl font-bold mb-4">Sell Tokens</h3>
              <p className="text-gray-400 mb-6">
                Create a shareable link to sell your tokens. Set your own commission and share with your audience.
              </p>
              <Link href="/sell-token" className="button-primary">
                Sell Tokens
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* How It Works Section */}
      <section className="w-full py-12 md:py-24 lg:py-32">
        <div className="container px-4 md:px-6">
          <div className="flex flex-col items-center justify-center space-y-4 text-center">
            <div className="space-y-2">
              <h2 className="text-3xl font-bold tracking-tighter sm:text-5xl">How It Works</h2>
              <p className="max-w-[900px] text-gray-400 md:text-xl/relaxed lg:text-base/relaxed xl:text-xl/relaxed">
                Create and share Solana Actions Blinks in three simple steps
              </p>
            </div>
          </div>
          <div className="mx-auto grid max-w-5xl items-start gap-6 py-12 lg:grid-cols-3 lg:gap-12">
            <div className="flex flex-col items-center space-y-4 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-purple-600 text-white text-2xl font-bold">
                1
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Connect Wallet</h3>
                <p className="text-gray-400">
                  Connect your Solana wallet to get started. We support most popular wallets.
                </p>
              </div>
            </div>
            <div className="flex flex-col items-center space-y-4 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-purple-600 text-white text-2xl font-bold">
                2
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Customize Your Blink</h3>
                <p className="text-gray-400">Add a title, description, and image to make your Blink stand out.</p>
              </div>
            </div>
            <div className="flex flex-col items-center space-y-4 text-center">
              <div className="flex h-16 w-16 items-center justify-center rounded-full bg-purple-600 text-white text-2xl font-bold">
                3
              </div>
              <div className="space-y-2">
                <h3 className="text-xl font-bold">Share & Collect</h3>
                <p className="text-gray-400">Share your Blink link with anyone and start receiving SOL instantly.</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="w-full py-12 md:py-24 lg:py-32">
        <div className="container px-4 md:px-6">
          <div className="flex flex-col items-center justify-center space-y-4 text-center">
            <div className="space-y-2">
              <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl md:text-5xl">Ready to Get Started?</h2>
              <p className="max-w-[600px] text-gray-400 md:text-xl/relaxed lg:text-base/relaxed xl:text-xl/relaxed">
                Join thousands of creators and businesses using Solana Actions Blinks.
              </p>
            </div>
            <div className="flex flex-col gap-2 min-[400px]:flex-row">
              <Link href="/create" className="button-primary">
                Create Blink
              </Link>
              <Link href="/tokens" className="button-secondary">
                Trade Tokens
              </Link>
            </div>
          </div>
        </div>
      </section>
    </div>
  )
}
