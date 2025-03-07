import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Crud } from '../target/types/crud'

describe('crud', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env())

  const program = anchor.workspace.Crud as Program<Crud>

  it('should run the program', async () => {
    // Add your test here.
    const tx = await program.methods.greet().rpc()
    console.log('Your transaction signature', tx)
  })
})
