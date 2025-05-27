"use client";
import React from 'react';

export default function ComingSoonCard() {
  return (
    <div className="w-full max-w-[400px] p-8 rounded-2xl backdrop-blur-[20px] saturate-[138%] shadow-[inset_0px_0px_20px_rgba(255,255,255,0.15)] bg-[rgba(17,25,40,0)] text-center mt-[100px] md:max-w-[90%] md:p-6 mx-auto">
      <h1 className="text-[2.25rem] font-bold text-white mb-4 md:text-[1.75rem] sm:text-[1.5rem]">Coming Soon</h1>
      <p className="text-white/80 mb-6 md:text-[0.9rem] sm:text-[0.8rem]">
        We&apos;re working hard to bring you something amazing. Stay tuned!
      </p>
    </div>
  );
}
