import { createNft, fetchDigitalAsset, mplTokenMetadata } from "@metaplex-foundation/mpl-token-metadata";
import { generateSigner, keypairIdentity, percentAmount, publicKey } from "@metaplex-foundation/umi";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { airdropIfRequired, getExplorerLink } from "@solana-developers/helpers";
import { Connection, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import bs58 from "bs58"
import "dotenv/config"

(async() => {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const user = Keypair.fromSecretKey(bs58.decode(process.env.SECRET_KEY!));

    await airdropIfRequired(
        connection, user.publicKey, 1 * LAMPORTS_PER_SOL, 0.5 * LAMPORTS_PER_SOL
    )

    const umi = createUmi(connection.rpcEndpoint);
    umi.use(mplTokenMetadata());

    const umiUser = umi.eddsa.createKeypairFromSecretKey(user.secretKey);
    umi.use(keypairIdentity(umiUser));

    const collectionAddress = publicKey("")

    const mint = generateSigner(umi)

    const transaction = await createNft(umi, {
        mint,
        name: "My NFT",
        uri: "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-nft-offchain-data.json",
        sellerFeeBasisPoints: percentAmount(0),
        collection: {
          key: collectionAddress,
          verified: false,
        },
      });
      
      await transaction.sendAndConfirm(umi);
      
      const createdNft = await fetchDigitalAsset(umi, mint.publicKey);
      
      console.log(
        `🖼️ Created NFT! Address is ${getExplorerLink(
          "address",
          createdNft.mint.publicKey,
          "devnet"
        )}`
      );

})()