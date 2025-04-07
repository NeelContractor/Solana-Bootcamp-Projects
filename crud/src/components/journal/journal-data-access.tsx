'use client'

import { JOURNAL_PROGRAM_ID as programId, getJournalProgram, JournalIDL } from '@project/anchor'
import { useConnection } from '@solana/wallet-adapter-react'
import { useMutation, useQuery } from '@tanstack/react-query'

import toast from 'react-hot-toast'
import { useCluster } from '../cluster/cluster-data-access'
import { useAnchorProvider } from '../solana/solana-provider'
import { useTransactionToast } from '../ui/ui-layout'
import { PublicKey } from '@solana/web3.js'

interface CreateEntryArgs {
  owner: PublicKey;
  title: string;
  message: string;
}

export function useJournalProgram() {
  const { connection } = useConnection()
  const { cluster } = useCluster()
  const transactionToast = useTransactionToast()
  const provider = useAnchorProvider()
  const program = getJournalProgram(provider)

  const accounts = useQuery({
    queryKey: ['journal', 'all', { cluster }],
    queryFn: () => program.account.journalEntryState.all()
  })

  const getProgramAccount = useQuery({
    queryKey: ['get-program-account', { cluster }],
    queryFn: () => connection.getParsedAccountInfo(programId),
  })

  const createEntry = useMutation<string, Error, CreateEntryArgs>({
    mutationKey: ['jounralEntry', 'create', { cluster }],
    mutationFn: async ({ title, message, owner }) => {
      const [journalEntryAddress] = await PublicKey.findProgramAddressSync(
        [Buffer.from(title), owner.toBuffer()],
        programId
      );

      return program.methods.createJournalEntry(title, message).rpc();
    },
    onSuccess: (signature) => {
      transactionToast(signature);
      accounts.refetch();
    },
    onError: (error) => toast.error(`Failed to create journal entry: ${error.message}`),
  })

  return {
    program,
    programId,
    accounts,
    getProgramAccount,
    createEntry,
  }
}

export function useJournalProgramAccount({ account }: { account: PublicKey }) {
  const { cluster } = useCluster();
  const transactionToast = useTransactionToast();
  const { program, accounts } = useJournalProgram();

  const accountQuery = useQuery({
    queryKey: ["journal", "fetch", { cluster, account }],
    queryFn: () => program.account.journalEntryState.fetch(account),
  });

  const updateEntry = useMutation<string, Error, CreateEntryArgs>({
    mutationKey: ['journalEntry', 'update', { cluster }],
    mutationFn: async ({ title, message, owner }) => {
      const [journalEntryAddress] = await PublicKey.findProgramAddressSync(
        [Buffer.from(title), owner.toBuffer()],
        programId
      );

      return program.methods.updateJournalEntry(title, message).rpc();
    },
    onSuccess: (signature) => {
      transactionToast(signature);
      accounts.refetch();
    },
    onError: (error) => {
      toast.error(`Failed to update journal entry: ${error.message}`);
    }
  });
  
  const deleteEntry = useMutation({
    mutationKey: ["journal", "deleteEntry", { cluster, account }],
    mutationFn: (title: string) => 
    program.methods.deleteJournalEntry(title).rpc(),
    onSuccess: (tx) => {
      transactionToast(tx);
      return accounts.refetch();
    }
  })

  return {
    accountQuery,
    updateEntry,
    deleteEntry
  }
}