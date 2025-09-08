const { Connection, PublicKey, Keypair } = require('@solana/web3.js');
const fs = require('fs');

// Load the test wallet
const walletData = JSON.parse(fs.readFileSync('./test-wallet.json', 'utf8'));
const secretKey = new Uint8Array(walletData.secretKey);
const keypair = Keypair.fromSecretKey(secretKey);

console.log('🔑 Wallet address:', walletData.publicKey);

// Check balance on both networks
async function checkBalances() {
  try {
    // Check mainnet balance
    const mainnetConnection = new Connection('https://api.mainnet-beta.solana.com');
    const mainnetBalance = await mainnetConnection.getBalance(keypair.publicKey);
    console.log('💰 Mainnet balance:', mainnetBalance / 1e9, 'SOL');
    
    // Check devnet balance
    const devnetConnection = new Connection('https://api.devnet.solana.com');
    const devnetBalance = await devnetConnection.getBalance(keypair.publicKey);
    console.log('💰 Devnet balance:', devnetBalance / 1e9, 'SOL');
    
    if (mainnetBalance === 0) {
      console.log('\n⚠️  No mainnet SOL found!');
      console.log('📝 You need to fund this wallet with mainnet SOL to test payments.');
      console.log('🔗 Get mainnet SOL from: https://faucet.solana.com/ (select mainnet)');
      console.log('   Or use a DEX to swap devnet SOL to mainnet SOL');
    } else {
      console.log('\n✅ Wallet has mainnet SOL! Ready to test payments.');
    }
    
  } catch (error) {
    console.error('❌ Error checking balances:', error.message);
  }
}

checkBalances();
