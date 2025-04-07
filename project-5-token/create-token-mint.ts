import { getExplorerLink } from "@solana-developers/helpers";
import { getOrCreateAssociatedTokenAccount, mintTo } from "@solana/spl-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js"
import bs58 from "bs58";
import "dotenv/config";

(async() => {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    const MINOR_UNITS_PER_MAJOR_UNITS = Math.pow(10, 2);
    let key = bs58.decode(process.env.SECRET_KEY!);
    const user = Keypair.fromSecretKey(key);

    const tokenMintAccount = new PublicKey("B9WPobhKKiXXTRd3A2uJtZDnuHoPjxA45UmQ3KCc3XKc");

    const recipientAssociatedTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection, user, tokenMintAccount, user.publicKey
    )

    const transactionSignature = await mintTo(
        connection,
        user,
        tokenMintAccount,
        recipientAssociatedTokenAccount.address,
        user,
        10 * MINOR_UNITS_PER_MAJOR_UNITS
    )

    const link = getExplorerLink("transaction", transactionSignature, "devnet");

    console.log(`✅ Success! Mint Token Transaction: ${link}`);
})()