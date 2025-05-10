import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import Form from "@/components/form"
import { useWallet } from "@solana/wallet-adapter-react"

// Mock the useWallet hook
jest.mock("@solana/wallet-adapter-react")

// Mock the fetch function
global.fetch = jest.fn()

describe("Form Component", () => {
  const mockProps = {
    icon: "https://example.com/icon.png",
    setIcon: jest.fn(),
    description: "Test description",
    setDescription: jest.fn(),
    title: "Test title",
    setTitle: jest.fn(),
    showForm: true,
    setShowForm: jest.fn(),
  }

  beforeEach(() => {
    jest.clearAllMocks()

    // Mock the useWallet hook
    ;(useWallet as jest.Mock).mockReturnValue({
      publicKey: { toString: () => "test-wallet-address" },
      connected: true,
      sendTransaction: jest.fn().mockResolvedValue("test-signature"),
    })

    // Mock fetch for successful API responses
    ;(global.fetch as jest.Mock).mockImplementation((url) => {
      if (url === "/api/actions/generate-blink") {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ id: "test-blink-id" }),
        })
      } else if (url === "/api/actions/order") {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ blinkLink: "test-blink-link" }),
        })
      }
      return Promise.resolve({
        ok: false,
        json: () => Promise.resolve({}),
      })
    })
  })

  test("renders form correctly when showForm is true", () => {
    render(<Form {...mockProps} />)

    expect(screen.getByText("Customize Your Blink")).toBeInTheDocument()
    expect(screen.getByLabelText(/Title/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/Image URL/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/Description/i)).toBeInTheDocument()
    expect(screen.getByText("Generate Blink")).toBeInTheDocument()

    console.log("✅ Form renders correctly when showForm is true")
  })

  test("renders success view when showForm is false", () => {
    render(<Form {...mockProps} showForm={false} blinkLink="test-blink-link" />)

    expect(screen.getByText("Your Blink is Ready!")).toBeInTheDocument()
    expect(screen.getByText(/Blink Link:/i)).toBeInTheDocument()
    expect(screen.getByText("Share on X")).toBeInTheDocument()
    expect(screen.getByText("Create New Blink")).toBeInTheDocument()

    console.log("✅ Form renders success view when showForm is false")
  })

  test("handles form input changes", () => {
    render(<Form {...mockProps} />)

    const titleInput = screen.getByLabelText(/Title/i)
    const iconInput = screen.getByLabelText(/Image URL/i)
    const descriptionInput = screen.getByLabelText(/Description/i)

    fireEvent.change(titleInput, { target: { value: "New Title" } })
    fireEvent.change(iconInput, { target: { value: "https://example.com/new-icon.png" } })
    fireEvent.change(descriptionInput, { target: { value: "New Description" } })

    expect(mockProps.setTitle).toHaveBeenCalledWith("New Title")
    expect(mockProps.setIcon).toHaveBeenCalledWith("https://example.com/new-icon.png")
    expect(mockProps.setDescription).toHaveBeenCalledWith("New Description")

    console.log("✅ Form handles input changes correctly")
  })

  test("shows wallet button when wallet is not connected", () => {
    ;(useWallet as jest.Mock).mockReturnValue({
      publicKey: null,
      connected: false,
    })

    render(<Form {...mockProps} />)

    expect(screen.getByText("Connect your wallet to generate a Blink")).toBeInTheDocument()

    console.log("✅ Form shows wallet button when wallet is not connected")
  })

  test("handles blink generation successfully", async () => {
    const mockSendTransaction = jest.fn().mockResolvedValue("test-signature")
    ;(useWallet as jest.Mock).mockReturnValue({
      publicKey: { toString: () => "test-wallet-address" },
      connected: true,
      sendTransaction: mockSendTransaction,
    })

    render(<Form {...mockProps} />)

    const generateButton = screen.getByText("Generate Blink")
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(mockProps.setShowForm).toHaveBeenCalledWith(false)
    })

    console.log("✅ Form handles blink generation successfully")
  })

  test("handles API errors during blink generation", async () => {
    ;(global.fetch as jest.Mock).mockImplementationOnce(() =>
      Promise.resolve({
        ok: false,
        json: () => Promise.resolve({ error: "API Error" }),
      }),
    )

    const consoleErrorSpy = jest.spyOn(console, "error").mockImplementation()
    const alertSpy = jest.spyOn(window, "alert").mockImplementation()

    render(<Form {...mockProps} />)

    const generateButton = screen.getByText("Generate Blink")
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled()
      expect(alertSpy).toHaveBeenCalled()
    })

    console.log("✅ Form handles API errors during blink generation")

    consoleErrorSpy.mockRestore()
    alertSpy.mockRestore()
  })

  test("handles transaction errors", async () => {
    const mockSendTransaction = jest.fn().mockRejectedValue(new Error("Transaction Error"))
    ;(useWallet as jest.Mock).mockReturnValue({
      publicKey: { toString: () => "test-wallet-address" },
      connected: true,
      sendTransaction: mockSendTransaction,
    })

    const consoleErrorSpy = jest.spyOn(console, "error").mockImplementation()
    const alertSpy = jest.spyOn(window, "alert").mockImplementation()

    render(<Form {...mockProps} />)

    const generateButton = screen.getByText("Generate Blink")
    fireEvent.click(generateButton)

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled()
      expect(alertSpy).toHaveBeenCalled()
    })

    console.log("✅ Form handles transaction errors")

    consoleErrorSpy.mockRestore()
    alertSpy.mockRestore()
  })
})
