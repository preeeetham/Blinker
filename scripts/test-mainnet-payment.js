const { Connection, PublicKey, Keypair, Transaction, SystemProgram, LAMPORTS_PER_SOL, TransactionInstruction } = require('@solana/web3.js');
const fs = require('fs');

// The wallet address you provided
const walletAddress = '6KpVthY1cTceiHcsnuYm34kQvcSrzNBQ1PjaTkZ4FZzu';
const blinkId = 'cmfaruixx000213e23ussjwlv';

console.log('🔑 Testing with wallet:', walletAddress);
console.log('📋 Blink ID:', blinkId);

// Connect to Solana mainnet
const connection = new Connection('https://api.mainnet-beta.solana.com');

async function testMainnetPayment() {
  try {
    // Check wallet balance first
    const wallet = new PublicKey(walletAddress);
    const balance = await connection.getBalance(wallet);
    console.log('💰 Wallet balance:', balance / LAMPORTS_PER_SOL, 'SOL');
    
    if (balance === 0) {
      console.log('⚠️  Wallet has no SOL! You need to fund it first.');
      console.log('📝 Fund this wallet: ' + walletAddress);
      console.log('🔗 Get SOL from: https://faucet.solana.com/ (select mainnet)');
      return;
    }
    
    // Get the latest blockhash
    const { blockhash } = await connection.getLatestBlockhash();
    
    // Create a test transaction with the correct amount (0.001 SOL for donate)
    const transaction = new Transaction();
    
    // Add a memo instruction with the expected nonce
    const nonce = walletAddress + blinkId;
    
    // Create memo instruction
    const memoInstruction = new TransactionInstruction({
      keys: [],
      programId: new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'),
      data: Buffer.from(nonce, 'utf8')
    });
    
    // Add transfer instruction with correct amount (0.001 SOL for donate)
    // Using the default treasury wallet from the order route
    const TREASURY_WALLET = new PublicKey('HMdzGauLv7s8LQbuEwJvqFEM3Za1VRC2h6Jdq7nVT7YX');
    
    const transferInstruction = SystemProgram.transfer({
      fromPubkey: wallet,
      toPubkey: TREASURY_WALLET, // Treasury wallet
      lamports: 0.001 * LAMPORTS_PER_SOL // Correct amount for donate
    });
    
    transaction.add(memoInstruction);
    transaction.add(transferInstruction);
    transaction.recentBlockhash = blockhash;
    transaction.feePayer = wallet;
    
    // Serialize the transaction for signing
    const serializedTransaction = transaction.serialize({
      requireAllSignatures: false,
      verifySignatures: false
    });
    
    console.log('📤 Transaction created successfully!');
    console.log('📋 Transaction details:');
    console.log('   - Amount: 0.001 SOL');
    console.log('   - From: ' + walletAddress);
    console.log('   - To: ' + TREASURY_WALLET.toString() + ' (Treasury wallet)');
    console.log('   - Memo: ' + nonce);
    console.log('   - Serialized length:', serializedTransaction.length, 'bytes');
    
    console.log('\n🔐 Transaction needs to be signed by wallet:', walletAddress);
    console.log('📝 You can sign this transaction using:');
    console.log('   1. Phantom wallet');
    console.log('   2. Solflare wallet');
    console.log('   3. Any Solana wallet that supports the transaction');
    
    console.log('\n🧪 After signing and sending, test the order endpoint:');
    console.log('curl -X POST http://localhost:3000/api/actions/order \\');
    console.log('  -H "Content-Type: application/json" \\');
    console.log('  -d \'{"signature": "YOUR_TRANSACTION_SIGNATURE", "orderId": "' + blinkId + '"}\'');
    
  } catch (error) {
    console.error('❌ Error:', error.message);
  }
}

testMainnetPayment();
