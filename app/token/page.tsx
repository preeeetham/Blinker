"use client";
import { useState, useRef, useEffect } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import TokenPreview from "@/components/preview/token-preview";
import TokenForm from '@/components/form/tokenForm';
import {
  PublicKey,
  Transaction,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Connection,
  clusterApiUrl,
  TransactionInstruction
} from '@solana/web3.js';
import { HiOutlineClipboardCopy, HiOutlineShare, HiOutlinePlus } from 'react-icons/hi';
import { Footer } from '@/components/footer';
import LoadingScreen from '@/components/Loading/loading';
import { confirmTransaction, createTransaction } from '@/server/transaction';

// Define commission types for type safety
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
    if(takeCommission === "no"){
      setPercentage(0);
    }
  }, [takeCommission]);

  const handleSubmit = async () => {
    setLoading(true);
    setLoadingText('Waiting for Transaction confirmation!!');
    try {
      if (!connected || !publicKey) {
        console.error('Wallet not connected or not available');
        window.alert('Please connect your wallet first');
        return;
      }

      // Validate form fields (label, description, mint)
      if (!description || !mint) {
        console.error('Please fill all fields');
        window.alert('Please fill all fields');
        return;
      }

      try {
        const response = await fetch('/api/actions/generate-blink/token', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            label: 'Buy Token',
            description,
            wallet: publicKey.toString(),
            mint,
            commission: takeCommission,
            percentage: percentage,
          }),
        });

        if (!response.ok) {
          throw new Error('Failed to generate blink');
        }

        const data = await response.json();


        const messageString = `${publicKey.toString() + data.id.toString()}`;
        const getTransaction = await createTransaction(messageString, 0.01, publicKey.toString());
        const  {serializedTransaction, blockhash, lastValidBlockHeight} = getTransaction;
        const transaction = Transaction.from(Buffer.from(serializedTransaction, 'base64'));
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
            orderId: data.id.toString(),
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
        console.error('Error sending transaction:', error);
        window.alert('Transaction failed. Please try again.');
      }

    } catch (error) {
      setLoading(false);
      console.error('Error in handleSubmit:', error);
      window.alert('There was an issue generating your blink. Please try again.');
    }
  };

  const handlePreview = async () => {
    try {
      setLoading(true);
      setLoadingText('Generating Blink Preview!!');
      if (!connected || !publicKey) {
        console.error('Wallet not connected');
        return;
      }

      if (!description || !mint) {
        console.error('Please fill all fields');
        window.alert('Please fill all fields');
        return;
      }
      const response = await fetch('/api/actions/generate-blink/token?mint=' + mint);

      if (!response.ok) {
        console.log('Error:', response);
        throw new Error('Failed to generate blink');
      }

      const data = await response.json();
      setShowPreview(true);
      setIcon(data.icon);
      setTitle(data.title);
      setLoading(false);
    } catch(err){
      setLoading(false);
      console.error(err);
      window.alert("Invalid Mint Address!!");
      return;
    }
  }

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
    <div className="flex flex-col md:min-h-screen">
      {loading && <LoadingScreen subtext={loadingText}/>}

      <div className="flex-1 flex flex-col md:flex-row items-center md:items-start md:justify-center gap-8 md:p-8">
        <div className="w-full max-w-2xl">
          <div className="md:card md:p-10" ref={form}>
            {showForm && (
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
            )}

            {!showForm && (
              <div className="space-y-6">
                <h1 className="text-2xl md:text-3xl font-bold mb-6 text-gradient bg-gradient-to-r from-[var(--gradient-start)] to-[var(--gradient-end)] bg-clip-text text-transparent">
                  Your Blink is Ready!
                </h1>

                <div className="p-4 rounded-xl bg-[var(--card-bg)] border border-[var(--border-color)]">
                  <p className="text-sm text-[var(--text-secondary)] mb-2">Blink Link:</p>
                  <div className="flex items-center gap-2">
                    <div
                        className="flex-1 p-3 bg-[rgba(0,0,0,0.2)] rounded-lg text-sm overflow-hidden overflow-ellipsis whitespace-nowrap cursor-pointer"
                        onClick={()=>{window.open(`https://dial.to/?action=solana-action:${blinkLink}`, '_blank', 'noopener');}}>
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
