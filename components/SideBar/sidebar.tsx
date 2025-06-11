"use client";
import React, { useState, useEffect } from 'react';
import Link from 'next/link';
import { FaGithub, FaTwitter, FaBars, FaTimes, FaHardHat } from 'react-icons/fa';
import { IoWater } from "react-icons/io5";
import { HiOutlineCash, HiOutlineShoppingCart, HiOutlineCollection, HiPresentationChartLine } from 'react-icons/hi';
import Image from 'next/image';
import { useGlobalTitleState } from '@/app/GlobalStateContext';

interface SideButtonProps {
  href: string;
  children: React.ReactNode;
  icon: React.ReactNode;
  isActive: boolean;
  onClick: () => void;
}

const SideBarButton: React.FC<SideButtonProps> = ({ href, children, icon, isActive, onClick }) => (
  <Link href={href} className="no-underline w-full">
    <div
      className={`flex items-center gap-3 py-[min(3.5%,25px)] px-[min(5%,20px)] my-2 rounded-xl transition-all duration-300 hover-lift whitespace-normal break-words overflow-hidden ${
        isActive
          ? 'bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white font-medium shadow-md glow'
          : 'text-[var(--text-secondary)] hover:bg-[var(--hover-bg)] hover:text-[var(--text-color)]'
      }`}
      onClick={onClick}
    >
      <span className="text-xl">{icon}</span>
      <span className="font-medium">{children}</span>
    </div>
  </Link>
);

const Sidebar = () => {
  const [activeButton, setActiveButton] = useState('');
  const [isSidebarOpen, setIsSidebarOpen] = useState(false);
  const [isMobile, setIsMobile] = useState(true);
  const { setValue, value, setInfo } = useGlobalTitleState();

  useEffect(() => {
    const path = window.location.pathname;
    const segments = path.split('/');
    const endpoint = segments[segments.length - 1] || '';
    setActiveButton('/' + endpoint);

    if(endpoint === ''){
      setValue('Request SOL');
      setInfo("Easily generate a shareable Blink to receive SOL from anyone.");
    }else if(endpoint === 'token'){
      setValue('Sell/ReSell Tokens');
      setInfo("Create a Blink to sell or resell any SPL token. Share it with others so they can easily buy the token.")
    }else if(endpoint === 'Blinks'){
      setValue('My Blinks');
      setInfo('The list of Blinks created by this wallet')
    }

    const handleResize = () => {
      setIsMobile(window.innerWidth <= 1270);
    };

    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const handleSidebarToggle = () => {
    setIsSidebarOpen(!isSidebarOpen);
  };

  const handleButtonClick = (href: string) => {
    setActiveButton(href);
    if(href === '/'){
      setValue('Request SOL');
      setInfo("Easily generate a shareable Blink to receive SOL from anyone.");
    }else if(href === '/token'){
      setValue('Sell/ReSell Tokens');
      setInfo("Create a Blink to sell or resell any SPL token. Share it with others so they can easily buy the token.")
    }else if(href === '/Blinks'){
      setValue('My Blinks');
      setInfo('The list of Blinks created by this wallet')
    }
    console.log('Active Button:', href);
    console.log('value:', value);
    if (isMobile) {
      setIsSidebarOpen(false);
    }
  };

  return (
    <>
      <div
        className={`fixed lap:relative w-[300px] md:w-[min(300px,30vw)] h-full max-h-screen card py-10 flex flex-col justify-between z-20 backdrop-blur-md bg-[var(--card-bg)]/90 border-r border-[var(--border-color)] transition-all duration-300 overflow-hidden ${
          isMobile ? (isSidebarOpen ? 'translate-x-0' : '-translate-x-full') : 'translate-x-0'
        } sm:w-full sm:max-w-[min(300px,90vw)]`}
      >
        <div>
       <div
          className="flex justify-center cursor-pointer hover:scale-105 transition-transform duration-300"
          onClick={() => window.location.href = '/'}
        >
          <Image
            src="/blinker_logo.png"
            width={200}
            height={160}
            alt="Blinker Logo"
            className="rounded-lg shadow-xl"
          />
        </div>


          <div className="px-[min(5%,20px)] overflow-hidden">
          <div className="h-px w-full bg-gradient-to-r from-transparent via-[var(--border-color)] to-transparent"></div>
            <div className="fade-in animation-delay-100">
              <SideBarButton
                href="/"
                icon={<HiOutlineCash />}
                isActive={activeButton === '/'}
                onClick={() => handleButtonClick('/')}
              >
                Request SOL
              </SideBarButton>
            </div>

            <div className="fade-in animation-delay-200">
              <SideBarButton
                href="/token"
                icon={<HiOutlineShoppingCart />}
                isActive={activeButton === '/token'}
                onClick={() => handleButtonClick('/token')}
              >
                Sell/ReSell Tokens
              </SideBarButton>
            </div>

            <div className="fade-in animation-delay-300">
              <SideBarButton
                href="/Blinks"
                icon={<HiOutlineCollection />}
                isActive={activeButton === '/Blinks'}
                onClick={() => handleButtonClick('/Blinks')}
              >
                My Blinks
              </SideBarButton>
            </div>
          </div>
        </div>
      </div>

      {isMobile && (
        <>
          <button
            className="fixed bottom-6 right-6 z-30 flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-r from-[var(--accent-primary)] to-[var(--accent-secondary)] text-white shadow-lg hover:scale-110 active:scale-90 transition-transform duration-200 glow"
            onClick={handleSidebarToggle}
            aria-label="Toggle sidebar"
          >
            {isSidebarOpen ? <FaTimes size={24} /> : <FaBars size={24} />}
          </button>

          {isSidebarOpen && (
            <div
              className="fixed inset-0 bg-black/30 backdrop-blur-sm z-10 fade-in"
              onClick={handleSidebarToggle}
            />
          )}
        </>
      )}
    </>
  );
};

export default Sidebar;
