"use client"
import Form from "@/components/form/form"
import Preview from "@/components/preview/preview"
import { useState, useEffect } from "react"
import { Footer } from "@/components/footer"
import { useWallet } from "@solana/wallet-adapter-react"
import { useRouter } from "next/navigation"

export default function Index() {
  const [icon, setIcon] = useState<string>("")
  const [description, setDescription] = useState<string>("")
  const [title, setTitle] = useState<string>("")
  const [showForm, setShowForm] = useState(true)
  const { connected } = useWallet()
  const router = useRouter()

  useEffect(() => {
    if (!connected) {
      router.push("/landing")
    }
  }, [connected, router])

  return (
    <div className="flex flex-col md:min-h-screen">
      <div className="flex-1 flex flex-col md:flex-row items-center md:items-start md:justify-center gap-8 md:p-8">
        <Form
          icon={icon}
          setIcon={setIcon}
          description={description}
          setDescription={setDescription}
          title={title}
          setTitle={setTitle}
          showForm={showForm}
          setShowForm={setShowForm}
        />
        {showForm && (
          <Preview
            icon={icon || "solana.jpg"}
            description={description || "Your Description shows up here, Keep it short and simple"}
            title={title || "Your Title"}
          />
        )}
      </div>
      <Footer />
    </div>
  )
}
