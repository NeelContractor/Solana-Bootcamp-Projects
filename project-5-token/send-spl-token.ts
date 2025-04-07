import "dotenv/config";
import {
  getExplorerLink,
  getKeypairFromEnvironment,
} from "@solana-developers/helpers";
import bs58 from "bs58"
import { Connection, Keypair, PublicKey, clusterApiUrl } from "@solana/web3.js";
import { getOrCreateAssociatedTokenAccount, transfer } from "@solana/spl-token";

(async() => {
    
    const connection = new Connection(clusterApiUrl("devnet"));

    const sender = Keypair.fromSecretKey(bs58.decode(process.env.SECRET_KEY!));

    console.log(
    `🔑 Loaded our keypair securely, using an env file! Our public key is: ${sender.publicKey.toBase58()}`
    );

    const recipient = new PublicKey("YOUR_RECIPIENT_HERE");

    const tokenMintAccount = new PublicKey("B9WPobhKKiXXTRd3A2uJtZDnuHoPjxA45UmQ3KCc3XKc");

    const MINOR_UNITS_PER_MAJOR_UNITS = Math.pow(10, 2);

    console.log(`💸 Attempting to send 1 token to ${recipient.toBase58()}...`);

    const sourceTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    sender,
    tokenMintAccount,
    sender.publicKey
    );

    const destinationTokenAccount = await getOrCreateAssociatedTokenAccount(
    connection,
    sender,
    tokenMintAccount,
    recipient
    );


    const signature = await transfer(
    connection,
    sender,
    sourceTokenAccount.address,
    destinationTokenAccount.address,
    sender,
    1 * MINOR_UNITS_PER_MAJOR_UNITS
    );

    const explorerLink = getExplorerLink("transaction", signature, "devnet");

    console.log(`✅ Transaction confirmed, explorer link is: ${explorerLink}!`);

})()
