import type React from "react"
import "./global.css"
import { AppLayout } from "@/ui-kit/primitives/ui/app-layout"
import { ClusterProvider } from "@/integrations/cluster/cluster-data-access"
import { SolanaProvider } from "@/integrations/solana/solana-provider"
import { GlobalTitleProvider } from "./GlobalStateContext"
import { ErrorBoundary } from "@/ui-kit/primitives/ui/error-boundary"

export const metadata = {
  title: "Blinker",
  description: "Create Powerful Solana Blinks - Transform your transactions into shareable, interactive experiences",
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" dir="ltr" className="scroll-smooth antialiased" suppressHydrationWarning>
      <head>
        {/* Theme-aware fallback background to prevent white/black flash */}
        <style>{`
          html, body {
            background-color: white;
            color: black;
          }

          @media (prefers-color-scheme: dark) {
            html, body {
              background-color: #000000;
              color: #f8fafc;
            }
          }
        `}</style>
      </head>
      <body className="bg-[var(--bg-color)] text-[var(--text-color)] overflow-x-hidden transition-colors duration-300 min-h-screen">
        <ErrorBoundary>
          <GlobalTitleProvider>
            <ClusterProvider>
              <SolanaProvider>
                {/* Check if we're on the landing page */}
                {typeof window !== "undefined" && window.location.pathname === "/landing" ? (
                  children
                ) : (
                  <AppLayout>{children}</AppLayout>
                )}
              </SolanaProvider>
            </ClusterProvider>
          </GlobalTitleProvider>
        </ErrorBoundary>
      </body>
    </html>
  )
}
