"use client"
import Form from "@/components/form/form"
import Preview from "@/components/preview/preview"
import { useState, useEffect } from "react"
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
    <div className="bg-[var(--bg-color)] text-[var(--text-color)]">
      <section className="py-8 px-4">
        <div className="max-w-6xl mx-auto">
          <div className="mb-8 text-center">
            <h1 className="text-3xl md:text-4xl font-black gradient-text">Request SOL</h1>
            <p className="text-[var(--text-secondary)] mt-2">Create a shareable Blink to receive SOL payments</p>
          </div>

          <div className="grid lg:grid-cols-2 gap-8 items-start">
            <div>
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
            </div>

            {showForm && (
              <div>
                <div className="mb-4 text-center">
                  <h2 className="text-xl font-semibold gradient-text">Live Preview</h2>
                </div>
                <Preview
                  icon={icon || "solana.jpg"}
                  description={description || "Your Description shows up here, Keep it short and simple"}
                  title={title || "Your Title"}
                />
              </div>
            )}
          </div>
        </div>
      </section>

      
    </div>
  )
}
