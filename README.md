# Blinker - Solana Blink Creator

## 🚀 One-Stop Solution for Creating Blinks on Solana

Blinker is a powerful and user-friendly React/Next.js web application that enables you to effortlessly create, customize, and share **Blinks** — interactive, Solana-powered micro-actions that can be shared as unique links. Transform your Solana interactions into shareable experiences with just a few clicks!

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
git clone https://github.com/preeeetham/Blinker.git
```

### 2. Navigate into the Project Directory

```bash
cd Blinker
```

### 3. Install Dependencies

Make sure you have Node.js installed (version 18 or above is recommended).

```bash
npm install
```

### 4. Configure Environment Variables

Create a `.env` file in the root of the project with the following variables:

```env
DATABASE_URL="postgresql://username:password@localhost:5432/blinker_db"  # PostgreSQL database connection string
SOLANA_RPC="https://api.mainnet-beta.solana.com"  # Your Solana RPC endpoint URL
WALLET="HMdzGauLv7s8LQbuEwJvqFEM3Za1VRC2h6Jdq7nVT7YX"  # Default treasury wallet public key
NODE_ENV="development"  # Environment mode (development/production)
```

### 5. Set up the Database

Initialize and migrate your PostgreSQL database:

```bash
npx prisma generate
npx prisma db push
```

### 6. Run the Development Server

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
- **PostgreSQL** with **Prisma ORM** for database management and Blink data storage.
- **React Icons** and **Lucide React** for UI icons.
- **Tailwind CSS** for modern, responsive styling.
- **Radix UI** for accessible UI components.
- **Framer Motion** for smooth animations.

---

## Troubleshooting

- Ensure your wallet is properly connected and has enough SOL to pay for transaction fees.
- Make sure your `.env` variables are correctly set.
- Check your network connection to the Solana RPC server.
- For database issues, verify the PostgreSQL DATABASE_URL and ensure the database is running.
- Run `npx prisma db push` if you encounter database schema issues.

---

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you find bugs or want to add features.

---

## License

This project is open source and available under the [MIT License](LICENSE).
