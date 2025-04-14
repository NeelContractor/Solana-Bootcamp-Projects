import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Tokenlottery } from '../target/types/tokenlottery'
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'

describe('tokenlottery', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const wallet = provider.wallet as anchor.Wallet

  const program = anchor.workspace.Tokenlottery as Program<Tokenlottery>

  it('Should Initialize', async () => {
    const initConfigIx = await program.methods.initializeConfig(
      new anchor.BN(0),
      new anchor.BN(1844615497),
      new anchor.BN(10000),
    ).instruction();

    const blockhashWithContext = await provider.connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: blockhashWithContext.blockhash,
      lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight
    }).add(initConfigIx);

    const signature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [wallet.payer]);
    console.log("Your Transaction Signature: ", signature);

    const initLotteryIx = await program.methods.initializeLottery().accounts({
      tokenProgram: TOKEN_PROGRAM_ID
    }).instruction();

    const initLotteryTx = new anchor.web3.Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: blockhashWithContext.blockhash,
        lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight
      }
    ).add(initLotteryIx);

    const initLotterySignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, initLotteryTx, [wallet.payer], { skipPreflight: true });
    console.log("initialize Lottery Signature: ", initLotterySignature);
  })

})
