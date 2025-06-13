import { FaHeart, FaTwitter, FaDiscord, FaGithub } from "react-icons/fa";

export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="py-6 border-t border-[var(--border-color)] bg-[rgba(30,41,59,0.2)] backdrop-blur-sm">
      <div className="container mx-auto px-4">
        <div className="flex flex-col items-center gap-4">

          {/* Copyright and tagline */}
          <div className="flex flex-col sm:flex-row items-center gap-2 sm:gap-4 text-[var(--text-secondary)] text-xs sm:text-sm">
            <span>© {currentYear} Blinker. All rights reserved.</span>
            <span className="text-[var(--border-color)] hidden sm:block">•</span>
            <span>Powering the future of decentralized payments</span>
          </div>
          <div className="flex justify-center items-center gap-2 text-[var(--text-secondary)] text-sm sm:text-base">
            <span>Made with</span>
            <FaHeart className="text-[var(--accent-primary)] animate-pulse" size={12} />
            <span>on</span>
            <span className="font-medium gradient-text">
              Solana
            </span>
          </div>
        </div>
      </div>
    </footer>
  );
}