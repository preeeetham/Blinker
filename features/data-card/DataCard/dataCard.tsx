"use client";
import { useState } from "react";
import { HiOutlineClipboardCopy, HiOutlineShare } from 'react-icons/hi';
import { Card, CardContent, CardHeader, CardTitle } from "@/ui-kit/primitives/card";
import { Button } from "@/ui-kit/primitives/button";
import { useToast } from "@/ui-kit/primitives/use-toast";

interface DataProps {
  base: string;
  code: string;
  title: string;
  endpoint: string;
}

const DataCard: React.FC<DataProps> = ({ base, code, title, endpoint }) => {
    const [copied, setCopied] = useState(false);
    const { toast } = useToast();
    
    // Construct the proper blink link
    const blinkLink = `${base}${code}`;

    const handleCopy = async () => {
      try {
        await navigator.clipboard.writeText(blinkLink);
        setCopied(true);
        toast({
          title: "Link copied",
          description: "The link has been copied to your clipboard",
          duration: 2000,
        });
        setTimeout(() => setCopied(false), 1500);
      } catch (error) {
        console.error('Failed to copy to clipboard:', error);
        toast({
          title: "Copy failed",
          description: "Please try copying the link manually",
          variant: "destructive",
        });
      }
    };

    const handleTweet = () => {
      const tweetText = `Check out this Blink I just made using blinkgen(dot)xyz: ${blinkLink}`;
      const twitterUrl = `https://X.com/intent/tweet?text=${encodeURIComponent(tweetText)}`;
      window.open(twitterUrl, '_blank', 'noopener,noreferrer');
    };

    return (
      <Card className="card hover-lift group transition-all duration-300 hover:shadow-lg">
        <CardHeader className="pb-4">
          <CardTitle className="text-xl sm:text-2xl font-bold gradient-text group-hover:scale-105 transition-transform duration-300">
            {title}
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-6">
          <div className="p-4 bg-[var(--bg-color)] rounded-xl border border-[var(--border-color)] transition-all duration-300 hover:border-[var(--accent-primary)] hover:shadow-md group/link">
            <a
              href={blinkLink}
              target="_blank"
              className="text-[var(--text-color)] group-hover/link:text-[var(--accent-primary)] transition-all duration-300 text-sm sm:text-base font-medium"
              rel="noopener noreferrer"
            >
              <p className="w-full overflow-hidden whitespace-normal break-all">
                {blinkLink.length > 60 ? blinkLink.slice(0, 60) + "..." : blinkLink}
              </p>
            </a>
          </div>

          <div className="flex gap-3 flex-wrap sm:flex-nowrap">
            <Button
              variant="outline"
              size="sm"
              className="button-secondary flex-1 hover-lift"
              onClick={handleCopy}
              aria-label="Copy link"
            >
              <HiOutlineClipboardCopy className="mr-2 h-4 w-4" />
              {copied ? "Copied!" : "Copy"}
            </Button>
            <Button
              variant="default"
              size="sm"
              className="button-primary flex-1"
              onClick={handleTweet}
              aria-label="Share on Twitter"
            >
              <HiOutlineShare className="mr-2 h-4 w-4" />
              Share
            </Button>
          </div>
        </CardContent>
      </Card>
    );
}

export default DataCard;
