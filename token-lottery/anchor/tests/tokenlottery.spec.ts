import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import * as sb from "@switchboard-xyz/on-demand";
import { Tokenlottery } from '../target/types/tokenlottery'
import { TOKEN_PROGRAM_ID } from '@solana/spl-token'
import SwitchboardIDL from "./ondeman-ild.json";

describe('tokenlottery', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider)
  const wallet = provider.wallet as anchor.Wallet

  const program = anchor.workspace.Tokenlottery as Program<Tokenlottery>

  const switchboardProgram = new anchor.Program(SwitchboardIDL as anchor.Idl, provider);
  const rndgKp = anchor.web3.Keypair.generate();

  /* before("load switchboard program", async () => {
      const switchboardIDL = await anchor.Program.fetchIdl(
        sb.ON_DEMAND_MAINNET_PID,
        {connection: new anchor.web3.Connection("https://api.mainnet-beta.solana.com")}
      ) as anchor.Idl;

      var fs

      switchboardProgram = new anchor.Program(switchboardIDL, provider);
  }); */

  async function BuyTicket() {
    const buyTicketIx = await program.methods.buyTicket().accounts({
      tokenProgram: TOKEN_PROGRAM_ID
    }).instruction();

    const blockhashContext = await provider.connection.getLatestBlockhash();

    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 300000
      });

    const priorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1
    })
    
    const tx = new anchor.web3.Transaction({
      feePayer: wallet.payer.publicKey,
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight
    }).add(buyTicketIx)
    .add(computeIx)
    .add(priorityIx);

    const signature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [wallet.payer]);
    console.log("Buy Ticket:", signature);
  }

  it('Should Initialize', async () => {

    const slot = await provider.connection.getSlot();
    const endSlot = slot + 20; 
    
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

    await BuyTicket();
    await BuyTicket();
    await BuyTicket();
    await BuyTicket();
    await BuyTicket();
    await BuyTicket();
    await BuyTicket();

    const queue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w");

    const queueAccount = new sb.Queue(switchboardProgram, queue);
    console.log("Queue Account ", queue.toString());
    try {
      await queueAccount.loadData()
    } catch (e) {
      console.log("Error ", e);
      process.exit(1);
    }

    const [randomness, createRandomnessIx] = await sb.Randomness.create(switchboardProgram, rndgKp, queue);

    const createRandomnessTx = await sb.asV0Tx({
      connection: provider.connection,
      ixs: [createRandomnessIx],
      payer: wallet.publicKey,
      signers: [wallet.payer, rndgKp]
    });

    const createRandomnessSignature = await provider.connection.sendTransaction(createRandomnessTx);
    console.log("createRandomnessSignature ", createRandomnessSignature);

    // let confirmed = false;

    // while(!confirmed) {
    //   try {
    //     const confirmedRandomness = await provider.connection.getSignatureStatuses([createRandomnessSignature]);
    //     const randomnessStatus = confirmedRandomness.value[0];
    //     if (randomnessStatus?.confirmations != null && randomnessStatus.confirmationStatus === 'confirmed') {
    //       confirmed = true
    //     }
    //   } catch (error) {
    //     console.log("error ", error);
    //   }
    // }

    const sbCommitIx = await randomness.commitIx(queue) ;

    const commitIx = await program.methods.commitRandomness()
    .accounts({
      randomnessAccount: randomness.pubkey
    }).instruction();

    const commitComputeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 100000
    });

    const commitPriorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1
    });

    const commitBlockhashWithContext = await provider.connection.getLatestBlockhash();
    const commitTx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: commitBlockhashWithContext.blockhash,
      lastValidBlockHeight: commitBlockhashWithContext.lastValidBlockHeight
    })
    .add(commitComputeIx)
    .add(commitPriorityIx)
    .add(sbCommitIx)
    .add(commitIx)

    const commitSignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, commitTx, [wallet.payer]);
    console.log("commit Signature ", commitSignature);

    const sbRevealIx = await randomness.revealIx();
    const revealWinnerIx = await program.methods.revealWinner().accounts({
      randomnessAccount: randomness.pubkey
    }).instruction();

    const revealBlockhashWithContext = await provider.connection.getLatestBlockhash();

    const revealTx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: revealBlockhashWithContext.blockhash,
      lastValidBlockHeight: revealBlockhashWithContext.lastValidBlockHeight
    }).add(sbRevealIx)
      .add(revealWinnerIx);

    
    let currentSlot = 0;
    while (currentSlot < endSlot) {
      const slot = await provider.connection.getSlot();
      if(slot > currentSlot) {
        currentSlot = slot;
        console.log('Current Slot: ', currentSlot);
      }
    }

    const revealSignature = anchor.web3.sendAndConfirmTransaction(provider.connection, revealTx, [wallet.payer]);
    console.log("reveal signature: ", revealSignature);

  }, 300000)

})
