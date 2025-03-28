import { getExplorerLink } from "@solana-developers/helpers";
import { createMint } from "@solana/spl-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import dotenv from "dotenv";
dotenv.config();

(async() => {
    const connection = new Connection("https://api.devnet.solana.com", "confirmed");
    let key = bs58.decode(process.env.SECRET_KEY!);
    const user = Keypair.fromSecretKey(key);

    const tokenMint = await createMint(connection, user, user.publicKey, null, 2);

    const link = getExplorerLink("address", tokenMint.toString(), "devnet");
    console.log(`✅ Success! Created token mint: ${link}`);
})();