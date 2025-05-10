import { render, screen, fireEvent } from "@testing-library/react"
import TokenForm from "@/components/tokens/token-form"
import type { PublicKey } from "@solana/web3.js"

describe("TokenForm Component", () => {
  const mockProps = {
    mint: "test-mint-address",
    description: "Test description",
    percentage: 0.5,
    takeCommission: "yes" as "yes" | "no",
    showPreview: false,
    setMint: jest.fn(),
    setDescription: jest.fn(),
    setPercentage: jest.fn(),
    setTakeCommission: jest.fn(),
    handlePreview: jest.fn().mockResolvedValue(undefined),
    handleSubmit: jest.fn().mockResolvedValue(undefined),
    connected: true,
    publicKey: { toString: () => "test-wallet-address" } as unknown as PublicKey,
  }

  beforeEach(() => {
    jest.clearAllMocks()
  })

  test("renders form correctly", () => {
    render(<TokenForm {...mockProps} />)

    expect(screen.getByText("Sell/Resell Token")).toBeInTheDocument()
    expect(screen.getByLabelText(/Mint Address/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/Description/i)).toBeInTheDocument()
    expect(screen.getByText("Take commission")).toBeInTheDocument()
    expect(screen.getByText("Preview Blink")).toBeInTheDocument()

    console.log("✅ TokenForm renders correctly")
  })

  test("handles form input changes", () => {
    render(<TokenForm {...mockProps} />)

    const mintInput = screen.getByLabelText(/Mint Address/i)
    const descriptionInput = screen.getByLabelText(/Description/i)

    fireEvent.change(mintInput, { target: { value: "new-mint-address" } })
    fireEvent.change(descriptionInput, { target: { value: "New Description" } })

    expect(mockProps.setMint).toHaveBeenCalledWith("new-mint-address")
    expect(mockProps.setDescription).toHaveBeenCalledWith("New Description")

    console.log("✅ TokenForm handles input changes correctly")
  })

  test("handles commission radio button changes", () => {
    render(<TokenForm {...mockProps} />)

    const noRadio = screen.getByLabelText("No")
    fireEvent.click(noRadio)

    expect(mockProps.setTakeCommission).toHaveBeenCalledWith("no")

    console.log("✅ TokenForm handles commission radio button changes")
  })

  test("shows percentage input when commission is enabled", () => {
    render(<TokenForm {...mockProps} takeCommission="yes" />)

    expect(screen.getByLabelText(/Commission Percentage/i)).toBeInTheDocument()

    console.log("✅ TokenForm shows percentage input when commission is enabled")
  })

  test("hides percentage input when commission is disabled", () => {
    render(<TokenForm {...mockProps} takeCommission="no" />)

    expect(screen.queryByLabelText(/Commission Percentage/i)).not.toBeInTheDocument()

    console.log("✅ TokenForm hides percentage input when commission is disabled")
  })

  test("handles percentage input changes", () => {
    render(<TokenForm {...mockProps} takeCommission="yes" />)

    const percentageInput = screen.getByLabelText(/Commission Percentage/i)
    fireEvent.change(percentageInput, { target: { value: "0.75" } })

    expect(mockProps.setPercentage).toHaveBeenCalledWith(0.75)

    console.log("✅ TokenForm handles percentage input changes")
  })

  test("caps percentage at 1%", () => {
    render(<TokenForm {...mockProps} takeCommission="yes" />)

    const percentageInput = screen.getByLabelText(/Commission Percentage/i)
    fireEvent.change(percentageInput, { target: { value: "2" } })

    expect(mockProps.setPercentage).toHaveBeenCalledWith(1)

    console.log("✅ TokenForm caps percentage at 1%")
  })

  test("shows wallet button when wallet is not connected", () => {
    render(<TokenForm {...mockProps} connected={false} publicKey={null} />)

    expect(screen.getByText("Connect your wallet to generate a Blink")).toBeInTheDocument()

    console.log("✅ TokenForm shows wallet button when wallet is not connected")
  })

  test("calls handlePreview when preview button is clicked", () => {
    render(<TokenForm {...mockProps} showPreview={false} />)

    const previewButton = screen.getByText("Preview Blink")
    fireEvent.click(previewButton)

    expect(mockProps.handlePreview).toHaveBeenCalled()

    console.log("✅ TokenForm calls handlePreview when preview button is clicked")
  })

  test("calls handleSubmit when generate button is clicked", () => {
    render(<TokenForm {...mockProps} showPreview={true} />)

    const generateButton = screen.getByText("Generate Blink")
    fireEvent.click(generateButton)

    expect(mockProps.handleSubmit).toHaveBeenCalled()

    console.log("✅ TokenForm calls handleSubmit when generate button is clicked")
  })
})
