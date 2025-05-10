"use client"

import { useState } from "react"
import Form from "@/components/form"
import Preview from "@/components/preview"

export default function CreatePage() {
  const [icon, setIcon] = useState("")
  const [description, setDescription] = useState("")
  const [title, setTitle] = useState("")
  const [showForm, setShowForm] = useState(true)

  return (
    <div className="container mx-auto py-8">
      <div className="flex flex-col lg:flex-row gap-8 items-start">
        <div className="w-full lg:w-3/5">
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
        <div className="w-full lg:w-2/5 sticky top-24">
          <div className="card p-6">
            <h2 className="text-xl font-bold mb-4">Preview</h2>
            <div className="flex justify-center">
              <Preview icon={icon} description={description} title={title} />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
