"use client";
import { ReactNode, useEffect, useState } from 'react';
import { WalletButton } from '../solana/solana-provider';
import { useWallet } from '@solana/wallet-adapter-react';
import Link from 'next/link';
import { Footer } from '@/components/footer';
import { usePathname } from 'next/navigation';
import ThemeToggle from './theme-toggle';
import { useGlobalTitleState } from '@/app/GlobalStateContext';
import { FaInfo } from 'react-icons/fa';
import { HiInformationCircle } from 'react-icons/hi';
import { Pointer } from 'lucide-react';
import * as Tooltip from '@radix-ui/react-tooltip';

export function AppLayout({ children }: { children: ReactNode }) {
  const { value, info } = useGlobalTitleState();
  const [isTooltipOpen, setIsTooltipOpen] = useState(false);
  const { connected } = useWallet();
  const [activePath, setActivePath] = useState<string>('');
  const pathname = usePathname();

  // set active nav based on location safely in effect
  useEffect(() => {
    if (pathname) setActivePath(pathname);
  }, [pathname]);

  // If wallet is not connected, show only the main content (landing page)
  if (!connected) {
    return (
      <div className="h-full w-full">
        {children}
      </div>
    );
  }

  // If wallet is connected, show full layout with top header nav
  return (
    <div className="flex flex-col min-h-screen">
      {/* Top Header */}
      <header className="sticky top-0 z-40 w-full border-b border-[var(--border-color)] bg-[var(--card-bg)]/90 backdrop-blur-md">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-3 flex items-center justify-between">
          {/* Brand / Title with info */}
          <div className="flex items-center gap-2">
            <h1 className="Title bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-transparent bg-clip-text">
              {value}
            </h1>
            <Tooltip.Provider delayDuration={0}>
              <Tooltip.Root open={isTooltipOpen} onOpenChange={setIsTooltipOpen}>
                <Tooltip.Trigger asChild>
                  <span
                    className="inline-block ml-1"
                    onClick={() => setIsTooltipOpen(!isTooltipOpen)}
                  >
                    <HiInformationCircle color='#A48BFA' cursor='pointer' size={22} />
                  </span>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="bg-[var(--card-bg)] text-[var(--text-color)] px-3 py-1.5 rounded-md text-sm shadow-md max-w-[31rem] max-sm:max-w-[20rem]"
                    sideOffset={1}
                  >
                    {info}
                    <Tooltip.Arrow className="fill-[var(--card-bg)]" />
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>
            </Tooltip.Provider>
          </div>

          {/* Nav */}
          <nav className="hidden md:flex items-center gap-2">
            <Link href="/token" className={`px-3 py-2 rounded-lg transition-all duration-200 ${activePath === '/token' ? 'bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white shadow-md' : 'text-[var(--text-secondary)] hover:bg-[var(--hover-bg)]'}`}>Sell/Resell Tokens</Link>
            <Link href="/Blinks" className={`px-3 py-2 rounded-lg transition-all duration-200 ${activePath === '/Blinks' ? 'bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white shadow-md' : 'text-[var(--text-secondary)] hover:bg-[var(--hover-bg)]'}`}>My Blinks</Link>
          </nav>

          {/* Actions */}
          <div className="flex items-center gap-3">
            <ThemeToggle />
            <WalletButton />
          </div>
        </div>
      </header>

      {/* Content Shell - simplified for natural page scrolling */}
      <main className="flex-1 w-full">
        <div className="max-w-7xl mx-auto px-4 md:px-8 py-6 md:py-8">
          {children}
        </div>
      </main>
      <Footer />
    </div>
  );
}