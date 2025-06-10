# Blink Creator App

## One-Stop Solution for Creating Blinks on Solana

Blink Creator is a user-friendly React/Next.js web application that allows you to easily create, customize, and share **Blinks** — interactive, Solana-powered micro-actions that can be shared as unique links.

---

## What is a Blink?

A **Blink** is a simple, customizable Solana blockchain transaction wrapped in a shareable link. It allows users to perform a specific action on Solana — such as sending a small donation — without requiring them to write any code or manage complex blockchain interactions.

With Blink Creator, anyone can generate their own Blink by specifying an icon, title, description, and wallet address, then share it instantly with others. This lowers the barrier for engaging with blockchain-powered interactions and makes Solana accessible to everyone.

---

## Features

- **Wallet Integration:** Connect your Solana wallet seamlessly.
- **Custom Blink Creation:** Add a title, description, and an image/icon.
- **One-Click Transaction:** Automatically create and sign Solana transactions.
- **Instant Blink Link Generation:** Share your Blink via a unique URL.
- **Copy & Share:** Easily copy the Blink link or share it on social media.
- **Responsive UI:** Works well on both desktop and mobile devices.

---

## Getting Started

Follow these steps to set up the project locally:

### 1. Clone the Repository

```bash
https://github.com/preeeetham/Blinker.git
```

### 2. Navigate into the Project Directory

```bash
cd blinker
```

### 3. Install Dependencies

Make sure you have Node.js installed (version 18 or above is recommended).

```bash
npm install
```

### 4. Configure Environment Variables

Create a `.env` file in the root of the project with the following variables:

```env
SOLANA_RPC_SERVER=""   # Your Solana RPC endpoint URL
WALLET=""    # Default wallet public key
METEORA_IMAGE=""    # (Optional) URL for Meteora image
MONGODB_URI=""  # MongoDB connection URI
```

### 5. Run the Development Server

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser to see the app in action.

---

## How to Use

1. Connect your Solana wallet using the wallet button.
2. Fill in the Blink details:
   - **Title:** A short title for your Blink.
   - **Image URL:** An icon or image to represent your Blink.
   - **Description:** A brief description of the action or purpose.
3. Click **Generate Blink** to create your Blink and send the transaction.
4. Once the transaction confirms, you will get a unique Blink link.
5. Copy the link or share it on social platforms directly from the app.

---

## Tech Stack

- **Next.js** for server-side rendering and React framework.
- **Solana Wallet Adapter** for wallet connection and transaction management.
- **Solana Web3.js SDK** to interact with Solana blockchain.
- **MongoDB Atlas** as the database to store Blink data.
- **React Icons** for UI icons.
- **Tailwind CSS** for styling (optional if you use it).

---

## Troubleshooting

- Ensure your wallet is properly connected and has enough SOL to pay for transaction fees.
- Make sure your `.env` variables are correctly set.
- Check your network connection to the Solana RPC server.
- For database issues, verify the MongoDB URI and network permissions.

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you find bugs or want to add features.

---

## License

This project is open source and available under the [MIT License](LICENSE).
