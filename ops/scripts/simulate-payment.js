const { PrismaClient } = require('@prisma/client');

const prisma = new PrismaClient();

async function simulatePayment() {
  try {
    console.log('🧪 Simulating payment for blink...');
    
    // Update the donation blink to paid status
    const result = await prisma.blink.update({
      where: { id: 'cmfarek4k000013e2hix8u7zc' },
      data: { isPaid: true }
    });
    
    console.log('✅ Payment simulated successfully!');
    console.log('📋 Updated blink:', {
      id: result.id,
      title: result.title,
      isPaid: result.isPaid
    });
    
    // Test the donate endpoint now
    console.log('\n🧪 Testing donate endpoint after payment...');
    const response = await fetch('http://localhost:3000/api/actions/contributions/cmfarek4k000013e2hix8u7zc');
    const data = await response.json();
    
    console.log('📋 Donate endpoint response:');
    console.log(JSON.stringify(data, null, 2));
    
    if (data.icon && data.label) {
      console.log('🎉 Donate endpoint is working! Blink is now active!');
    } else {
      console.log('❌ Donate endpoint failed:', data.message);
    }
    
    // Test the tokens endpoint as well
    console.log('\n🧪 Testing tokens endpoint...');
    const tokenResponse = await fetch('http://localhost:3000/api/tokens/cmfareuei000113e2ieoux3xz');
    const tokenData = await tokenResponse.json();
    
    console.log('📋 Tokens endpoint response:');
    console.log(JSON.stringify(tokenData, null, 2));
    
    if (tokenData.message && tokenData.message.includes('not paid')) {
      console.log('✅ Tokens endpoint correctly shows unpaid status');
    } else if (tokenData.icon && tokenData.label) {
      console.log('🎉 Tokens endpoint is working!');
    }
    
  } catch (error) {
    console.error('❌ Error:', error.message);
  } finally {
    await prisma.$disconnect();
  }
}

simulatePayment();
