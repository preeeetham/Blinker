import { POST } from "@/app/api/actions/generate-blink/route"
import clientPromise from "@/lib/mongodb"
import { jest } from "@jest/globals"

// Mock MongoDB client
jest.mock("@/lib/mongodb", () => {
  return {
    __esModule: true,
    default: {
      connect: jest.fn().mockResolvedValue({}),
      db: jest.fn().mockReturnValue({
        collection: jest.fn().mockReturnValue({
          insertOne: jest.fn().mockResolvedValue({ insertedId: "test-id" }),
        }),
      }),
    },
  }
})

describe("Generate Blink API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("successfully generates a blink", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/generate-blink", {
      method: "POST",
      body: JSON.stringify({
        icon: "https://example.com/icon.png",
        label: "donate Sol",
        description: "Test description",
        title: "Test title",
        wallet: "test-wallet-address",
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("blinkLink")
    expect(data).toHaveProperty("id")

    // Verify MongoDB was called correctly
    expect(clientPromise.db).toHaveBeenCalledWith("Cluster0")
    expect(clientPromise.db().collection).toHaveBeenCalledWith("blinks")
    expect(clientPromise.db().collection().insertOne).toHaveBeenCalledWith({
      icon: "https://example.com/icon.png",
      label: "donate Sol",
      description: "Test description",
      title: "Test title",
      wallet: "test-wallet-address",
      createdAt: expect.any(Date),
      isPaid: false,
    })

    console.log("✅ Generate Blink API successfully generates a blink")
  })

  test("returns 400 for missing required fields", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/generate-blink", {
      method: "POST",
      body: JSON.stringify({
        // Missing required fields
        icon: "https://example.com/icon.png",
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Missing required fields")

    console.log("✅ Generate Blink API returns 400 for missing required fields")
  })

  test("returns 400 for invalid JSON", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/generate-blink", {
      method: "POST",
      body: "invalid-json",
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Invalid JSON in request body")

    console.log("✅ Generate Blink API returns 400 for invalid JSON")
  })

  test("returns 500 for database errors", async () => {
    // Mock database error
    ;(clientPromise.db().collection().insertOne as jest.Mock).mockRejectedValueOnce(new Error("Database error"))

    const mockRequest = new Request("http://localhost:3000/api/actions/generate-blink", {
      method: "POST",
      body: JSON.stringify({
        icon: "https://example.com/icon.png",
        label: "donate Sol",
        description: "Test description",
        title: "Test title",
        wallet: "test-wallet-address",
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("error", "Failed to generate blink")

    console.log("✅ Generate Blink API returns 500 for database errors")
  })
})
