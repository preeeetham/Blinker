"use client";
import { ReactNode, useState } from 'react';
import { WalletButton } from '../solana/solana-provider';
import Sidebar from '../SideBar/sidebar';
import ThemeToggle from './theme-toggle';
import { useGlobalTitleState } from '@/app/GlobalStateContext';
import { FaInfo } from 'react-icons/fa';
import { HiInformationCircle } from 'react-icons/hi';
import { Pointer } from 'lucide-react';
import * as Tooltip from '@radix-ui/react-tooltip';

export function AppLayout({ children }: { children: ReactNode }) {
  const { value, info } = useGlobalTitleState();
  const [isTooltipOpen, setIsTooltipOpen] = useState(false);
  return (
    <div className="flex flex-row h-full">
      <div className='md:my-6 lap:mx-6'>
        <Sidebar />
      </div>
      <div
        className="flex-1 flex flex-col m-2 md:m-6 card overflow-hidden shadow-lg fade-in bg-opacity-90 backdrop-blur-md"
      >
        <div className="flex flex-col md:flex-row items-center justify-between p-2 md:p-7 border-b border-[var(--border-color)] bg-[var(--card-bg)] backdrop-blur-sm z-40">
          <span className='flex items-start justify-center'>
            <h1
              className="Title bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-transparent bg-clip-text animation-delay-200 fade-in"
            >
              {value}
            </h1>
            <Tooltip.Provider delayDuration={0}>
              <Tooltip.Root open={isTooltipOpen} onOpenChange={setIsTooltipOpen}>
                <Tooltip.Trigger asChild>
                  <span
                    className="inline-block ml-1"
                    onClick={() => setIsTooltipOpen(!isTooltipOpen)}
                  >
                    <HiInformationCircle color='#A48BFA' cursor='pointer' size={25} />
                  </span>
                </Tooltip.Trigger>
                <Tooltip.Portal>
                  <Tooltip.Content
                    className="bg-[var(--card-bg)] text-[var(--text-color)] px-3 py-1.5 rounded-md text-sm shadow-md max-w-[31rem] max-sm:max-w-[20rem]"
                    sideOffset={1}
                  >
                    {info}
                    <Tooltip.Arrow className="fill-[var(--card-bg)]" />
                  </Tooltip.Content>
                </Tooltip.Portal>
              </Tooltip.Root>
            </Tooltip.Provider>
          </span>
          <div
            className="flex items-center gap-6 mt-3 md:mt-0 animation-delay-300 fade-in"
          >
            <ThemeToggle />
            <WalletButton />
          </div>
        </div>
        <div
          className="flex-1 p-5 md:p-7 overflow-y-auto custom-scrollbar animation-delay-400 fade-in"
          style={{
            backgroundImage: 'radial-gradient(circle at 50% 50%, var(--bg-color) 0%, transparent 100%)',
            backgroundSize: '100% 100%'
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
