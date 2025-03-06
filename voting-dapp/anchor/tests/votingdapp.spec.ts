import * as anchor from '@coral-xyz/anchor'
import {Program} from '@coral-xyz/anchor'
import {PublicKey} from '@solana/web3.js'
import {Votingdapp} from '../target/types/votingdapp'
import { BankrunProvider, startAnchor } from 'anchor-bankrun';

const IDL = require("../target/idl/votingdapp.json");

const votingAddress = new PublicKey("JBE29oQiALL59F3H7u11wGRdUkVuFhAHN32T7jE6rgt4");

describe('Voting', () => {

  let context;
  let provider: BankrunProvider;
  let votingProgram: anchor.Program<Votingdapp>;

  beforeAll(async() => {
    context = await startAnchor("", [{ name: "votingdapp", programId: votingAddress }], []);
    provider = new BankrunProvider(context);

    votingProgram = new Program<Votingdapp>(
      IDL,
      provider
    );
  })

  it('Initialize Poll', async () => {
      await votingProgram.methods.initializePoll(
        new anchor.BN(1),
        "What is your favorite type of peanut butter?",
        new anchor.BN(0),
        new anchor.BN(1840820123),
      ).rpc();

      const [pollAddress] = PublicKey.findProgramAddressSync(
        [new anchor.BN(1).toArrayLike(Buffer, "le", 8)],
        votingAddress
      );

      const poll = await votingProgram.account.poll.fetch(pollAddress);

      console.log(poll);

      expect(poll.pollId.toNumber()).toEqual(1);
      expect(poll.description).toEqual("What is your favorite type of peanut butter?");
      expect(poll.pollStart.toNumber()).toBeLessThan(poll.pollEnd.toNumber());
    })

  it("initialize candidate", async() => {
    await votingProgram.methods.initializeCandidate(
      "Smooth",
      new anchor.BN(1)
    ).rpc();

    await votingProgram.methods.initializeCandidate(
      "Crunchy",
      new anchor.BN(1)
    ).rpc();

    const [crunchyAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Crunchy")],
      votingAddress
    );
    const crunchyCandidate = await votingProgram.account.candidate.fetch(crunchyAddress);
    expect(crunchyCandidate.candidateVotes.toNumber()).toEqual(0);
    console.log(crunchyCandidate);

    const [smoothAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Smooth")],
      votingAddress
    );
    const smoothCandidate = await votingProgram.account.candidate.fetch(smoothAddress);
    expect(smoothCandidate.candidateVotes.toNumber()).toEqual(0);
    console.log(smoothCandidate);


  });

  it("vote", async() => {
    await votingProgram.methods
    .vote(
      "Smooth",
      new anchor.BN(1),
    ).rpc();

    const [smoothAddress] = PublicKey.findProgramAddressSync(
      [new anchor.BN(1).toArrayLike(Buffer, "le", 8), Buffer.from("Smooth")],
      votingAddress
    );
    const smoothCandidate = await votingProgram.account.candidate.fetch(smoothAddress);
    expect(smoothCandidate.candidateVotes.toNumber()).toEqual(1);
    console.log(smoothCandidate);

    const signer = provider.wallet.publicKey;
    const [voterRecord] = PublicKey.findProgramAddressSync(
      [Buffer.from("voter-record"), new anchor.BN(1).toArrayLike(Buffer, 'le', 8), signer.toBytes()],
      votingAddress
    );

    const record = await votingProgram.account.voterRecord.fetch(voterRecord);
    console.log(record);

    
  });
})
