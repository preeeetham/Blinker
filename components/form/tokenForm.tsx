"use client";
import { useEffect } from 'react';
import { WalletButton } from '@/components/solana/solana-provider';
import { FaInfoCircle } from 'react-icons/fa';
import { PublicKey } from '@solana/web3.js';
import { Button } from '../ui/button';

// Define commission types for type safety
type CommissionType = "yes" | "no";

interface TokenFormProps {
  mint: string;
  description: string;
  percentage: number;
  takeCommission: CommissionType;
  showPreview: boolean;
  setMint: (value: string) => void;
  setDescription: (value: string) => void;
  setPercentage: (value: number) => void;
  setTakeCommission: (value: CommissionType) => void;
  handlePreview: () => Promise<void>;
  handleSubmit: () => Promise<void>;
  connected: boolean;
  publicKey: PublicKey | null;
}

export default function TokenForm({
  mint,
  description,
  percentage,
  takeCommission,
  showPreview,
  setMint,
  setDescription,
  setPercentage,
  setTakeCommission,
  handlePreview,
  handleSubmit,
  connected,
  publicKey,
}: TokenFormProps) {

  useEffect(() => {
    const handleInfoClick = (e: MouseEvent) => {
      window.alert("If you opt to take a commission, the specified percentage of the total transaction amount will be credited to your wallet. Please note that the maximum commission percentage allowed is 1%.");
    };

    document.querySelectorAll('.info-icon').forEach(icon => {
      icon.addEventListener('click', handleInfoClick as EventListener);
    });

    return () => {
      document.querySelectorAll('.info-icon').forEach(icon => {
        icon.removeEventListener('click', handleInfoClick as EventListener);
      });
    };
  }, []);

  return (
    <div className="space-y-6">
      <h1 className="text-2xl md:text-3xl font-bold mb-6 text-gradient bg-gradient-to-r from-[var(--gradient-start)] to-[var(--gradient-end)] bg-clip-text text-transparent">
        Sell/Resell Token
      </h1>

      <div>
        <label className="block text-sm font-medium mb-2 text-[var(--text-secondary)]">Mint Address</label>
        <input
          type="text"
          value={mint}
          onChange={(e) => setMint(e.target.value)}
          className="input-field"
          placeholder="Enter token mint address"
          maxLength={45}
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

      <div className="bg-[var(--card-bg)] rounded-xl p-4 border border-[var(--border-color)]">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <label className="text-sm font-medium text-[var(--text-color)]">
              Take commission
            </label>
            <FaInfoCircle className="text-[var(--text-secondary)] cursor-pointer info-icon" />
          </div>

          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                value="yes"
                checked={takeCommission === "yes"}
                onChange={(e) => setTakeCommission(e.target.value as CommissionType)}
                className="accent-[var(--accent-primary)]"
              />
              <span className="text-[var(--text-color)]">Yes</span>
            </label>

            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                value="no"
                checked={takeCommission === "no"}
                onChange={(e) => setTakeCommission(e.target.value as CommissionType)}
                className="accent-[var(--accent-primary)]"
              />
              <span className="text-[var(--text-color)]">No</span>
            </label>
          </div>
        </div>

        {takeCommission === "yes" && (
          <div className="mt-4">
            <label className="block text-sm font-medium mb-2 text-[var(--text-secondary)]">
              Commission Percentage (max 1%)
            </label>
            <input
              type="number"
              value={percentage}
              onChange={(e) => {
                const value = Math.min(1, parseFloat(e.target.value) || 0);
                setPercentage(value);
              }}
              className="input-field"
              placeholder="Enter commission percentage"
              max={1}
              min={0}
              step={0.01}
            />
          </div>
        )}
      </div>

      {publicKey ? (
        <Button
          className="py-3 px-6 rounded-xl font-medium cursor-pointer transition-all duration-300 bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white hover:opacity-90 active:scale-95 flex items-center justify-center gap-2 shadow-md w-full mt-4 text-lg"
          onClick={!showPreview ? handlePreview : handleSubmit}
          disabled={!connected || !mint || !description || mint.length <= 0 || description.length <= 0}
        >
          {!showPreview ? 'Preview Blink' : 'Generate Blink'}
        </Button>
      ) : (
        <div className="mt-4 text-center">
          <p className="text-[var(--text-secondary)] mb-3">Connect your wallet to generate a Blink</p>
          <WalletButton />
        </div>
      )}
    </div>
  );
}
