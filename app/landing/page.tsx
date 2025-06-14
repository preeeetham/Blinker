"use client"
import { useState, useEffect } from "react"
import { useWallet } from "@solana/wallet-adapter-react"
import { useRouter } from "next/navigation"
import { WalletButton } from "@/components/solana/solana-provider"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import Image from "next/image"
import {
  HiOutlineCash,
  HiOutlineShoppingCart,
  HiOutlineCollection,
  HiOutlineGlobe,
  HiOutlineLightningBolt,
  HiOutlineShieldCheck,
  HiArrowRight,
} from "react-icons/hi"
import { FaTwitter, FaGithub, FaDiscord } from "react-icons/fa"

export default function LandingPage() {
  const { connected } = useWallet()
  const router = useRouter()
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    setIsVisible(true)
    if (connected) {
      router.push("/")
    }
  }, [connected, router])

  const features = [
    {
      icon: <HiOutlineCash className="w-8 h-8" />,
      title: "Request SOL",
      description: "Create shareable Blinks to receive SOL payments from anyone, anywhere.",
    },
    {
      icon: <HiOutlineShoppingCart className="w-8 h-8" />,
      title: "Sell/Resell Tokens",
      description: "Generate Blinks to sell or resell any SPL token with custom commission rates.",
    },
    {
      icon: <HiOutlineCollection className="w-8 h-8" />,
      title: "Manage Your Blinks",
      description: "Keep track of all your created Blinks in one organized dashboard.",
    },
  ]

  const benefits = [
    {
      icon: <HiOutlineLightningBolt className="w-6 h-6" />,
      title: "Lightning Fast",
      description: "Create and share Blinks in seconds",
    },
    {
      icon: <HiOutlineShieldCheck className="w-6 h-6" />,
      title: "Secure & Reliable",
      description: "Built on Solana's robust blockchain",
    },
    {
      icon: <HiOutlineGlobe className="w-6 h-6" />,
      title: "Universal Sharing",
      description: "Share anywhere - Twitter, Discord, or any platform",
    },
  ]

  return (
    <div className="min-h-screen bg-[var(--bg-color)] text-[var(--text-color)] overflow-x-hidden">
      {/* Hero Section */}
      <section className="relative min-h-screen flex items-center justify-center px-4 py-10">
        <div className="absolute inset-0 bg-gradient-to-br from-[var(--accent-primary)]/5 to-[var(--accent-secondary)]/5"></div>

        <div
          className={`relative z-10 max-w-6xl mx-auto text-center transition-all duration-1000 ${
            isVisible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-10"
          }`}
        >

          {/* Main Heading - Bold BLINKER Style */}
          <div className="mb-8">
            <h1 className="text-8xl md:text-9xl lg:text-[12rem] xl:text-[15rem] font-black tracking-tighter leading-none mb-4 select-none">
              <span className="gradient-text bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] bg-clip-text text-transparent gap-3">
                BLINKER
              </span>
            </h1>
          </div>

          {/* Subtitle */}
          <div className="space-y-4 mb-12">
            <h2 className="text-2xl md:text-3xl lg:text-4xl font-semibold text-[var(--text-color)] mb-4">
              Create Powerful Solana Blinks
            </h2>
            <p className="text-lg md:text-xl text-[var(--text-secondary)] max-w-4xl mx-auto leading-relaxed">
              Transform your Solana transactions into shareable, interactive Blinks. Request payments, sell tokens, and
              engage your community like never before.
            </p>
          </div>

          {/* CTA Section */}
          <div className="space-y-6">
            <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
              {!connected ? (
                <>
                  <p className="text-lg text-[var(--text-secondary)] mb-2">Ready to get started?</p>
                  <div className="pulse">
                    <WalletButton />
                  </div>
                </>
              ) : (
                <Button onClick={() => router.push("/")} className="button-primary text-lg px-8 py-4 group">
                  Go Create Your Blinks
                  <HiArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
                </Button>
              )}
            </div>

            <p className="text-sm text-[var(--text-secondary)]">Connect your wallet to start creating amazing Blinks</p>
          </div>

          {/* Floating Elements */}
          <div className="absolute top-20 left-10 w-20 h-20 bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] rounded-full opacity-20 blur-xl floating animation-delay-100"></div>
          <div className="absolute bottom-20 right-10 w-32 h-32 bg-gradient-to-r from-[var(--accent-secondary)] to-[var(--accent-primary)] rounded-full opacity-10 blur-2xl floating animation-delay-300"></div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4 gradient-text">Everything You Need</h2>
            <p className="text-xl text-[var(--text-secondary)] max-w-2xl mx-auto">
              Powerful features to create, manage, and share your Solana Blinks
            </p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {features.map((feature, index) => (
              <Card
                key={index}
                className={`card p-8 text-center hover-lift fade-in animation-delay-${(index + 1) * 100} group`}
              >
                <div className="mb-6 flex justify-center">
                  <div className="p-4 rounded-2xl bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white group-hover:scale-110 transition-transform duration-300">
                    {feature.icon}
                  </div>
                </div>
                <h3 className="text-xl font-semibold mb-4 text-[var(--text-color)]">{feature.title}</h3>
                <p className="text-[var(--text-secondary)] leading-relaxed">{feature.description}</p>
              </Card>
            ))}
          </div>
        </div>
      </section>

      {/* Benefits Section */}
      <section className="py-20 px-4 bg-gradient-to-r from-[var(--accent-primary)]/5 to-[var(--accent-secondary)]/5">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4 gradient-text">Why Choose Blinker?</h2>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {benefits.map((benefit, index) => (
              <div key={index} className={`text-center fade-in animation-delay-${(index + 1) * 200}`}>
                <div className="mb-4 flex justify-center">
                  <div className="p-3 rounded-xl bg-[var(--card-bg)] border border-[var(--border-color)] text-[var(--accent-primary)]">
                    {benefit.icon}
                  </div>
                </div>
                <h3 className="text-lg font-semibold mb-2 text-[var(--text-color)]">{benefit.title}</h3>
                <p className="text-[var(--text-secondary)]">{benefit.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* How It Works Section */}
      <section className="py-20 px-4">
        <div className="max-w-4xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold mb-4 gradient-text">How It Works</h2>
            <p className="text-xl text-[var(--text-secondary)]">Create your first Blink in just 3 simple steps</p>
          </div>

          <div className="grid md:grid-cols-3 gap-8">
            {[
              {
                step: "01",
                title: "Connect Wallet",
                description: "Connect your Solana wallet to get started",
              },
              {
                step: "02",
                title: "Customize Blink",
                description: "Add your title, description, and image",
              },
              {
                step: "03",
                title: "Share & Earn",
                description: "Share your Blink anywhere and start receiving payments",
              },
            ].map((step, index) => (
              <div key={index} className={`text-center fade-in animation-delay-${(index + 1) * 100}`}>
                <div className="mb-6">
                  <div className="w-16 h-16 mx-auto rounded-full bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] flex items-center justify-center text-white font-bold text-xl">
                    {step.step}
                  </div>
                </div>
                <h3 className="text-xl font-semibold mb-3 text-[var(--text-color)]">{step.title}</h3>
                <p className="text-[var(--text-secondary)]">{step.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 px-4 bg-gradient-to-r from-[var(--accent-primary)]/10 to-[var(--accent-secondary)]/10">
        <div className="max-w-4xl mx-auto text-center">
          <h2 className="text-3xl md:text-4xl font-bold mb-6 gradient-text">Ready to Create Your First Blink?</h2>
          <p className="text-xl text-[var(--text-secondary)] mb-8 max-w-2xl mx-auto">
            Join the future of Solana interactions. Create, share, and monetize your content with powerful Blinks.
          </p>

          {!connected ? (
            <div className="space-y-4">
              <div className="pulse">
                <WalletButton />
              </div>
              <p className="text-sm text-[var(--text-secondary)]">Connect your wallet to get started</p>
            </div>
          ) : (
            <Button onClick={() => router.push("/")} className="button-primary text-lg px-8 py-4 group">
              Start Creating Now
              <HiArrowRight className="ml-2 w-5 h-5 group-hover:translate-x-1 transition-transform" />
            </Button>
          )}
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 px-4 border-t border-[var(--border-color)]">
        <div className="max-w-6xl mx-auto">
          <div className="flex flex-col md:flex-row items-center justify-between gap-6">
            <div className="flex items-center gap-4">
              <Image src="/blinker-logo.png" width={120} height={80} alt="Blinker Logo" className="rounded-lg" />
              <div>
                <h3 className="font-semibold text-[var(--text-color)]">Blinker</h3>
                <p className="text-sm text-[var(--text-secondary)]">Powering Solana Blinks</p>
              </div>
            </div>

            <div className="flex items-center gap-6">
              <a
                href="https://github.com/preeeetham/Blinker"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--text-secondary)] hover:text-[var(--accent-primary)] transition-colors duration-300"
                aria-label="GitHub"
              >
                <FaGithub size={24} />
              </a>
              <a
                href="https://discord.com/channels/@me"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--text-secondary)] hover:text-[var(--accent-primary)] transition-colors duration-300"
                aria-label="Discord"
              >
                <FaDiscord size={24} />
              </a>
              <a
                href="https://x.com/0xgravitty"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--text-secondary)] hover:text-[var(--accent-primary)] transition-colors duration-300"
                aria-label="Twitter"
              >
                <FaTwitter size={24} />
              </a>
            </div>
          </div>

        </div>
      </footer>
    </div>
  )
}