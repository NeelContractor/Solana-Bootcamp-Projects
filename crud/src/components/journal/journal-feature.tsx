'use client'

import { useWallet } from '@solana/wallet-adapter-react'
import { ExplorerLink } from '../cluster/cluster-ui'
import { WalletButton } from '../solana/solana-provider'
import { AppHero, ellipsify } from '../ui/ui-layout'
import { useJournalProgram } from './journal-data-access'
import { JournalCreate, JournalProgram } from './journal-ui'

export default function JournalFeature() {
  const { publicKey } = useWallet()
  const { programId } = useJournalProgram()

  return publicKey ? (
    <div>
      <AppHero title="My Solana Journal" subtitle={'Create your journal here!'}>
        <p className="mb-6">
          <ExplorerLink path={`account/${programId}`} label={ellipsify(programId.toString())} />
        </p>
        <JournalCreate />
      </AppHero>
      <JournalProgram />
    </div>
  ) : (
    <div className="max-w-4xl mx-auto">
      <div className="hero py-[64px]">
        <div className="hero-content text-center">
          <WalletButton className="btn btn-primary" />
        </div>
      </div>
    </div>
  )
}
