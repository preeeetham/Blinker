import { POST } from "@/app/api/actions/token-blink/route"
import clientPromise from "@/lib/mongodb"
import { getMint } from "@solana/spl-token"
import { programs } from "@metaplex/js"
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

// Mock fetch for token metadata
global.fetch = jest.fn().mockImplementation(() =>
  Promise.resolve({
    json: () =>
      Promise.resolve({
        name: "Test Token",
        image: "https://example.com/token.png",
      }),
  }),
)

describe("Token Blink API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("successfully generates a token blink", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/token-blink", {
      method: "POST",
      body: JSON.stringify({
        label: "Buy Token",
        description: "Test description",
        wallet: "test-wallet-address",
        mint: "test-mint-address",
        commission: "yes",
        percentage: 0.5,
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
      icon: "https://example.com/token.png",
      label: "Buy Token",
      description: "Test description",
      title: "BUY Test Token",
      wallet: "test-wallet-address",
      mint: "test-mint-address",
      commission: "yes",
      percentage: 0.5,
      decimals: 9,
      createdAt: expect.any(Date),
      isPaid: false,
    })

    console.log("✅ Token Blink API successfully generates a token blink")
  })

  test("returns 400 for missing required fields", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/token-blink", {
      method: "POST",
      body: JSON.stringify({
        // Missing required fields
        label: "Buy Token",
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Missing required fields")

    console.log("✅ Token Blink API returns 400 for missing required fields")
  })

  test("returns 400 for invalid JSON", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/token-blink", {
      method: "POST",
      body: "invalid-json",
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Invalid JSON in request body")

    console.log("✅ Token Blink API returns 400 for invalid JSON")
  })

  test("returns 500 for invalid mint", async () => {
    // Mock getMint to throw an error
    ;(getMint as jest.Mock).mockRejectedValueOnce(new Error("Invalid mint"))

    const mockRequest = new Request("http://localhost:3000/api/actions/token-blink", {
      method: "POST",
      body: JSON.stringify({
        label: "Buy Token",
        description: "Test description",
        wallet: "test-wallet-address",
        mint: "invalid-mint-address",
        commission: "yes",
        percentage: 0.5,
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("error", "Failed to generate blink")

    console.log("✅ Token Blink API returns 500 for invalid mint")
  })

  test("returns 500 for metadata fetch errors", async () => {
    // Mock Metadata.load to throw an error
    jest.spyOn(programs.metadata.Metadata, "load").mockImplementationOnce(() => {
      throw new Error("Metadata not found")
    })

    // Mock TokenListProvider to throw an error
    jest
      .spyOn(require("@solana/spl-token-registry").TokenListProvider.prototype, "resolve")
      .mockImplementationOnce(() => {
        throw new Error("Registry error")
      })

    const mockRequest = new Request("http://localhost:3000/api/actions/token-blink", {
      method: "POST",
      body: JSON.stringify({
        label: "Buy Token",
        description: "Test description",
        wallet: "test-wallet-address",
        mint: "test-mint-address",
        commission: "yes",
        percentage: 0.5,
      }),
    })

    const response = await POST(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("error", "Failed to generate blink")

    console.log("✅ Token Blink API returns 500 for metadata fetch errors")
  })
})
