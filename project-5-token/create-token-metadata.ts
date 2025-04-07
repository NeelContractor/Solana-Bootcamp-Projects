import { getExplorerLink } from "@solana-developers/helpers";
import { createCreateNativeMintInstruction, createInitializeMetadataPointerInstruction, createMint, createUpdateMetadataPointerInstruction } from "@solana/spl-token";
import { Connection, Keypair, PublicKey, sendAndConfirmTransaction, Transaction } from "@solana/web3.js";
import {
    DataV2,
    createCreateMetadataAccountV3Instruction,
  } from "@metaplex-foundation/mpl-token-metadata";
  
import bs58 from "bs58";
import dotenv from "dotenv";
dotenv.config();

(async() => {

    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    let key = bs58.decode(process.env.SECRET_KEY!);
    const user = Keypair.fromSecretKey(key);

    const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

    const tokenMintAccount = new PublicKey("B9WPobhKKiXXTRd3A2uJtZDnuHoPjxA45UmQ3KCc3XKc");

    const metadataData: DataV2 = {
        name: "Solana Training Token",
        symbol: "TRAINING",
        uri: "https://raw.githubusercontent.com/solana-developers/professional-education/main/labs/sample-token-metadata.json",
        sellerFeeBasisPoints: 0,
        creators: null,
        collection: null,
        uses: null,
    }

    const metadataPDAAndBump = PublicKey.findProgramAddressSync(
        [
            Buffer.from("metadata"),
            TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            tokenMintAccount.toBuffer()
        ],
        TOKEN_METADATA_PROGRAM_ID
    );

    const metadataPDA = metadataPDAAndBump[0];

    const transaction = new Transaction();

    const createMetadataAccountInstruction = createInitializeMetadataPointerInstruction(
        {
            metadata: metadataPDA,
            mint: tokenMintAccount,
            mintAuthority: user.publicKey,
            payer: user,
            updateAuthority: user.publicKey 
        },
        {
            data: metadataData,
            isMutable: true,
            collectionDetails: null
        }
    );

    transaction.add(createMetadataAccountInstruction)

    const transactionSignature = await sendAndConfirmTransaction(
        connection,
        transaction,
        [user]
      );
      
      const transactionLink = getExplorerLink(
        "transaction",
        transactionSignature,
        "devnet"
      );
      
      console.log(`✅ Transaction confirmed, explorer link is: ${transactionLink}!`);
      
      const tokenMintLink = getExplorerLink(
        "address",
        tokenMintAccount.toString(),
        "devnet"
      );
      
      console.log(`✅ Look at the token mint again: ${tokenMintLink}!`);

})();