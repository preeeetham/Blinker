import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { BuySellForm } from "@/components/tokens/buy-sell-form"
import { useWallet } from "@solana/wallet-adapter-react"

// Mock the useWallet hook
jest.mock("@solana/wallet-adapter-react")

describe("BuySellForm Component", () => {
  const mockProps = {
    type: "buy" as "buy" | "sell",
    selectedToken: "sol",
    setSelectedToken: jest.fn(),
    setIsLoading: jest.fn(),
  }

  beforeEach(() => {
    jest.clearAllMocks()

    // Mock the useWallet hook
    ;(useWallet as jest.Mock).mockReturnValue({
      publicKey: { toString: () => "test-wallet-address" },
      connected: true,
      sendTransaction: jest.fn().mockResolvedValue("test-signature"),
    })
  })

  test("renders buy form correctly", () => {
    render(<BuySellForm {...mockProps} />)

    expect(screen.getByText("Amount to Buy")).toBeInTheDocument()
    expect(screen.getByText("Transaction Summary")).toBeInTheDocument()
    expect(screen.getByText("Buy SOL")).toBeInTheDocument()

    console.log("✅ BuySellForm renders buy form correctly")
  })

  test("renders sell form correctly", () => {
    render(<BuySellForm {...mockProps} type="sell" />)

    expect(screen.getByText("Amount to Sell")).toBeInTheDocument()
    expect(screen.getByText("Transaction Summary")).toBeInTheDocument()
    expect(screen.getByText("Sell SOL")).toBeInTheDocument()

    console.log("✅ BuySellForm renders sell form correctly")
  })

  test("shows token selection message when no token is selected", () => {
    render(<BuySellForm {...mockProps} selectedToken={null} />)

    expect(screen.getByText("Please select a token from the list")).toBeInTheDocument()

    console.log("✅ BuySellForm shows token selection message when no token is selected")
  })

  test("handles amount input changes", () => {
    render(<BuySellForm {...mockProps} />)

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "10" } })

    expect(amountInput).toHaveValue("10")

    console.log("✅ BuySellForm handles amount input changes")
  })

  test("validates amount input - only allows numbers and decimals", () => {
    render(<BuySellForm {...mockProps} />)

    const amountInput = screen.getByPlaceholderText("0.00")

    // Valid input
    fireEvent.change(amountInput, { target: { value: "10.5" } })
    expect(amountInput).toHaveValue("10.5")

    // Invalid input (letters)
    fireEvent.change(amountInput, { target: { value: "10.5abc" } })
    expect(amountInput).toHaveValue("10.5")

    console.log("✅ BuySellForm validates amount input correctly")
  })

  test("shows error when selling more than balance", () => {
    render(<BuySellForm {...mockProps} type="sell" />)

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "100" } })

    expect(screen.getByText("Insufficient SOL balance")).toBeInTheDocument()

    console.log("✅ BuySellForm shows error when selling more than balance")
  })

  test("calculates USD value correctly", () => {
    render(<BuySellForm {...mockProps} />)

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "2" } })

    // SOL price is mocked at 150.25, so 2 SOL = $300.50
    expect(screen.getByText("≈ $300.50")).toBeInTheDocument()

    console.log("✅ BuySellForm calculates USD value correctly")
  })

  test("disables submit button when there are errors", () => {
    render(<BuySellForm {...mockProps} type="sell" />)

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "100" } })

    const submitButton = screen.getByText("Sell SOL")
    expect(submitButton).toBeDisabled()

    console.log("✅ BuySellForm disables submit button when there are errors")
  })

  test("disables submit button when amount is 0 or empty", () => {
    render(<BuySellForm {...mockProps} />)

    const submitButton = screen.getByText("Buy SOL")
    expect(submitButton).toBeDisabled()

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "0" } })
    expect(submitButton).toBeDisabled()

    console.log("✅ BuySellForm disables submit button when amount is 0 or empty")
  })

  test("handles form submission successfully", async () => {
    const alertSpy = jest.spyOn(window, "alert").mockImplementation()

    render(<BuySellForm {...mockProps} />)

    const amountInput = screen.getByPlaceholderText("0.00")
    fireEvent.change(amountInput, { target: { value: "1" } })

    const submitButton = screen.getByText("Buy SOL")
    fireEvent.click(submitButton)

    await waitFor(() => {
      expect(mockProps.setIsLoading).toHaveBeenCalledWith(true)
      expect(mockProps.setIsLoading).toHaveBeenCalledWith(false)
      expect(alertSpy).toHaveBeenCalled()
    })

    console.log("✅ BuySellForm handles form submission successfully")

    alertSpy.mockRestore()
  })
})
