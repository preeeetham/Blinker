"use client";

import { useState, useEffect } from 'react';
import { FaMoon, FaSun } from 'react-icons/fa';

export const ThemeToggle = () => {
  const [isDarkMode, setIsDarkMode] = useState(false);

  // Initialize theme based on localStorage or system preference
  useEffect(() => {
    // Check if user has a saved preference
    const savedTheme = localStorage.getItem('theme');

    if (savedTheme === 'dark') {
      setIsDarkMode(true);
      document.body.classList.add('dark-mode');
    } else if (savedTheme === 'light') {
      setIsDarkMode(false);
      document.body.classList.remove('dark-mode');
    } else {
      // Check system preference if no saved preference
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      setIsDarkMode(prefersDark);
      if (prefersDark) {
        document.body.classList.add('dark-mode');
      }
    }
  }, []);

  const toggleTheme = () => {
    setIsDarkMode(!isDarkMode);

    if (!isDarkMode) {
      document.body.classList.add('dark-mode');
      localStorage.setItem('theme', 'dark');
    } else {
      document.body.classList.remove('dark-mode');
      localStorage.setItem('theme', 'light');
    }
  };

  return (
    <div
      className="flex items-center gap-1 hover:scale-105 active:scale-95 transition-transform duration-200"
    >
      <span
        className={`text-xl ${isDarkMode ? 'text-[#a78bfa]' : 'text-[#fbbf24]'} transition-all duration-500`}
        style={{ transform: isDarkMode ? 'rotate(-30deg)' : 'rotate(0deg)' }}
      >
        {isDarkMode ? <FaMoon className="floating" /> : <FaSun className="floating" />}
      </span>
      <label className="theme-toggle">
        <input
          type="checkbox"
          className="theme-toggle-checkbox"
          checked={isDarkMode}
          onChange={toggleTheme}
        />
        <span className="theme-toggle-slider glow">
          <span className="absolute inset-0 flex items-center justify-center text-xs fade-in">
            {isDarkMode ?
              <span className="absolute right-1 text-white">🌙</span> :
              <span className="absolute left-1 text-white">☀️</span>
            }
          </span>
        </span>
      </label>
    </div>
  );
};

export default ThemeToggle;
