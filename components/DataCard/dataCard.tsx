"use client";
import { useState } from "react";
import { HiOutlineClipboardCopy, HiOutlineShare } from 'react-icons/hi';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/use-toast";

interface DataProps {
  base: string;
  code: string;
  title: string;
  endpoint: string;
}

const DataCard: React.FC<DataProps> = ({ base, code, title, endpoint }) => {
    const [copied, setCopied] = useState(false);
    const { toast } = useToast();
    const blinkLink = `https://www.getblink.fun/api/actions/${endpoint}/${code}`;

    const handleCopy = () => {
      navigator.clipboard.writeText(`${base}${blinkLink}`);
      setCopied(true);
      toast({
        title: "Link copied",
        description: "The link has been copied to your clipboard",
        duration: 2000,
      });
      setTimeout(() => setCopied(false), 1500);
    };

    const handleTweet = () => {
      const tweetText = `Check out this Blink I just made using @getblinkdotfun: ${base}${blinkLink}`;
      const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`;
      window.open(twitterUrl, '_blank');
    };

    return (
      <Card className="border-[var(--border-color)] bg-[var(--card-bg)] shadow-md border-2">
        <CardHeader className="pb-2">
          <CardTitle className="text-lg sm:text-xl font-semibold gradient-text">
            {title}
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="p-3 bg-[var(--bg-color)] max-sm:h-12 rounded-lg border border-[var(--border-color)] transition-all duration-300 hover:border-[var(--accent-primary)] group">
            <a
              href={`${base}${blinkLink}`}
              target="_blank"
              className="text-[var(--text-color)] group-hover:text-[var(--accent-primary)] transition-all duration-300 text-sm sm:text-base"
              rel="noopener noreferrer"
            >
              <p className="w-full h-full overflow-hidden whitespace-normal text-ellipsis">
                {base + base.includes("devnet") ? blinkLink.slice(0, -9)+"..." : blinkLink}
              </p>
            </a>
          </div>

          <div className="flex gap-2 sm:gap-3 flex-wrap sm:flex-nowrap">
            <Button
              variant="outline"
              size="sm"
              className="border-[var(--border-color)] text-[var(--text-color)]"
              onClick={handleCopy}
              aria-label="Copy link"
            >
              <HiOutlineClipboardCopy className="mr-2 h-4 w-4" />
              {copied ? "Copied!" : "Copy"}
            </Button>
            <Button
              variant="default"
              size="sm"
              className="bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white"
              onClick={handleTweet}
              aria-label="Share on Twitter"
            >
              <HiOutlineShare className="mr-2 h-4 w-4" />
              Tweet
            </Button>
          </div>
        </CardContent>
      </Card>
    );
}

export default DataCard;
