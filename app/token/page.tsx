"use client";
import { useState, useRef, useEffect } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import TokenPreview from "@/components/preview/token-preview";
import TokenForm from '@/components/form/tokenForm';
import {
  PublicKey,
  Transaction,
} from '@solana/web3.js';
import { HiOutlineClipboardCopy, HiOutlineShare, HiOutlinePlus } from 'react-icons/hi';
import { Footer } from '@/components/footer';
import { confirmTransaction } from '@/server/transaction';

type CommissionType = "yes" | "no";

export default function Page() {
  const { publicKey, connected, sendTransaction } = useWallet();
  const [icon, setIcon] = useState<string>('');
  const [percentage, setPercentage] = useState<number>(0);
  const [takeCommission, setTakeCommission] = useState<CommissionType>("no");
  const [description, setDescription] = useState<string>('');
  const [title, setTitle] = useState<string>('');
  const [mint, setMint] = useState<string>('');
  const [showPreview, setShowPreview] = useState(false);
  const [loading, setLoading] = useState(false);
  const [loadingText, setLoadingText] = useState('Please Wait!!');
  const [showForm, setShowForm] = useState(true);
  const [blinkLink, setBlinkLink] = useState('');
  const [copied, setCopied] = useState(false);
  const form = useRef<HTMLDivElement | null>(null);
  const { connection } = useConnection();

  useEffect(() => {
    setShowPreview(false);
  }, [mint]);

  useEffect(() => {
    if (takeCommission === "no") {
      setPercentage(0);
    }
  }, [takeCommission]);

  const handleSubmit = async () => {
    setLoading(true);
    setLoadingText('Waiting for Transaction confirmation!!');
    try {
      if (!connected || !publicKey) {
        window.alert('Please connect your wallet first');
        return;
      }

      if (!description || !mint) {
        window.alert('Please fill all fields');
        return;
      }

      const response = await fetch('/api/actions/generate-blink/token', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          label: 'Buy Token',
          description,
          wallet: publicKey.toString(),
          mint,
          commission: takeCommission,
          percentage: percentage,
        }),
      });

      if (!response.ok) throw new Error('Failed to generate blink');

      const data = await response.json();
      const getTransaction = await data.transaction;
      const { serializedTransaction, blockhash, lastValidBlockHeight } = getTransaction;

      const transaction = Transaction.from(Buffer.from(serializedTransaction, 'base64'));
      const signature = await sendTransaction(transaction, connection);

      const confirmation = await confirmTransaction(signature, blockhash, lastValidBlockHeight);

      const res = await fetch('/api/actions/order', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          signature,
          orderId: data.id.toString(),
        }),
      });

      const link = await res.json();
      if (!link.blinkLink) throw new Error('Failed to generate blink');

      setBlinkLink(link.blinkLink);
      setShowForm(false);
      setLoading(false);
    } catch (error) {
      console.error('Error in handleSubmit:', error);
      window.alert('Transaction failed or blink generation error. Please try again.');
      setLoading(false);
    }
  };

  const handlePreview = async () => {
    try {
      setLoading(true);
      setLoadingText('Generating Blink Preview!!');
      if (!connected || !publicKey) {
        window.alert('Please connect your wallet');
        return;
      }

      if (!description || !mint) {
        window.alert('Please fill all fields');
        return;
      }

      const response = await fetch('/api/actions/generate-blink/token?mint=' + mint);
      if (!response.ok) throw new Error('Failed to generate preview');

      const data = await response.json();
      setShowPreview(true);
      setIcon(data.icon);
      setTitle(data.title);
      setLoading(false);
    } catch (err) {
      console.error(err);
      window.alert("Invalid Mint Address!!");
      setLoading(false);
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(`https://dial.to/?action=solana-action:${blinkLink}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleTweet = () => {
    const tweetText = `Check out this Blink I just made using @getblinkdotfun: https://dial.to/?action=solana-action:${blinkLink}`;
    const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`;
    window.open(twitterUrl, '_blank');
  };

  const handleNew = () => {
    setShowForm(true);
  };

  return (
    <div className="flex flex-col md:min-h-screen relative">
      {/* Inline loading screen */}
      {loading && (
        <div className="absolute inset-0 z-50 bg-black/60 flex items-center justify-center">
          <div className="bg-white dark:bg-gray-900 rounded-xl p-6 shadow-lg flex flex-col items-center space-y-4">
            <svg
              className="animate-spin h-8 w-8 text-[var(--accent-primary)]"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8v8H4z"
              />
            </svg>
            <p className="text-sm text-[var(--text-secondary)]">{loadingText}</p>
          </div>
        </div>
      )}

      <div className="flex-1 flex flex-col md:flex-row items-center md:items-start md:justify-center gap-8 md:p-8">
        <div className="w-full max-w-2xl">
          <div className="md:card md:p-10" ref={form}>
            {showForm ? (
              <TokenForm
                mint={mint}
                description={description}
                percentage={percentage}
                takeCommission={takeCommission}
                showPreview={showPreview}
                setMint={setMint}
                setDescription={setDescription}
                setPercentage={setPercentage}
                setTakeCommission={setTakeCommission}
                handlePreview={handlePreview}
                handleSubmit={handleSubmit}
                connected={connected}
                publicKey={publicKey}
              />
            ) : (
              <div className="space-y-6">
                <h1 className="text-2xl md:text-3xl font-bold mb-6 text-gradient bg-gradient-to-r from-[var(--gradient-start)] to-[var(--gradient-end)] bg-clip-text text-transparent">
                  Your Blink is Ready!
                </h1>

                <div className="p-4 rounded-xl bg-[var(--card-bg)] border border-[var(--border-color)]">
                  <p className="text-sm text-[var(--text-secondary)] mb-2">Blink Link:</p>
                  <div className="flex items-center gap-2">
                    <div
                      className="flex-1 p-3 bg-[rgba(0,0,0,0.2)] rounded-lg text-sm overflow-hidden overflow-ellipsis whitespace-nowrap cursor-pointer"
                      onClick={() => {
                        window.open(`https://dial.to/?action=solana-action:${blinkLink}`, '_blank');
                      }}
                    >
                      https://dial.to/?action=solana-action:{blinkLink}
                    </div>
                    <button
                      onClick={handleCopy}
                      className="p-3 rounded-lg bg-[var(--border-color)] hover:bg-[var(--accent-primary)] transition-colors duration-300"
                      title="Copy to clipboard"
                    >
                      {copied ? 'Copied!' : <HiOutlineClipboardCopy size={20} />}
                    </button>
                  </div>
                </div>

                <div className="flex flex-col sm:flex-row gap-4 mt-6">
                  <button
                    className="button-primary flex-1 flex items-center justify-center gap-2"
                    onClick={handleTweet}
                  >
                    <HiOutlineShare size={18} />
                    Share on X
                  </button>

                  <button
                    className="button-secondary flex-1 flex items-center justify-center gap-2"
                    onClick={handleNew}
                  >
                    <HiOutlinePlus size={18} />
                    Create New Blink
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>

        {showForm && (
          <div className="w-full md:w-auto flex justify-center">
            <TokenPreview
              icon={icon || 'solana.jpg'}
              description={description || 'Your Description shows up here, Keep it short and simple'}
              title={title || "Your Title"}
            />
          </div>
        )}
      </div>

      <Footer />
    </div>
  );
}
