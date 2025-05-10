import { GET } from "@/app/api/actions/token-info/route"
import { programs } from "@metaplex/js"
import { TokenListProvider } from "@solana/spl-token-registry"
import { jest } from "@jest/globals"

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

describe("Token Info API", () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("successfully fetches token info from metadata", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/token-info?mint=test-mint-address")

    const response = await GET(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("icon", "https://example.com/token.png")
    expect(data).toHaveProperty("title", "BUY Test Token")

    console.log("✅ Token Info API successfully fetches token info from metadata")
  })

  test("successfully fetches token info from registry when metadata fails", async () => {
    // Mock Metadata.load to throw an error
    jest.spyOn(programs.metadata.Metadata, "load").mockImplementationOnce(() => {
      throw new Error("Metadata not found")
    })

    const mockRequest = new Request("http://localhost:3000/api/actions/token-info?mint=test-mint-address")

    const response = await GET(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(200)
    expect(data).toHaveProperty("icon", "https://test-logo.com")
    expect(data).toHaveProperty("title", "BUY Test Token")

    console.log("✅ Token Info API successfully fetches token info from registry when metadata fails")
  })

  test("returns 400 for missing mint address", async () => {
    const mockRequest = new Request("http://localhost:3000/api/actions/token-info")

    const response = await GET(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(400)
    expect(data).toHaveProperty("error", "Mint address is required")

    console.log("✅ Token Info API returns 400 for missing mint address")
  })

  test("returns 500 when both metadata and registry fail", async () => {
    // Mock Metadata.load to throw an error
    jest.spyOn(programs.metadata.Metadata, "load").mockImplementationOnce(() => {
      throw new Error("Metadata not found")
    })

    // Mock TokenListProvider to throw an error
    jest.spyOn(TokenListProvider.prototype, "resolve").mockImplementationOnce(() => {
      throw new Error("Registry error")
    })

    const mockRequest = new Request("http://localhost:3000/api/actions/token-info?mint=test-mint-address")

    const response = await GET(mockRequest)
    const data = await response.json()

    expect(response.status).toBe(500)
    expect(data).toHaveProperty("error", "Token Info not found")

    console.log("✅ Token Info API returns 500 when both metadata and registry fail")
  })
})
