import Image from "next/image"
import Link from "next/link"
import { ArrowRight, Zap, Shield, Coins, Share2, Wallet, FileText, CheckCircle, RefreshCw } from "lucide-react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"

export default function HowItWorksPage() {
  return (
    <div className="container mx-auto py-8">
      <h1 className="text-3xl md:text-4xl font-bold mb-6 gradient-text text-center">How SolBlinks Works</h1>
      <p className="text-gray-300 text-center max-w-3xl mx-auto mb-12">
        SolBlinks makes it easy to create shareable Solana transactions and trade tokens with just a few clicks. Learn
        how to get started below.
      </p>

      {/* Main Tabs */}
      <Tabs defaultValue="blinks" className="w-full mb-16">
        <TabsList className="grid w-full max-w-md mx-auto grid-cols-2 mb-8">
          <TabsTrigger value="blinks">Solana Blinks</TabsTrigger>
          <TabsTrigger value="tokens">Token Trading</TabsTrigger>
        </TabsList>

        {/* Blinks Tab Content */}
        <TabsContent value="blinks" className="space-y-16">
          {/* What are Blinks Section */}
          <section className="card p-8">
            <div className="grid md:grid-cols-2 gap-12 items-center">
              <div>
                <h2 className="text-2xl font-bold mb-4">What are Solana Blinks?</h2>
                <p className="text-gray-300 mb-4">
                  Solana Blinks are shareable links that allow anyone to send SOL to your wallet with just a few clicks.
                  They're perfect for creators, developers, and businesses who want to accept payments or donations
                  quickly and easily.
                </p>
                <p className="text-gray-300 mb-4">
                  Each Blink contains customizable information like a title, description, and image, making it easy to
                  communicate what the payment is for.
                </p>
                <p className="text-gray-300">
                  Blinks are powered by Solana Actions, a protocol that enables interactive transactions on the Solana
                  blockchain.
                </p>
              </div>
              <div className="flex justify-center">
                <div className="relative w-full max-w-sm overflow-hidden rounded-xl border border-white/10 bg-black/20 p-2 shadow-2xl">
                  <Image
                    src="/placeholder.svg?key=a3q8n"
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
          </section>

          {/* Creating Blinks Process */}
          <section>
            <h2 className="text-2xl font-bold mb-8 text-center">Creating and Sharing Blinks</h2>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  1
                </div>
                <div className="pt-6 text-center">
                  <Wallet className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Connect Your Wallet</h3>
                  <p className="text-gray-300">
                    Start by connecting your Solana wallet. We support Phantom, Solflare, and other popular wallets.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  2
                </div>
                <div className="pt-6 text-center">
                  <FileText className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Customize Your Blink</h3>
                  <p className="text-gray-300">
                    Add a title, description, and image to make your Blink stand out. Choose preset SOL amounts or allow
                    custom payments.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  3
                </div>
                <div className="pt-6 text-center">
                  <Share2 className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Share & Collect</h3>
                  <p className="text-gray-300">
                    Share your unique Blink link on social media, websites, or directly with your audience. Receive SOL
                    instantly when someone uses your Blink.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex justify-center mt-12">
              <Link href="/create" className="button-primary inline-flex items-center gap-2">
                Create Your First Blink
                <ArrowRight className="h-4 w-4" />
              </Link>
            </div>
          </section>

          {/* Use Cases */}
          <section className="card p-8">
            <h2 className="text-2xl font-bold mb-8 text-center">Popular Use Cases</h2>

            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="bg-black/30 rounded-xl p-6 border border-white/10">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/20 text-purple-400 mb-4">
                  <Zap className="h-6 w-6" />
                </div>
                <h3 className="text-lg font-bold mb-2">Content Creators</h3>
                <p className="text-gray-300 text-sm">
                  Accept tips and donations from your audience with a simple link in your bio or description.
                </p>
              </div>

              <div className="bg-black/30 rounded-xl p-6 border border-white/10">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/20 text-purple-400 mb-4">
                  <Shield className="h-6 w-6" />
                </div>
                <h3 className="text-lg font-bold mb-2">Developers</h3>
                <p className="text-gray-300 text-sm">
                  Add payment functionality to your apps and websites without complex integrations.
                </p>
              </div>

              <div className="bg-black/30 rounded-xl p-6 border border-white/10">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/20 text-purple-400 mb-4">
                  <Coins className="h-6 w-6" />
                </div>
                <h3 className="text-lg font-bold mb-2">Small Businesses</h3>
                <p className="text-gray-300 text-sm">
                  Accept crypto payments for products or services with customized payment links.
                </p>
              </div>

              <div className="bg-black/30 rounded-xl p-6 border border-white/10">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-purple-600/20 text-purple-400 mb-4">
                  <Share2 className="h-6 w-6" />
                </div>
                <h3 className="text-lg font-bold mb-2">Communities</h3>
                <p className="text-gray-300 text-sm">
                  Collect donations for community projects or fundraisers with transparent on-chain transactions.
                </p>
              </div>
            </div>
          </section>
        </TabsContent>

        {/* Tokens Tab Content */}
        <TabsContent value="tokens" className="space-y-16">
          {/* Token Trading Overview */}
          <section className="card p-8">
            <div className="grid md:grid-cols-2 gap-12 items-center">
              <div>
                <h2 className="text-2xl font-bold mb-4">Token Trading on SolBlinks</h2>
                <p className="text-gray-300 mb-4">
                  SolBlinks makes it easy to buy and sell Solana tokens with our user-friendly platform. Whether you're
                  looking to purchase tokens or create a shareable link to sell your own tokens, we've got you covered.
                </p>
                <p className="text-gray-300 mb-4">
                  Our platform offers low fees, fast transactions, and a seamless user experience. Plus, token sellers
                  can earn commissions on each transaction.
                </p>
                <div className="flex flex-col sm:flex-row gap-4 mt-6">
                  <Link href="/tokens" className="button-primary">
                    Buy Tokens
                  </Link>
                  <Link href="/sell-token" className="button-secondary">
                    Sell Tokens
                  </Link>
                </div>
              </div>
              <div className="flex justify-center">
                <div className="relative w-full max-w-sm overflow-hidden rounded-xl border border-white/10 bg-black/20 p-6 shadow-2xl">
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="relative w-10 h-10 rounded-full overflow-hidden bg-black/20">
                          <Image src="/solana-token-logo.png" alt="SOL" fill className="object-cover" />
                        </div>
                        <div>
                          <p className="font-medium">SOL</p>
                          <p className="text-xs text-gray-400">Solana</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-medium">$150.25</p>
                        <p className="text-xs text-green-400">+5.2%</p>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="relative w-10 h-10 rounded-full overflow-hidden bg-black/20">
                          <Image src="/placeholder.svg?key=u4t5h" alt="USDC" fill className="object-cover" />
                        </div>
                        <div>
                          <p className="font-medium">USDC</p>
                          <p className="text-xs text-gray-400">USD Coin</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-medium">$1.00</p>
                        <p className="text-xs text-green-400">+0.01%</p>
                      </div>
                    </div>

                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-3">
                        <div className="relative w-10 h-10 rounded-full overflow-hidden bg-black/20">
                          <Image src="/placeholder.svg?key=hcqa0" alt="BONK" fill className="object-cover" />
                        </div>
                        <div>
                          <p className="font-medium">BONK</p>
                          <p className="text-xs text-gray-400">Bonk</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <p className="font-medium">$0.00002</p>
                        <p className="text-xs text-red-400">-2.5%</p>
                      </div>
                    </div>

                    <div className="pt-4 mt-4 border-t border-white/10">
                      <button className="w-full py-3 rounded-lg bg-purple-600 hover:bg-purple-700 text-white font-medium">
                        Trade Now
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </section>

          {/* Buying Tokens Process */}
          <section>
            <h2 className="text-2xl font-bold mb-8 text-center">How to Buy Tokens</h2>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-green-600 text-white text-xl font-bold">
                  1
                </div>
                <div className="pt-6 text-center">
                  <Wallet className="h-12 w-12 mx-auto mb-4 text-green-400" />
                  <h3 className="text-xl font-bold mb-2">Connect Your Wallet</h3>
                  <p className="text-gray-300">
                    Connect your Solana wallet to access your funds and prepare for trading.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-green-600 text-white text-xl font-bold">
                  2
                </div>
                <div className="pt-6 text-center">
                  <Coins className="h-12 w-12 mx-auto mb-4 text-green-400" />
                  <h3 className="text-xl font-bold mb-2">Select Token & Amount</h3>
                  <p className="text-gray-300">
                    Browse available tokens, select the one you want to buy, and enter the amount.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-green-600 text-white text-xl font-bold">
                  3
                </div>
                <div className="pt-6 text-center">
                  <CheckCircle className="h-12 w-12 mx-auto mb-4 text-green-400" />
                  <h3 className="text-xl font-bold mb-2">Confirm & Complete</h3>
                  <p className="text-gray-300">
                    Review the transaction details, confirm with your wallet, and receive your tokens instantly.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex justify-center mt-12">
              <Link
                href="/tokens"
                className="button-primary inline-flex items-center gap-2 bg-green-600 hover:bg-green-700"
              >
                Start Buying Tokens
                <ArrowRight className="h-4 w-4" />
              </Link>
            </div>
          </section>

          {/* Selling Tokens Process */}
          <section>
            <h2 className="text-2xl font-bold mb-8 text-center">How to Sell Tokens</h2>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-8">
              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  1
                </div>
                <div className="pt-6 text-center">
                  <Wallet className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Connect Wallet</h3>
                  <p className="text-gray-300 text-sm">
                    Connect your Solana wallet that holds the tokens you want to sell.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  2
                </div>
                <div className="pt-6 text-center">
                  <FileText className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Enter Token Details</h3>
                  <p className="text-gray-300 text-sm">
                    Enter the token mint address and add a description to help buyers understand your offering.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  3
                </div>
                <div className="pt-6 text-center">
                  <RefreshCw className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Set Commission</h3>
                  <p className="text-gray-300 text-sm">
                    Optionally set a commission percentage (up to 1%) to earn from each transaction.
                  </p>
                </div>
              </div>

              <div className="card p-6 relative">
                <div className="absolute -top-6 left-1/2 -translate-x-1/2 flex h-12 w-12 items-center justify-center rounded-full bg-purple-600 text-white text-xl font-bold">
                  4
                </div>
                <div className="pt-6 text-center">
                  <Share2 className="h-12 w-12 mx-auto mb-4 text-purple-400" />
                  <h3 className="text-xl font-bold mb-2">Share Your Link</h3>
                  <p className="text-gray-300 text-sm">
                    Generate your unique token selling link and share it with potential buyers.
                  </p>
                </div>
              </div>
            </div>

            <div className="flex justify-center mt-12">
              <Link href="/sell-token" className="button-primary inline-flex items-center gap-2">
                Start Selling Tokens
                <ArrowRight className="h-4 w-4" />
              </Link>
            </div>
          </section>

          {/* Commission Explanation */}
          <section className="card p-8">
            <h2 className="text-2xl font-bold mb-6 text-center">Earn Commission on Token Sales</h2>
            <div className="grid md:grid-cols-2 gap-8 items-center">
              <div>
                <p className="text-gray-300 mb-4">
                  When you create a token selling link, you can set a commission percentage of up to 1% on each
                  transaction. This means you'll earn a portion of every purchase made through your link.
                </p>
                <p className="text-gray-300 mb-4">
                  For example, if you set a 0.5% commission and someone buys 1000 SOL worth of tokens through your link,
                  you'll automatically receive 5 SOL as commission.
                </p>
                <p className="text-gray-300">
                  This is perfect for influencers, community managers, or anyone looking to monetize their audience
                  while promoting tokens they believe in.
                </p>
              </div>
              <div className="bg-black/30 rounded-xl p-6 border border-white/10">
                <h3 className="text-xl font-bold mb-4 text-center">Commission Example</h3>
                <div className="space-y-4">
                  <div className="flex justify-between items-center border-b border-white/10 pb-2">
                    <span className="text-gray-300">Token Purchase Amount</span>
                    <span className="font-bold">1000 SOL</span>
                  </div>
                  <div className="flex justify-between items-center border-b border-white/10 pb-2">
                    <span className="text-gray-300">Commission Rate</span>
                    <span className="font-bold">0.5%</span>
                  </div>
                  <div className="flex justify-between items-center border-b border-white/10 pb-2">
                    <span className="text-gray-300">Your Commission</span>
                    <span className="font-bold text-green-400">5 SOL</span>
                  </div>
                  <div className="flex justify-between items-center">
                    <span className="text-gray-300">Buyer Receives</span>
                    <span className="font-bold">Tokens worth 995 SOL</span>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </TabsContent>
      </Tabs>

      {/* Security Section */}
      <section className="card p-8 mb-16">
        <h2 className="text-2xl font-bold mb-6 text-center">Security & Technical Details</h2>

        <div className="grid md:grid-cols-2 gap-8">
          <div>
            <h3 className="text-xl font-bold mb-4">How Solana Actions Work</h3>
            <p className="text-gray-300 mb-4">
              Solana Actions is a protocol that enables interactive transactions on the Solana blockchain. When you
              create a Blink or token selling link, we register it with a unique URL.
            </p>
            <p className="text-gray-300">
              When someone clicks on your link, our service checks if the link is registered and displays your
              information. When a user clicks a button in your Blink, it triggers a transaction that they can sign with
              their wallet.
            </p>
          </div>

          <div>
            <h3 className="text-xl font-bold mb-4">Security</h3>
            <p className="text-gray-300 mb-4">
              All transactions are secured by the Solana blockchain. Users always have full control over their funds and
              must explicitly approve any transaction with their wallet.
            </p>
            <p className="text-gray-300">
              Our service never has access to your private keys or funds. We simply facilitate the creation and sharing
              of Blinks and token trading links.
            </p>
          </div>
        </div>
      </section>

      {/* FAQ Section */}
      <section className="mb-16">
        <h2 className="text-2xl font-bold mb-8 text-center">Frequently Asked Questions</h2>

        <div className="grid md:grid-cols-2 gap-6">
          <div className="card p-6">
            <h3 className="text-xl font-bold mb-2">What wallets are supported?</h3>
            <p className="text-gray-300">
              We support most popular Solana wallets including Phantom, Solflare, Backpack, and more. Any wallet that
              supports Solana Actions should work with our platform.
            </p>
          </div>

          <div className="card p-6">
            <h3 className="text-xl font-bold mb-2">Are there any fees?</h3>
            <p className="text-gray-300">
              Creating Blinks is free. When using Blinks or trading tokens, you'll only pay the standard Solana network
              transaction fees, which are typically less than $0.01.
            </p>
          </div>

          <div className="card p-6">
            <h3 className="text-xl font-bold mb-2">How do I track my earnings?</h3>
            <p className="text-gray-300">
              You can view all transactions associated with your Blinks in the "My Blinks" section. For token sales, you
              can track commissions through your wallet's transaction history.
            </p>
          </div>

          <div className="card p-6">
            <h3 className="text-xl font-bold mb-2">Can I customize the appearance of my Blinks?</h3>
            <p className="text-gray-300">
              Yes, you can add a custom title, description, and image to your Blinks. For token selling links, the token
              information is automatically fetched from the blockchain.
            </p>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="card p-8 text-center">
        <h2 className="text-2xl font-bold mb-4">Ready to Get Started?</h2>
        <p className="text-gray-300 mb-6 max-w-2xl mx-auto">
          Create your first Blink or token trading link in minutes. Connect your wallet and start accepting SOL or
          trading tokens today.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link href="/create" className="button-primary inline-flex items-center gap-2">
            Create a Blink
            <ArrowRight className="h-4 w-4" />
          </Link>
          <Link href="/tokens" className="button-secondary inline-flex items-center gap-2">
            Buy Tokens
          </Link>
          <Link href="/sell-token" className="button-secondary inline-flex items-center gap-2">
            Sell Tokens
          </Link>
        </div>
      </section>
    </div>
  )
}
