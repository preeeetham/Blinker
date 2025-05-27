'use client';

import dynamic from 'next/dynamic';

import { WalletError } from '@solana/wallet-adapter-base';
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import { ReactNode, useCallback, useEffect, useMemo, useState } from 'react';
import { useCluster } from '../cluster/cluster-data-access';
import { LedgerWalletAdapter, PhantomWalletAdapter, SolflareWalletAdapter } from "@solana/wallet-adapter-wallets";

require('@solana/wallet-adapter-react-ui/styles.css');

export const WalletButton = dynamic(
  async () =>
    (await import('@solana/wallet-adapter-react-ui')).WalletMultiButton,
  { ssr: false }
);

export function SolanaProvider({ children }: { children: ReactNode }) {
  const { cluster } = useCluster();
  const [endpointUrl, setEndpointUrl] = useState<string>("");
  useEffect(() => {
    if (typeof window !== "undefined") {
      const origin = window.location.origin;
      setEndpointUrl(`${origin}/api/rpc`);
    }
  }, []);

  const wallets = useMemo(() => [
    new PhantomWalletAdapter,
    new SolflareWalletAdapter,
    new LedgerWalletAdapter
  ], []);

  if (!endpointUrl) {
    return null;
  }

  return (
    <ConnectionProvider endpoint={endpointUrl}>
      <WalletProvider wallets={wallets} autoConnect={true}>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
