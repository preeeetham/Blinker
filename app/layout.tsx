import type React from "react"
import { ThemeProvider } from "@/components/theme-provider"
import { ClusterProvider } from "@/components/cluster/cluster-data-access"
import { SolanaProvider } from "@/components/solana/solana-provider"
import { StarryBackground } from "@/components/ui/starry-background"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { Inter } from "next/font/google"
import "./globals.css"
import type { Metadata } from "next"

const inter = Inter({ subsets: ["latin"] })

export const metadata: Metadata = {
  title: "Solana Actions - Create Shareable Blinks",
  description: "Create and share Solana Actions Blinks with your audience",
    generator: 'v0.dev'
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body className={`${inter.className} dark`}>
        <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false} disableTransitionOnChange>
          <ClusterProvider>
            <SolanaProvider>
              <div className="relative min-h-screen flex flex-col">
                <StarryBackground />
                <Navbar />
                <main className="flex-grow container mx-auto px-4 py-8 relative z-10">{children}</main>
                <Footer />
              </div>
            </SolanaProvider>
          </ClusterProvider>
        </ThemeProvider>
      </body>
    </html>
  )
}
