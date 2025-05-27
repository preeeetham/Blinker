import React, { useEffect, useState } from 'react';
import Image from 'next/image';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { MeteoraDlmmPair } from '@/server/meteora';
import { Label } from '../ui/label';

interface FormProps {
    icon: string;
    description: string;
    title: string;
    selectedPair: MeteoraDlmmPair | null;
}

const Preview: React.FC<FormProps> = ({ icon, description, title, selectedPair }) => {
    const [isDarkMode, setIsDarkMode] = useState(true);

    useEffect(() => {
        // Check if dark mode is active
        const checkTheme = () => {
            const isDark = document.body.classList.contains('dark-mode');
            setIsDarkMode(isDark);
        };

        // Initial check
        checkTheme();

        // Set up an observer to watch for class changes on the body
        const observer = new MutationObserver(checkTheme);
        observer.observe(document.body, {
            attributes: true,
            attributeFilter: ['class']
        });

        return () => observer.disconnect();
    }, []);

    const handleClick = () => {
      window.alert("It's a dummy button only for preview, Generate Blink and try it for real!!");
    }

    return (
        <div className="w-full max-w-md">
            <Card className={`overflow-hidden rounded-xl border-none shadow-xl ${
                isDarkMode
                ? "bg-[#0f0a19] text-white"
                : "bg-white text-[#1a1225] border border-gray-200"
            }`}>
                {/* Image Area */}
                <div className={`relative h-80 max-sm:h-80 w-full flex items-center justify-center ${
                    isDarkMode ? "bg-[#1a1225]" : "bg-gray-100"
                }`}>
                    {icon ? (
                        <Image
                            src={icon}
                            alt="Icon"
                            fill
                            style={{ objectFit: 'cover' }}
                            className="opacity-90"
                        />
                    ) : (
                        <div className={isDarkMode ? "text-purple-300 opacity-70" : "text-purple-500 opacity-70"}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                                <path d="M12 5v14M5 12h14"></path>
                            </svg>
                        </div>
                    )}
                </div>

                {/* Content Area */}
                <div className="p-5 space-y-3">
                    <h3 className={`text-lg font-semibold ${isDarkMode ? "text-white" : "text-gray-800"}`}>
                        {title || "Your Title"}
                    </h3>
                    {!selectedPair ? (
                      <p className={`text-sm ${isDarkMode ? "text-gray-300" : "text-gray-600"}`}>
                          {description || "Your Description shows up here. Keep it short and simple"}
                      </p>
                    ):(
                      <div className='flex flex-col gap-2'>
                        <Label className="text-sm text-gray-500">Pool Information</Label>
                        <div className='grid grid-cols-2 gap-2'>
                          <p>Liq: ${parseFloat(selectedPair?.liquidity || '0').toFixed(3)}</p>
                          <p>APR: {selectedPair.apr.toFixed(3)}%</p>
                          <p>Bin Step: {selectedPair.bin_step}</p>

                          <p>24h Vol: ${selectedPair.trade_volume_24h.toFixed(3)}</p>
                          <p>fee: {parseFloat(selectedPair.base_fee_percentage).toFixed(3)}</p>
                          <p>24h fees: {selectedPair.fees_24h.toFixed(3)}%</p>
                        </div>
                        <Label className="text-sm text-gray-500">Select a strategy</Label>
                        <div className='grid grid-cols-3 gap-2' onClick={handleClick}>
                          <div className='flex items-center gap-2'>
                            <input type="radio" id="spot" name="spot" value="spot" disabled/>
                            <label htmlFor="spot">Spot</label>
                          </div>
                          <div className='flex items-center gap-2'>
                            <input type="radio" id="Curve" name="Curve" value="Curve" disabled/>
                            <label htmlFor="Curve">Curve</label>
                          </div>
                          <div className='flex items-center gap-2'>
                            <input type="radio" id="BidAsk" name="BidAsk" value="BidAsk" disabled/>
                            <label htmlFor="BidAsk">Bid Ask</label>
                          </div>
                        </div>
                      </div>
                    )}

                    {/* SOL Buttons */}
                    <div className="grid grid-cols-2 gap-2 mt-4" onClick={handleClick}>
                        <Input
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none"
                            }
                            placeholder={`Enter ${selectedPair?.name.split('-')[0]} amount`}
                            disabled
                        />
                        <Input
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none"
                            }
                            placeholder={`Enter ${selectedPair?.name.split('-')[1]} amount`}
                            disabled
                        />
                    </div>

                    <div className="grid grid-cols-2 gap-2 mt-4" onClick={handleClick}>
                        <Input
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none"
                            }
                            placeholder={`Min Price for ` + selectedPair?.name.split('-')[0]}
                            disabled
                        />
                        <Input
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none"
                            }
                            placeholder={`Max Price for ` + selectedPair?.name.split('-')[0]}
                            disabled
                        />
                    </div>

                    {/* Send SOL Input */}
                    <div className="flex gap-2 mt-3">
                        <Button
                            className={isDarkMode
                                ? "bg-purple-500 hover:bg-purple-600 text-white w-full"
                                : "bg-purple-600 hover:bg-purple-700 text-white w-full"
                            }
                            onClick={handleClick}
                        >
                            Open a {selectedPair?.name || 'Pool Name'} Position
                        </Button>
                    </div>
                </div>
            </Card>
        </div>
    );
}

export default Preview;
