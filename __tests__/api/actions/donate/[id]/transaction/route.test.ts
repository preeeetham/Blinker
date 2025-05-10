import { POST } from "@/app/api/actions/donate/[id]/transaction/route"
import clientPromise from "@/lib/mongodb"
import { SystemProgram } from "@solana/web3.js"
import { jest } from "@jest/globals"

// Mock MongoDB client
jest.mock("@/lib/mongodb", () => {
  return {
    __esModule: true,
    default: {
      connect: jest.fn().mockResolvedValue({}),
      db: jest.fn().mockReturnValue({
        collection: jest.fn().mockReturnValue({
          insertOne: jest.fn().mockResolvedValue({ insertedId: "test-transaction-id" }),
        }),
      }),
    },
  }
})

describe("Donate Transaction API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("POST creates a transaction for donation", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id/transaction?amount=1.0", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
      }),
    })
    const mockParams = { id: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("transaction")
    expect(data).toHaveProperty("message", "Sending 1 SOL to the creator")

    // Verify SystemProgram.transfer was called correctly
    expect(SystemProgram.transfer).toHaveBeenCalledWith({
      fromPubkey: expect.any(Object),
      toPubkey: expect.any(Object),
      lamports: 1000000000, // 1 SOL in lamports
    })

    // Verify transaction was recorded in the database
    expect(clientPromise.db).toHaveBeenCalledWith("Cluster0")
    expect(clientPromise.db().collection).toHaveBeenCalledWith("transactions")
    expect(clientPromise.db().collection().insertOne).toHaveBeenCalledWith({
      blinkId: "test-id",
      sender: "test-wallet-address",
      amount: 1,
      createdAt: expect.any(Date),
      status: "pending",
    })

    console.log("✅ Donate Transaction API POST creates a transaction for donation")
  })

  test("POST uses input amount when query param is missing", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id/transaction", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
        input: {
          amount: "2.5",
        },
      }),
    })
    const mockParams = { id: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("message", "Sending 2.5 SOL to the creator")

    // Verify SystemProgram.transfer was called with the correct amount
    expect(SystemProgram.transfer).toHaveBeenCalledWith({
      fromPubkey: expect.any(Object),
      toPubkey: expect.any(Object),
      lamports: 2500000000, // 2.5 SOL in lamports
    })

    console.log("✅ Donate Transaction API POST uses input amount when query param is missing")
  })

  test("POST uses default amount when both query param and input are missing", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id/transaction", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
      }),
    })
    const mockParams = { id: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("message", "Sending 0.1 SOL to the creator")

    // Verify SystemProgram.transfer was called with the default amount
    expect(SystemProgram.transfer).toHaveBeenCalledWith({
      fromPubkey: expect.any(Object),
      toPubkey: expect.any(Object),
      lamports: 100000000, // 0.1 SOL in lamports
    })

    console.log("✅ Donate Transaction API POST uses default amount when both query param and input are missing")
  })

  test("POST returns 400 for missing account", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id/transaction?amount=1.0", {
      method: "POST",
      body: JSON.stringify({}),
    })
    const mockParams = { id: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Missing account")

    console.log("✅ Donate Transaction API POST returns 400 for missing account")
  })

  test("POST handles database errors", async () => {
    // Mock database error
    ;(clientPromise.db().collection().insertOne as jest.Mock).mockRejectedValueOnce(new Error("Database error"))

    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id/transaction?amount=1.0", {
      method: "POST",
      body: JSON.stringify({
        account: "test-wallet-address",
      }),
    })
    const mockParams = { id: "test-id" }

    const response = await POST(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("error", "Failed to create transaction")

    console.log("✅ Donate Transaction API POST handles database errors")
  })
})
