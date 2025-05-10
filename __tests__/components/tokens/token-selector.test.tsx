import { render, screen, fireEvent } from "@testing-library/react"
import { TokenSelector } from "@/components/tokens/token-selector"

describe("TokenSelector Component", () => {
  const mockProps = {
    selectedToken: null,
    setSelectedToken: jest.fn(),
  }

  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("renders token list correctly", () => {
    render(<TokenSelector {...mockProps} />)

    // Check for token symbols
    expect(screen.getByText("SOL")).toBeInTheDocument()
    expect(screen.getByText("USDC")).toBeInTheDocument()
    expect(screen.getByText("BONK")).toBeInTheDocument()
    expect(screen.getByText("ORCA")).toBeInTheDocument()
    expect(screen.getByText("RAY")).toBeInTheDocument()

    console.log("✅ TokenSelector renders token list correctly")
  })

  test("handles token selection", () => {
    render(<TokenSelector {...mockProps} />)

    const solToken = screen.getByText("SOL").closest('div[role="button"]')
    fireEvent.click(solToken)

    expect(mockProps.setSelectedToken).toHaveBeenCalledWith("sol")

    console.log("✅ TokenSelector handles token selection")
  })

  test("highlights selected token", () => {
    render(<TokenSelector {...mockProps} selectedToken="sol" />)

    const solToken = screen.getByText("SOL").closest('div[role="button"]')
    expect(solToken).toHaveClass("bg-purple-600/20")

    console.log("✅ TokenSelector highlights selected token")
  })

  test("filters tokens by search query", () => {
    render(<TokenSelector {...mockProps} />)

    const searchInput = screen.getByPlaceholderText("Search tokens...")
    fireEvent.change(searchInput, { target: { value: "sol" } })

    // SOL should be visible
    expect(screen.getByText("SOL")).toBeInTheDocument()

    // Other tokens should not be visible
    expect(screen.queryByText("USDC")).not.toBeInTheDocument()
    expect(screen.queryByText("BONK")).not.toBeInTheDocument()

    console.log("✅ TokenSelector filters tokens by search query")
  })

  test('shows "no tokens found" message when search has no results', () => {
    render(<TokenSelector {...mockProps} />)

    const searchInput = screen.getByPlaceholderText("Search tokens...")
    fireEvent.change(searchInput, { target: { value: "xyz" } })

    expect(screen.getByText('No tokens found matching "xyz"')).toBeInTheDocument()

    console.log('✅ TokenSelector shows "no tokens found" message when search has no results')
  })

  test("filters tokens by name", () => {
    render(<TokenSelector {...mockProps} />)

    const searchInput = screen.getByPlaceholderText("Search tokens...")
    fireEvent.change(searchInput, { target: { value: "bonk" } })

    // BONK should be visible
    expect(screen.getByText("BONK")).toBeInTheDocument()

    // Other tokens should not be visible
    expect(screen.queryByText("SOL")).not.toBeInTheDocument()
    expect(screen.queryByText("USDC")).not.toBeInTheDocument()

    console.log("✅ TokenSelector filters tokens by name")
  })

  test("filters tokens by symbol", () => {
    render(<TokenSelector {...mockProps} />)

    const searchInput = screen.getByPlaceholderText("Search tokens...")
    fireEvent.change(searchInput, { target: { value: "usd" } })

    // USDC should be visible
    expect(screen.getByText("USDC")).toBeInTheDocument()

    // Other tokens should not be visible
    expect(screen.queryByText("SOL")).not.toBeInTheDocument()
    expect(screen.queryByText("BONK")).not.toBeInTheDocument()

    console.log("✅ TokenSelector filters tokens by symbol")
  })
})
