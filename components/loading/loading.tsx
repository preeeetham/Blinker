import React from 'react';
import { Loader2 } from 'lucide-react';

interface LoadingScreenProps {
  subtext?: string;
}

const LoadingScreen: React.FC<LoadingScreenProps> = ({ subtext = 'Loading...' }) => {
  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 backdrop-blur-md z-[9999]">
      <div className="text-center">
        <div className="inline-block animate-spin">
          <Loader2 className="w-12 h-12 text-white" />
        </div>
        <p className="mt-4 text-white text-base font-medium">{subtext}</p>
      </div>
    </div>
  );
};

export default LoadingScreen;
