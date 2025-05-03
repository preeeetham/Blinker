const { Connection, PublicKey, Keypair, Transaction, SystemProgram, LAMPORTS_PER_SOL, TransactionInstruction } = require('@solana/web3.js')
const fs = require('fs')

// Load the test wallet
const walletData = JSON.parse(fs.readFileSync('./test-wallet.json', 'utf8'))
const secretKey = new Uint8Array(walletData.secretKey)
const keypair = Keypair.fromSecretKey(secretKey)

console.log('🔑 Using test wallet:', walletData.publicKey)

// Connect to Solana mainnet-beta
const connection = new Connection('https://api.mainnet-beta.solana.com')

async function testPayment() {
  try {
    // Get the latest blockhash
    const { blockhash } = await connection.getLatestBlockhash()
    
    // Create a test transaction that simulates payment
    const transaction = new Transaction()
    
    // Add a memo instruction with the expected nonce
    const blinkId = 'cmfarek4k000013e2hix8u7zc' // The donation blink ID
    const nonce = walletData.publicKey + blinkId
    
    // Create memo instruction
    const memoInstruction = new TransactionInstruction({
      keys: [],
      programId: new PublicKey('MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr'),
      data: Buffer.from(nonce, 'utf8')
    })
    
    // Add transfer instruction (0.1 SOL = 0.1 * LAMPORTS_PER_SOL)
    const transferInstruction = SystemProgram.transfer({
      fromPubkey: keypair.publicKey,
      toPubkey: new PublicKey('HMdzGauLv7s8LQbuEwJvqFEM3Za1VRC2h6Jdq7nVT7YX'), // Treasury wallet
      lamports: 0.1 * LAMPORTS_PER_SOL
    })
    
    transaction.add(memoInstruction)
    transaction.add(transferInstruction)
    transaction.recentBlockhash = blockhash
    transaction.feePayer = keypair.publicKey
    
    // Sign and send the transaction
    console.log('📤 Sending test payment transaction...')
    const signature = await connection.sendTransaction(transaction, [keypair])
    
    console.log('✅ Transaction sent!')
    console.log('📋 Transaction signature:', signature)
    console.log('🔗 View on Solana Explorer: https://explorer.solana.com/tx/' + signature + '?cluster=mainnet-beta')
    
    // Wait for confirmation
    console.log('⏳ Waiting for confirmation...')
    await connection.confirmTransaction(signature)
    
    console.log('✅ Transaction confirmed!')
    
    // Now test the order endpoint
    console.log('\n🧪 Testing order endpoint...')
    const orderResponse = await fetch('http://localhost:3000/api/actions/order', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        signature: signature,
        orderId: blinkId
      })
    })
    
    const orderResult = await orderResponse.json()
    console.log('📋 Order response:', orderResult)
    
    if (orderResult.blinkLink) {
      console.log('🎉 Payment successful! Blink is now active!')
      console.log('🔗 Blink link:', orderResult.blinkLink)
    } else {
      console.log('❌ Payment failed:', orderResult.error)
    }
    
  } catch (error) {
    console.error('❌ Test payment error:', error.message)
  }
}

testPayment()
