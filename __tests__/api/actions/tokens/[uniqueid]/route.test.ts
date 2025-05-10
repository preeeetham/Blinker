import { GET, POST } from "@/app/api/actions/tokens/[uniqueid]/route"
import clientPromise from "@/lib/mongodb"
import { PublicKey } from "@solana/web3.js"

// Mock MongoDB client
jest.mock("@/lib/mongodb", () => {
  return {
    __esModule: true,
    default: {
      connect: jest.fn().mockResolvedValue({}),
      db: jest.fn().mockReturnValue({
        collection: jest.fn().mockReturnValue({
          findOne: jest.fn().mockResolvedValue({
            _id: "test-id",
            icon: "https://example.com/token.png",
            label: "Buy Token",
            description: "Test description",
            title: "BUY Test Token",
            wallet: "test-wallet-address",
            mint: "test-mint-address",
            commission: "yes",
            percentage: 0.5,
            decimals: 9,
            isPaid: true,
          }),
        }),
      }),
    },
  }
})

// Mock ObjectId
jest.mock("mongodb", () => ({
  ObjectId: jest.fn().mockImplementation((id) => ({
    toString: () => id || "test-object-id",
  })),
  isValid: jest.fn().mockReturnValue(true),
}))

describe("Token Actions API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("GET returns token action details", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/tokens/test-id")
    const mockParams = { uniqueid: "test-id" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("icon", "https://example.com/token.png")
    expect(data).toHaveProperty("label", "Buy Token")
    expect(data).toHaveProperty("description", "Test description")
    expect(data).toHaveProperty("title", "BUY Test Token")
    expect(data).toHaveProperty("links.actions")
    expect(data.links.actions).toHaveLength(4)

    console.log("✅ Token Actions API GET returns token action details")
  })

  test("GET returns 403 for unpaid blinks", async () => {
    // Mock unpaid blink
    ;(clientPromise.db().collection().findOne as jest.Mock).mockResolvedValueOnce({
      isPaid: false,
    })

    const mockRequest = new Request("http://localhost:3000/api/actions/tokens/test-id")
    const mockParams = { uniqueid: "test-id" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(403)
    expect(data).toHaveProperty("message", "This blink is not paid for yet. Please pay to use it.")

    console.log("✅ Token Actions API GET returns 403 for unpaid blinks")
  })

  test("POST creates a transaction for token purchase", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/tokens/test-id?amount=100000", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
      }),
    })
    const mockParams = { uniqueid: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("fields.type", "transaction")
    expect(data).toHaveProperty("fields.transaction")
    expect(data).toHaveProperty("fields.message", "You just Pumped It!")

    console.log("✅ Token Actions API POST creates a transaction for token purchase")
  })

  test("POST returns 403 for unpaid blinks", async () => {
    // Mock unpaid blink
    ;(clientPromise.db().collection().findOne as jest.Mock).mockResolvedValueOnce({
      isPaid: false,
    })

    const mockRequest = new Request("http://localhost:3000/api/actions/tokens/test-id?amount=100000", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
      }),
    })
    const mockParams = { uniqueid: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(403)
    expect(data).toHaveProperty("message", "This blink is not paid for yet. Please pay to use it.")

    console.log("✅ Token Actions API POST returns 403 for unpaid blinks")
  })

  test("POST returns 500 for invalid account", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/tokens/test-id?amount=100000", {
      method: "POST",
      body: JSON.stringify({
        account: "invalid-account",
      }),
    })
    const mockParams = { uniqueid: "test-id" }

    // Mock PublicKey constructor to throw an error
    jest.spyOn(PublicKey.prototype, "constructor").mockImplementationOnce(() => {
      throw new Error("Invalid public key")
    })

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("message", "Invalid 'account' provided. It's not a real pubkey")

    console.log("✅ Token Actions API POST returns 500 for invalid account")
  })
})
