// Here we export some useful types and functions for interacting with the Anchor program.
import { AnchorProvider, Program } from '@coral-xyz/anchor'
import { PublicKey } from '@solana/web3.js'
import JournalIDL from '../target/idl/journal.json'
import type { Journal } from '../target/types/journal'

// Re-export the generated IDL and type
export { Journal, JournalIDL }

// The programId is imported from the program IDL.
export const JOURNAL_PROGRAM_ID = new PublicKey(JournalIDL.address)

// This is a helper function to get the Basic Anchor program.
export function getJournalProgram(provider: AnchorProvider) {
  return new Program(JournalIDL as Journal, provider)
}
