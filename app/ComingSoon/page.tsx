"use client";
import ComingSoonCard from "@/components/ComingSoon/coming-soon"
import { Footer } from "@/components/footer";
export default function Page(){
  return (
    <div style={{ display: 'flex',flexDirection: 'column', justifyContent: 'center', alignItems: 'center', height: "100%" }}>
      <ComingSoonCard />
      <Footer />
    </div>
  )
}
