import { FaHeart } from "react-icons/fa";

export function Footer() {
  return (
    <footer className="py-4 border-t border-[var(--border-color)] bg-[rgba(30,41,59,0.2)] backdrop-blur-sm">
      <div className="container mx-auto flex justify-center items-center gap-2 text-[var(--text-secondary)] text-sm sm:text-base px-4">
        <span>Made with</span>
        <FaHeart className="text-[var(--accent-primary)] animate-pulse" size={12} />
        <span>on</span>
        <span className="font-medium gradient-text">
          Solana
        </span>
      </div>
    </footer>
  );
}
