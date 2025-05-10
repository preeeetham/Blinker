import { GET } from "@/app/api/actions/donate/[id]/route"
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
          findOne: jest.fn().mockResolvedValue({
            _id: "test-id",
            icon: "https://example.com/icon.png",
            title: "Test Blink",
            description: "Test description",
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

describe("Donate API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("GET returns blink details", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id")
    const mockParams = { id: "test-id" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("title", "Test Blink")
    expect(data).toHaveProperty("description", "Test description")
    expect(data).toHaveProperty("icon", "https://example.com/icon.png")
    expect(data).toHaveProperty("buttons")
    expect(data.buttons).toHaveLength(3)
    expect(data).toHaveProperty("inputs")
    expect(data.inputs).toHaveLength(1)
    expect(data).toHaveProperty("actions")
    expect(data.actions).toHaveLength(1)

    console.log("✅ Donate API GET returns blink details")
  })

  test("GET returns 400 for missing ID", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/")
    const mockParams = { id: "" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Missing blink ID")

    console.log("✅ Donate API GET returns 400 for missing ID")
  })

  test("GET returns 404 for non-existent blink", async () => {
    // Mock blink not found
    ;(clientPromise.db().collection().findOne as jest.Mock).mockResolvedValueOnce(null)

    const mockRequest = new Request("http://localhost:3000/api/actions/donate/non-existent-id")
    const mockParams = { id: "non-existent-id" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    expect(response.status).toBe(404)
    expect(data).toHaveProperty("error", "Blink not found")

    console.log("✅ Donate API GET returns 404 for non-existent blink")
  })

  test("GET includes correct transaction targets", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/donate/test-id")
    const mockParams = { id: "test-id" }

    const response = await GET(mockRequest, { params: mockParams })
    const data = await response.json()

    // Check that buttons have correct transaction targets
    expect(data.buttons[0].action.target).toBe(
      "http://localhost:3000/api/actions/donate/test-id/transaction?amount=0.05",
    )
    expect(data.buttons[1].action.target).toBe(
      "http://localhost:3000/api/actions/donate/test-id/transaction?amount=1.0",
    )
    expect(data.buttons[2].action.target).toBe(
      "http://localhost:3000/api/actions/donate/test-id/transaction?amount=5.0",
    )

    // Check that the custom amount action has the correct target
    expect(data.actions[0].action.target).toBe("http://localhost:3000/api/actions/donate/test-id/transaction")

    console.log("✅ Donate API GET includes correct transaction targets")
  })
})
