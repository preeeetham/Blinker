"use client";
import LpForm from "@/components/form/lpForm";
import Preview from "@/components/preview/lpPreview";
import { useState } from "react";
import { Footer } from "@/components/footer";
import { MeteoraDlmmPair } from "@/server/meteora";

export default function Index() {
  const [mintAddress, setMintAddress] = useState<string>('');
  const [showForm, setShowForm] = useState(true);
  const [selectedPair, setSelectedPair] = useState<MeteoraDlmmPair | null>(null);

  return (
    <div className="flex flex-col md:min-h-screen">
      <div className="flex-1 flex flex-col md:flex-row items-center md:items-start md:justify-center gap-8 md:p-8">
        <LpForm
          mintAddress={mintAddress}
          setMintAddress={setMintAddress}
          showForm={showForm}
          setShowForm={setShowForm}
          selectedPair={selectedPair}
          setSelectedPair={setSelectedPair}
        />
        {showForm &&
        <Preview
          icon={'meteora.jpg'}
          description={'Your Description shows up here, Keep it short and simple'}
          title={selectedPair? `Open a ${selectedPair?.name} Position` :"Your Title"}
          selectedPair={selectedPair}
        />}
      </div>
      <Footer />
    </div>
  );
}
