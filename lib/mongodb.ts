import { MongoClient } from 'mongodb';

if (!process.env.MONGODB_URI) {
  throw new Error('Invalid/Missing environment variable: "MONGODB_URI"');
}

const uri = process.env.MONGODB_URI;
const options = {
  ssl: true,
  useUnifiedTopology: true,
  connectTimeoutMS: 30000,
};

let client: MongoClient;
let clientPromise: Promise<MongoClient>;

try {
  client = new MongoClient(uri, options);
  clientPromise = client.connect();
} catch (error) {
  console.error('MongoDB connection error:', error);
  throw error;
}

export default clientPromise;