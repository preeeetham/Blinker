'use client';

import React, { createContext, useContext, useState, ReactNode } from 'react';

type GlobalTitleContextType = {
  value: string;
  setValue: (newValue: string) => void;
  info: string;
  setInfo: (newInfo: string) => void;
};

const GlobalTitleContext = createContext<GlobalTitleContextType | undefined>(undefined);

export const GlobalTitleProvider = ({ children }: { children: ReactNode }) => {
  const [value, setValue] = useState<string>('GetBlink.fun');
  const [info, setInfo] = useState<string>("A website that let's your create blinks on solana");

  return (
    <GlobalTitleContext.Provider value={{ value, setValue, info, setInfo }}>
      {children}
    </GlobalTitleContext.Provider>
  );
};

export const useGlobalTitleState = (): GlobalTitleContextType => {
  const context = useContext(GlobalTitleContext);
  if (!context) {
    throw new Error('useGlobalTitleState must be used within a GlobalTitleProvider');
  }
  return context;
};
