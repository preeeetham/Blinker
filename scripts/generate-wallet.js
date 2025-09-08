const { Keypair } = require('@solana/web3.js');
const fs = require('fs');

// Generate a new keypair
const keypair = Keypair.generate();

// Get the public key (wallet address)
const publicKey = keypair.publicKey.toString();

// Get the secret key as an array
const secretKey = Array.from(keypair.secretKey);

console.log('🚀 New Solana Test Wallet Generated!');
console.log('='.repeat(50));
console.log('Public Key (Wallet Address):', publicKey);
console.log('='.repeat(50));

// Save the keypair to a file for future use
const walletData = {
  publicKey: publicKey,
  secretKey: secretKey
};

fs.writeFileSync('./test-wallet.json', JSON.stringify(walletData, null, 2));
console.log('✅ Wallet saved to test-wallet.json');
console.log('\n📝 You can fund this wallet with testnet SOL at:');
console.log('https://faucet.solana.com/');
console.log('\n🔑 Keep the test-wallet.json file secure - it contains your private key!');
