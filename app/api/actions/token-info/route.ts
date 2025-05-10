import { NextResponse } from "next/server"
import { Connection, PublicKey } from "@solana/web3.js"
import { programs } from "@metaplex/js"
import { TokenListProvider } from "@solana/spl-token-registry"

const {
  metadata: { Metadata },
} = programs

const connection = new Connection(process.env.NEXT_PUBLIC_SOLANA_RPC || "https://api.mainnet-beta.solana.com")

export async function GET(req: Request) {
  try {
    // Extract mint address from URL query parameters
    const { searchParams } = new URL(req.url)
    const mint = searchParams.get("mint") // Get the "mint" query parameter

    if (!mint) {
      return NextResponse.json({ error: "Mint address is required" }, { status: 400 })
    }

    const mintAddress = new PublicKey(mint)
    let tokenMetadata
    try {
      const metadataPDA = await Metadata.getPDA(mintAddress)
      tokenMetadata = await Metadata.load(connection, metadataPDA)
    } catch (error) {
      console.log("Error fetching token metadata:", error)
      // return NextResponse.json({ error: 'Failed to fetch token metadata' }, { status: 500 });
    }

    if (!tokenMetadata || !tokenMetadata.data || !tokenMetadata.data.data || !tokenMetadata.data.data.uri) {
      try {
        const tokenInfo = await getTokenInfoFromRegistry(mint)
        const icon = tokenInfo.image
        const title = "BUY " + tokenInfo.name
        const retPayLoad = { icon, title }
        return NextResponse.json(retPayLoad)
      } catch (error) {
        console.log("Error fetching data:", error)
        return NextResponse.json({ error: "Token Info not found" }, { status: 500 })
      }
    }

    const tokenName = tokenMetadata.data.data.name
    const tokenUri = tokenMetadata.data.data.uri

    let tokenJson
    try {
      const response = await fetch(tokenUri)
      tokenJson = await response.json()
      console.log(tokenJson)
    } catch (error) {
      console.log("Error fetching token metadata JSON:", error)
      return NextResponse.json({ error: "Failed to fetch token metadata JSON" }, { status: 500 })
    }

    const icon = tokenJson.image
    const title = "BUY " + tokenName

    const retPayLoad = { icon, title }
    return NextResponse.json(retPayLoad)
  } catch (error) {
    console.log("Error fetching data:", error)
    return NextResponse.json({ error: "Invalid mint" }, { status: 500 })
  }
}

async function getTokenInfoFromRegistry(mintAddress: string) {
  console.log("Fetching token info from registry")
  const tokenList = await new TokenListProvider().resolve()
  const tokens = tokenList.getList()

  // Find token by mint address
  const tokenInfo = tokens.find((t) => t.address === mintAddress)
  if (!tokenInfo) {
    throw new Error("Token not found in registry")
  }

  console.log(`Token Name: ${tokenInfo.name}`)
  console.log(`Token Image: ${tokenInfo.logoURI}`)
  return { name: tokenInfo.name, image: tokenInfo.logoURI }
}
