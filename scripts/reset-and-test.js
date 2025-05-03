const { PrismaClient } = require('@prisma/client')

const prisma = new PrismaClient()

async function resetAndTest() {
  try {
    console.log('🔄 Resetting blink to unpaid status...')
    
    // Reset the blink to unpaid
    await prisma.blink.update({
      where: { id: 'cmfarek4k000013e2hix8u7zc' },
      data: { isPaid: false }
    })
    
    console.log('✅ Blink reset to unpaid status')
    
    // Test the order endpoint with the devnet transaction
    console.log('\n🧪 Testing order endpoint with devnet transaction...')
    const response = await fetch('http://localhost:3000/api/actions/order', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        signature: '4EKZVBqQb5DkhRMyGQGpULD97TNax8HBdML5KF2Dw8YK1kdK5g1TprgtGtRKWkoHq1nLcs7xidRT2jGZX3s2VK2G',
        orderId: 'cmfarek4k000013e2hix8u7zc'
      })
    })
    
    const result = await response.json()
    console.log('📋 Order response:', result)
    
    if (result.blinkLink) {
      console.log('🎉 Payment successful! Blink is now active!')
      console.log('🔗 Blink link:', result.blinkLink)
      
      // Test the donate endpoint
      console.log('\n🧪 Testing donate endpoint...')
      const donateResponse = await fetch('http://localhost:3000/api/actions/donate/cmfarek4k000013e2hix8u7zc')
      const donateData = await donateResponse.json()
      
      if (donateData.icon && donateData.label) {
        console.log('✅ Donate endpoint working! Blink is active!')
      } else {
        console.log('❌ Donate endpoint failed:', donateData.message)
      }
    } else {
      console.log('❌ Payment failed:', result.error)
    }
    
  } catch (error) {
    console.error('❌ Reset and test error:', error.message)
  } finally {
    await prisma.$disconnect()
  }
}

resetAndTest()
