import { ReactNode } from 'react';
import Link from 'next/link';
import { WalletButton } from './solana/solana-provider';
import { Toaster } from '@/components/ui/toaster';

export function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="flex flex-col min-h-screen bg-[var(--bg-color)]">
      <header className="border-b border-[var(--border-color)] py-4 px-6">
        <div className="container mx-auto flex items-center justify-between">
          <div>
            <Link href="/">
              <h1 className='Title'>Blink Generator</h1>
            </Link>
          </div>
          <div>
            <WalletButton />
          </div>
        </div>
      </header>
      <main className="flex-grow container mx-auto py-6 px-4">{children}</main>
      <Toaster />
    </div>
  );
}
