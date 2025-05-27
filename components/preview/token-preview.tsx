import React, { useEffect, useState } from 'react';
import Image from 'next/image';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

interface TokenPreviewProps {
    icon: string;
    description: string;
    title: string;
}

const TokenPreview: React.FC<TokenPreviewProps> = ({ icon, description, title }) => {
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
                <div className={`relative h-96 max-sm:h-80 w-full flex items-center justify-center ${
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
                    <p className={`text-sm ${isDarkMode ? "text-gray-300" : "text-gray-600"}`}>
                        {description || "Your Description shows up here. Keep it short and simple"}
                    </p>
                    {/* {label && (
                        <p className={`font-medium ${isDarkMode ? "text-purple-400" : "text-purple-600"}`}>
                            {label || "Your Label"}
                        </p>
                    )} */}

                    {/* SOL Buttons */}
                    <div className="grid grid-cols-2 gap-2 mt-4">
                        <Button
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none whitespace-normal"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none whitespace-normal"
                            }
                            onClick={handleClick}
                        >
                            {title === "Your Title" ? "Buy 100k $Token" : `Buy 100k $${title.slice(4)}`}
                        </Button>
                        <Button
                            className={isDarkMode
                                ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none whitespace-normal"
                                : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none whitespace-normal"
                            }
                            onClick={handleClick}
                        >
                            {title === "Your Title" ? "Buy 500k $Token" : `Buy 500k $${title.slice(4)}`}
                        </Button>
                    </div>

                    <Button
                        className={isDarkMode
                            ? "bg-[#2a1b3d] hover:bg-[#3a2b4d] text-white border-none w-full whitespace-normal"
                            : "bg-purple-100 hover:bg-purple-200 text-purple-800 border-none w-full whitespace-normal"
                        }
                        onClick={handleClick}
                    >
                        {title === "Your Title" ? "Buy 1M $Token" : `Buy 1M $${title.slice(4)}`}
                    </Button>

                    {/* Buy Token Input */}
                    <div className="flex gap-2 mt-3">
                        <Input
                            type="text"
                            className={isDarkMode
                                ? "bg-[#1a1225] border-[#3a2b4d] text-gray-300 placeholder-gray-500"
                                : "bg-gray-50 border-gray-200 text-gray-700 placeholder-gray-400"
                            }
                            placeholder="Enter amount"
                        />
                        <Button
                            className={isDarkMode
                                ? "bg-purple-500 hover:bg-purple-600 text-white"
                                : "bg-purple-600 hover:bg-purple-700 text-white"
                            }
                            onClick={handleClick}
                        >
                            {title == "Your Title"? "Buy Token" : title}
                        </Button>
                    </div>
                </div>
            </Card>
        </div>
    );
}

export default TokenPreview;
