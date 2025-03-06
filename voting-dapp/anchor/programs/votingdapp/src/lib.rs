#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

declare_id!("JBE29oQiALL59F3H7u11wGRdUkVuFhAHN32T7jE6rgt4");

#[program]
pub mod votingdapp {
    use super::*;

    pub fn initialize_poll(ctx: Context<InitializePoll>, poll_id: u64, description: String, poll_start: u64, poll_end: u64) -> Result<()> {
      let poll = &mut ctx.accounts.poll;
      poll.poll_id = poll_id;
      poll.description = description;
      poll.poll_start = poll_start;
      poll.poll_end = poll_end;
      poll.candidate_amount = 0;
      Ok(())
    }

    pub fn initialize_candidate(ctx: Context<InitializeCandidiate>, candidate_name: String, _poll_id: u64) ->Result<()> {
      let candidate = &mut ctx.accounts.candidate;
      let poll = &mut ctx.accounts.poll;
      poll.candidate_amount += 1;
      candidate.candidate_name = candidate_name;
      candidate.candidate_votes = 0;
      Ok(())
    }

    pub fn vote(ctx: Context<Vote>, _candidate_name: String, _poll_id: u64) -> Result<()> {

      let current_time = Clock::get()?.unix_timestamp as u64;
      if current_time < ctx.accounts.poll.poll_start || current_time > ctx.accounts.poll.poll_end {
        return err!(VotingError::PollNotActive);
      }

      let poll = &ctx.accounts.poll.poll_id.to_le_bytes();
      let signer = &ctx.accounts.signer.key();
      let voter_record_seeds = &[
        b"voter-record",
        poll.as_ref(),
        signer.as_ref(),
      ];
      let (voter_record_key, _bump) = Pubkey::find_program_address(voter_record_seeds, ctx.program_id);

      if ctx.accounts.voter_record.key() != voter_record_key {
        return err!(VotingError::InvalidVoterRecord);
      }

      if ctx.accounts.voter_record.has_voted {
        return err!(VotingError::AlreadyVoted);
      }


      let candidate = &mut ctx.accounts.candidate;
      candidate.candidate_votes += 1;

      ctx.accounts.voter_record.has_voted = true;
      
      msg!("Voted for candidate: {}", candidate.candidate_name);
      msg!("Votes: {}", candidate.candidate_votes);
      Ok(())
    }
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct Vote<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    seeds = [poll_id.to_le_bytes().as_ref()],
    bump
  )]
  pub poll: Account<'info, Poll>,

  #[account(
    mut,
    seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
    bump
  )]
  pub candidate: Account<'info, Candidate>,

  #[account(
    init_if_needed,
    payer = signer,
    space = 8 + VoterRecord::INIT_SPACE,
    seeds = [
      b"voter-record",
      poll_id.to_le_bytes().as_ref(),
      signer.key().as_ref()
    ],
    bump
  )]
  pub voter_record: Account<'info, VoterRecord>,

  pub system_program: Program<'info, System>

}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct InitializeCandidiate<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    mut,
    seeds = [poll_id.to_le_bytes().as_ref()],
    bump
  )]
  pub poll: Account<'info, Poll>,

  #[account(
    init,
    payer = signer,
    space = 8 + Candidate::INIT_SPACE,
    seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
    bump
  )]
  pub candidate: Account<'info, Candidate>,

  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    init,
    payer = signer,
    space = 8 + Poll::INIT_SPACE,
    seeds = [poll_id.to_le_bytes().as_ref()],
    bump
  )]
  pub poll: Account<'info, Poll>,

  pub system_program: Program<'info, System>
}

#[account]
#[derive(InitSpace)]
pub struct VoterRecord {
  has_voted: bool
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
  #[max_len(32)]
  candidate_name: String,
  candidate_votes: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Poll {
  pub poll_id: u64,
  #[max_len(280)]
  pub description: String,
  pub poll_start: u64,
  pub poll_end: u64,
  pub candidate_amount: u64,
}

#[error_code]
pub enum VotingError {
  #[msg("You have already voted in this poll")]
    AlreadyVoted,
    #[msg("The poll is not currently active")]
    PollNotActive,
    #[msg("Invalid voter record account")]
    InvalidVoterRecord,
}