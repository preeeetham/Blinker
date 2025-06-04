"use client";
import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import DataCard from '../../components/DataCard/dataCard';
import { WalletButton } from '@/components/solana/solana-provider';
import { Footer } from '@/components/footer';

export default function Page() {
  const { publicKey } = useWallet();
  const [data, setData] = useState<Array<Record<string, any>>>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const getBlinks = async () => {
      if (!publicKey) return;

      try {
        const response = await fetch('/api/actions/getBlinks?wallet=' + publicKey.toString());
        const { blinks } = await response.json();
        setData(blinks);
        console.log('Blinks:', blinks);
        setLoading(false);
      } catch (error) {
        setLoading(false);
        console.error('Error fetching blinks:', error);
      }
    };

    if (publicKey) {
      getBlinks();
    } else {
      setLoading(false);
    }
  }, [publicKey]);

  return (
    <div className="flex flex-col min-h-screen">
      <main className="flex-grow sm:px-6 md:px-8 md:py-8 py-2 max-w-7xl mx-auto w-full">
        <div className="glass-card w-full max-w-4xl mx-auto overflow-hidden fade-in">
          <div className="sm:p-6 md:p-8">
            <h1 className="text-2xl sm:text-3xl md:text-4xl font-bold mb-2 md:mb-6 text-center gradient-text">
              Your Blinks
            </h1>

            {!publicKey ? (
              <div className="flex flex-col items-center justify-center py-10 space-y-4 fade-in">
                <p className="text-[var(--text-secondary)] text-center mb-4">
                  Connect your wallet to view your Blinks
                </p>
                <div className="pulse-subtle">
                  <WalletButton />
                </div>
              </div>
            ) : loading ? (
              <div className="flex justify-center items-center py-10">
                <div className="shimmer w-full max-w-md h-40 rounded-xl"></div>
              </div>
            ) : (
              <div className="space-y-4 max-h-[70vh] overflow-y-auto minimal-scrollbar pr-1">
                {data && data.length > 0 ? (
                  data.slice().reverse().map((blink, index) => (
                    <div
                      key={blink['_id']}
                      className="hover-lift fade-in mt-1"
                      style={{ animationDelay: `${index * 0.1}s` }}
                    >
                      <DataCard
                        code={blink['_id']}
                        base={"https://dial.to/?action=solana-action:"}
                        title={blink.title || `Open a ${blink.poolName} Position`}
                        endpoint={blink.endpoint}
                      />
                    </div>
                  ))
                ) : (
                  <div className="text-center py-8 fade-in">
                    <p className="text-[var(--text-secondary)]">No Blinks found</p>
                    <p className="text-sm mt-2 text-[var(--text-secondary)]">Create your first Blink to get started</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </main>
      <Footer />
    </div>
  );
}
