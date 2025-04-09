import * as anchor from '@coral-xyz/anchor'
import { Program, BN } from '@coral-xyz/anchor';
import { SYSTEM_PROGRAM_ID } from '@coral-xyz/anchor/dist/cjs/native/system';
import NodeWallet from '@coral-xyz/anchor/dist/cjs/nodewallet';
import { Tokenvesting } from '../target/types/tokenvesting';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { createMint, mintTo } from "spl-token-bankrun"
import { Keypair, PublicKey } from '@solana/web3.js';
import { BankrunProvider } from 'anchor-bankrun';
import { BanksClient, Clock, ProgramTestContext, startAnchor } from 'solana-bankrun';

const IDL = require("../target/idl/tokenvesting.json");

describe('tokenvesting', () => {
  let companyName = "Company";
  let beneficiary: Keypair;
  let context: ProgramTestContext;
  let provider: BankrunProvider;
  let program: Program<Tokenvesting>;
  let banksClient: BanksClient;
  let employer: Keypair;
  let mint: PublicKey;
  let beneficiaryProvider: BankrunProvider;
  let program2: Program<Tokenvesting>;
  let vestingAccountKey: PublicKey;
  let treasuryTokenAccount: PublicKey;
  let employeeAccount: PublicKey;

  beforeAll(async() => {
    beneficiary = new anchor.web3.Keypair();

    context = await startAnchor(
      "", 
      [{name: "tokenvesting", programId: new PublicKey(IDL.address)}],
      [
        {
          address: beneficiary.publicKey,
          info: {
            lamports: 1_000_000_000,
            data: Buffer.alloc(0),
            owner: SYSTEM_PROGRAM_ID,
            executable: false
          }
        }
      ]
    );
    provider = new BankrunProvider(context);
    anchor.setProvider(provider);

    program = new anchor.Program<Tokenvesting>(IDL as Tokenvesting, provider);

    banksClient = context.banksClient;

    employer = provider.wallet.payer;

    // @ts-ignore
    mint = await createMint(banksClient, employer, employer.publicKey, null, 2);

    beneficiaryProvider = new BankrunProvider(context);
    beneficiaryProvider.wallet = new NodeWallet(beneficiary);

    program2 = new Program<Tokenvesting>(IDL as Tokenvesting, beneficiaryProvider);

    [vestingAccountKey] = PublicKey.findProgramAddressSync(
      [Buffer.from(companyName)],
      program.programId
    );

    [treasuryTokenAccount] = PublicKey.findProgramAddressSync(
      [Buffer.from("vesting_treasury"), Buffer.from(companyName)],
      program.programId
    );

    [employeeAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("employee_vesting"),
        beneficiary.publicKey.toBuffer(),
        vestingAccountKey.toBuffer(),
      ],
      program.programId
    )
  });
  
  it('should create a vesting account', async () => {
    const tx = await program.methods
      .createVestingAccount(companyName)
      .accounts({
        signer: employer.publicKey,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: "confirmed" });

      const vestingAccountData = await program.account.vestingAccount.fetch(vestingAccountKey, "confirmed");
      console.log("Vesting Account Data: ", JSON.stringify(vestingAccountData, null, 2));
      console.log("Create Vesting Account Transaction Signature: ", tx);
  })

  it('should fund tje treasury token account', async() => {
    const amount = 10_000 * 10 ** 9;
    const mintTx = await mintTo(
      // @ts-ignore
      banksClient,
      employer,
      mint,
      treasuryTokenAccount,
      employer,
      amount
    );

    console.log("Mint to treasury Transaction Signature: ", mintTx);
  })

  it("should create an empolyee vesting account", async() => {
    const tx2 = await program.methods
      .createEmployeeVesting(new BN(0), new BN(100), new BN(100), new BN(0))
      .accounts({
        beneficiary: beneficiary.publicKey,
        vestingAccount: vestingAccountKey,
      })
      .rpc({ commitment: "confirmed" });

      console.log("Create Employee Account Transaction Signature: ", tx2);
      console.log("Employee account", employeeAccount.toBase58());
  })

  it("should claim tokens", async() => {
    await new Promise((resolve) => setTimeout(resolve, 1000));

    const currentClock = await banksClient.getClock();
    context.setClock(
      new Clock(
      currentClock.slot,
      currentClock.epochStartTimestamp,
      currentClock.epoch,
      currentClock.leaderScheduleEpoch,
      BigInt(1000)
      )
    );

    console.log("Employee account", employeeAccount.toBase58());

    const tx3 = await program2.methods
      .claimTokens(companyName)
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID
      })
      .rpc({ commitment: "confirmed" });

      console.log("Claim Tokens transaction signature", tx3);
  })

})
