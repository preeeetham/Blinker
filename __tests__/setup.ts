import type React from "react"
// This file sets up the testing environment
import "@testing-library/jest-dom"
import { TextDecoder, TextEncoder } from "util"
import { config } from "dotenv"
import { mockDeep } from "jest-mock-extended"
import { jest } from "@jest/globals"

// Load environment variables from .env file
config()

// Mock global objects that aren't available in Node.js
global.TextEncoder = TextEncoder
global.TextDecoder = TextDecoder

// Mock fetch
global.fetch = jest.fn()

// Mock window.alert
global.alert = jest.fn()

// Mock console methods for testing
const originalConsoleLog = console.log
const originalConsoleError = console.error

console.log = jest.fn((...args) => {
  originalConsoleLog(...args)
})

console.error = jest.fn((...args) => {
  originalConsoleError(...args)
})

// Mock localStorage
Object.defineProperty(window, "localStorage", {
  value: {
    getItem: jest.fn(),
    setItem: jest.fn(),
    removeItem: jest.fn(),
    clear: jest.fn(),
  },
})

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe = jest.fn()
  unobserve = jest.fn()
  disconnect = jest.fn()
}

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  constructor() {}
  observe = jest.fn()
  unobserve = jest.fn()
  disconnect = jest.fn()
}

// Mock clipboard API
Object.defineProperty(navigator, "clipboard", {
  value: {
    writeText: jest.fn().mockImplementation(() => Promise.resolve()),
  },
})

// Mock MongoDB
jest.mock("@/lib/mongodb", () => {
  const mockClient = mockDeep<any>()
  const mockDb = mockDeep<any>()
  const mockCollection = mockDeep<any>()

  mockClient.connect.mockResolvedValue(mockClient)
  mockClient.db.mockReturnValue(mockDb)
  mockDb.collection.mockReturnValue(mockCollection)

  return mockClient
})

// Mock Solana web3.js
jest.mock("@solana/web3.js", () => {
  return {
    Connection: jest.fn().mockImplementation(() => ({
      getLatestBlockhash: jest.fn().mockResolvedValue({ blockhash: "test-blockhash", lastValidBlockHeight: 123 }),
      getBalance: jest.fn().mockResolvedValue(1000000000), // 1 SOL in lamports
      confirmTransaction: jest.fn().mockResolvedValue({ value: { err: null } }),
      simulateTransaction: jest.fn().mockResolvedValue({ value: { err: null, accounts: [{ lamports: 900000000 }] } }),
      getAddressLookupTable: jest.fn().mockResolvedValue({ value: null }),
    })),
    PublicKey: jest.fn().mockImplementation((key) => ({
      toString: () => key,
      toBuffer: () => Buffer.from(key),
    })),
    Transaction: jest.fn().mockImplementation(() => ({
      add: jest.fn(),
      recentBlockhash: "",
      feePayer: null,
      serialize: jest.fn().mockReturnValue(Buffer.from("serialized-transaction")),
    })),
    TransactionInstruction: jest.fn(),
    SystemProgram: {
      transfer: jest.fn().mockReturnValue({}),
    },
    LAMPORTS_PER_SOL: 1000000000,
    clusterApiUrl: jest.fn().mockReturnValue("https://api.devnet.solana.com"),
    VersionedTransaction: {
      deserialize: jest.fn().mockReturnValue({
        message: {
          addressTableLookups: [],
          instructions: [],
        },
      }),
    },
    TransactionMessage: {
      decompile: jest.fn().mockReturnValue({
        instructions: [],
        compileToLegacyMessage: jest.fn().mockReturnValue({}),
      }),
    },
  }
})

// Mock wallet adapter
jest.mock("@solana/wallet-adapter-react", () => ({
  useWallet: jest.fn().mockReturnValue({
    publicKey: { toString: () => "test-public-key" },
    connected: true,
    sendTransaction: jest.fn().mockResolvedValue("test-signature"),
  }),
  ConnectionProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  WalletProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

jest.mock("@solana/wallet-adapter-react-ui", () => ({
  WalletModalProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  WalletMultiButton: () => <button>Connect Wallet</button>,
}))

// Mock SPL token
jest.mock("@solana/spl-token", () => ({
  TOKEN_PROGRAM_ID: { toBuffer: () => Buffer.from("token-program") },
  ASSOCIATED_TOKEN_PROGRAM_ID: { toBuffer: () => Buffer.from("associated-token-program") },
  getMint: jest.fn().mockResolvedValue({ decimals: 9 }),
}))

// Mock Metaplex
jest.mock("@metaplex/js", () => ({
  programs: {
    metadata: {
      Metadata: {
        getPDA: jest.fn().mockResolvedValue("metadata-pda"),
        load: jest.fn().mockResolvedValue({
          data: {
            data: {
              name: "Test Token",
              uri: "https://test-uri.com",
            },
          },
        }),
      },
    },
  },
}))

// Mock SPL token registry
jest.mock("@solana/spl-token-registry", () => ({
  TokenListProvider: jest.fn().mockImplementation(() => ({
    resolve: jest.fn().mockResolvedValue({
      getList: jest.fn().mockReturnValue([
        {
          address: "test-mint-address",
          name: "Test Token",
          logoURI: "https://test-logo.com",
        },
      ]),
    }),
  })),
}))

// Mock Solana Actions
jest.mock("@solana/actions", () => ({
  ACTIONS_CORS_HEADERS: { "Access-Control-Allow-Origin": "*" },
  createPostResponse: jest.fn().mockImplementation((options) => options),
}))

// Mock next/navigation
jest.mock("next/navigation", () => ({
  usePathname: jest.fn().mockReturnValue("/"),
  useRouter: jest.fn().mockReturnValue({
    push: jest.fn(),
    replace: jest.fn(),
    back: jest.fn(),
  }),
}))

// Mock ObjectId
jest.mock("mongodb", () => ({
  ObjectId: jest.fn().mockImplementation((id) => ({
    toString: () => id || "test-object-id",
  })),
  isValid: jest.fn().mockReturnValue(true),
}))
