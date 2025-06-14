'use client';

import React, { useState, useRef } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { WalletButton } from '../solana/solana-provider';
import {
  PublicKey,
  Transaction,
} from '@solana/web3.js';
import { HiOutlineClipboardCopy, HiOutlineShare, HiExclamation, HiX } from 'react-icons/hi';
import { confirmTransaction } from '@/server/transaction';
import { Button } from '../ui/button';

interface FormProps {
  icon: string;
  setIcon: (value: string) => void;
  description: string;
  setDescription: (value: string) => void;
  title: string;
  setTitle: (value: string) => void;
  showForm: boolean;
  setShowForm: (value: boolean) => void;
}

interface ErrorPopupProps {
  message: string;
  isVisible: boolean;
  onClose: () => void;
}

const ErrorPopup: React.FC<ErrorPopupProps> = ({ message, isVisible, onClose }) => {
  if (!isVisible) return null

  return (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center p-4">
      <div className="bg-red-50 border border-red-300 rounded-lg p-4 max-w-md w-full shadow-lg animate-in fade-in duration-200">
        <div className="flex items-start">
          <div className="flex-shrink-0">
            <HiExclamation className="h-5 w-5 text-red-400" />
          </div>
          <div className="ml-3 flex-1">
            <h3 className="text-sm font-medium text-red-800">
              Error Occurred
            </h3>
            <div className="mt-2 text-sm text-red-700">
              <p>{message}</p>
            </div>
          </div>
          <div className="ml-auto pl-3">
            <div className="-mx-1.5 -my-1.5">
              <button
                onClick={onClose}
                className="inline-flex rounded-md bg-red-50 p-1.5 text-red-500 hover:bg-red-100 focus:outline-none focus:ring-2 focus:ring-red-600 focus:ring-offset-2 focus:ring-offset-red-50"
              >
                <HiX className="h-5 w-5" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}

const Form: React.FC<FormProps> = ({
  icon,
  setIcon,
  description,
  setDescription,
  title,
  setTitle,
  showForm,
  setShowForm
}) => {
  const { publicKey, connected, sendTransaction } = useWallet();
  const [blinkLink, setBlinkLink] = useState('');
  const [copied, setCopied] = useState(false);
  const form = useRef<HTMLDivElement | null>(null);
  const [loading, setLoading] = useState(false);
  const { connection } = useConnection();

  const [errorMessage, setErrorMessage] = useState<string>("");
  const [showError, setShowError] = useState<boolean>(false);

  const showErrorPopup = (message: string): void => {
    setErrorMessage(message);
    setShowError(true);
  };

  const closeErrorPopup = (): void => {
    setShowError(false);
    setErrorMessage("");
  };

  const renderLoading = () => (
    <div className="fixed inset-0 z-50 bg-black bg-opacity-50 flex items-center justify-center">
      <div className="text-white text-lg font-semibold p-4 rounded-lg bg-[var(--card-bg)] border border-[var(--border-color)] shadow-lg">
        Waiting For Transaction Confirmation...
      </div>
    </div>
  );

  const handlePreview = async () => {
    setLoading(true);
    if (!connected || !publicKey) {
      console.error('Wallet not connected');
      showErrorPopup("Please connect your wallet first");
      setLoading(false);
      return;
    }

    if (!icon || !description || !title) {
      console.error('Please fill all fields');
      showErrorPopup('Please fill all fields');
      setLoading(false);
      return;
    }

    let BlinkData;
    try {
      const walletAddress = publicKey.toString();
      const response = await fetch('/api/actions/generate-blink', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          icon,
          label: 'donate Sol',
          description,
          title,
          wallet: walletAddress,
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to generate blink');
      }

      BlinkData = await response.json();
    } catch (error) {
      setLoading(false);
      console.error('Failed to generate blink', error);
      showErrorPopup('Failed to generate blink');
      return;
    }

    const getTransaction = BlinkData.transaction;
    const { serializedTransaction, blockhash, lastValidBlockHeight } = getTransaction;
    const transaction = Transaction.from(Buffer.from(serializedTransaction, 'base64'));

    try {
      const signature = await sendTransaction(transaction, connection);
      console.log('Transaction sent:', signature);

      const confirmation = await confirmTransaction(
        signature,
        blockhash,
        lastValidBlockHeight
      );
      console.log('Transaction confirmed:', confirmation);

      const res = await fetch('/api/actions/order', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          signature,
          orderId: BlinkData.id.toString(),
        }),
      });

      const link = await res.json();
      if (!link.blinkLink) {
        throw new Error('Failed to generate blink');
      }

      setBlinkLink(link.blinkLink);
      setShowForm(false);
      setLoading(false);
    } catch (error) {
      setLoading(false);
      console.error('Failed to send transaction', error);
      showErrorPopup('Failed to send transaction');
      return;
    }
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(`https://dial.to/?action=solana-action:${blinkLink}`);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  const handleTweet = () => {
    const tweetText = `Check out this Blink I just made using blinkgend(dot)xyz : https://dial.to/?action=solana-action:${blinkLink}`;
    const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`;
    window.open(twitterUrl, '_blank');
  };

  const handleNew = () => {
    setShowForm(true);
  };

  return (
    <div className="w-full max-w-2xl h-full">
      {loading && renderLoading()}
      
      <ErrorPopup 
        message={errorMessage}
        isVisible={showError}
        onClose={closeErrorPopup}
      />

      <div 
        className="md:card md:p-10 h-full transition-all duration-300 ease-in-out hover:shadow-[0_0_20px_rgba(0,0,255,0.5)] hover:scale-[1.02] rounded-xl" 
        ref={form}
      >
        {showForm && (
          <div className="space-y-6 h-full">
            <h1 className="text-2xl md:text-3xl font-bold mb-6 gradient-text">
              Customize Your Blink
            </h1>

            <div>
              <label className="block text-sm font-medium mb-2 text-[var(--text-secondary)]">Title</label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="input-field"
                placeholder="Enter a title for your Blink"
                maxLength={50}
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2 text-[var(--text-secondary)]">Image URL</label>
              <input
                type="text"
                value={icon}
                onChange={(e) => setIcon(e.target.value)}
                className="input-field"
                placeholder="Enter image URL"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-2 text-[var(--text-secondary)]">Description</label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                className="textarea-field"
                rows={3}
                placeholder="Enter a description"
                maxLength={143}
              />
              <p className="text-xs text-[var(--text-secondary)] mt-1">
                {description.length}/143 characters
              </p>
            </div>

            {publicKey ? (
              <Button
                className="py-3 px-6 rounded-xl font-medium cursor-pointer transition-all duration-300 bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white hover:opacity-90 active:scale-95 flex items-center justify-center gap-2 shadow-md w-full mt-4 text-lg"
                onClick={handlePreview}
                disabled={!connected || !title || !icon || !description || title.length <= 0 || icon.length <= 0}
              >
                Generate Blink
              </Button>
            ) : (
              <div className="mt-4 text-center">
                <p className="text-[var(--text-secondary)] mb-3">Connect your wallet to generate a Blink</p>
                <WalletButton />
              </div>
            )}
          </div>
        )}

        {!showForm && (
          <div className="space-y-6">
            <h1 className="text-2xl md:text-3xl font-bold mb-6 gradient-text">
              Your Blink is Ready!
            </h1>

            <div className="p-4 rounded-xl bg-[var(--card-bg)] border border-[var(--border-color)]">
              <p className="text-sm text-[var(--text-secondary)] mb-2">Blink Link:</p>
              <div className="flex items-center gap-2">
                <div
                  className="flex-1 p-3 bg-[rgba(0,0,0,0.2)] rounded-lg text-sm overflow-hidden overflow-ellipsis whitespace-nowrap cursor-pointer"
                  onClick={() => {
                    window.open(`https://dial.to/?action=solana-action:${blinkLink}`, '_blank', 'noopener');
                  }}>
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
                className="button-primary flex-1"
                onClick={handleTweet}
              >
                <HiOutlineShare size={18} className="mr-2" />
                Share on X
              </button>

              <button
                className="button-secondary flex-1"
                onClick={handleNew}
              >
                Create New Blink
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default Form;