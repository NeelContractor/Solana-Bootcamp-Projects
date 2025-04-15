#![allow(clippy::result_large_err)]
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::{
  associated_token::AssociatedToken,  
  token_interface::{Mint, TokenAccount, TokenInterface, mint_to, MintTo},
};
use anchor_spl::metadata::{
  Metadata,
  MetadataAccount,
  create_master_edition_v3, 
  create_metadata_accounts_v3, 
  mpl_token_metadata::types::{CollectionDetails, Creator, DataV2}, 
  sign_metadata, 
  CreateMasterEditionV3, 
  CreateMetadataAccountsV3, 
  SignMetadata,
  set_and_verify_sized_collection_item,
  SetAndVerifySizedCollectionItem
};
use switchboard_on_demand::RandomnessAccountData;


declare_id!("6MHBjpe9hhEF3kL56qLdxtuku5pwmQAe7qHq7q4FD1YS");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";

#[constant]
pub const SYMBOL: &str = "TLT";

#[constant]
pub const URI: &str = "https://raw.githubusercontent.com/NeelContractor/Solana-Bootcamp-Projects/refs/heads/main/token-lottery/anchor/metadata.json";

#[program]
pub mod tokenlottery {
    use super::*;

  pub fn initialize_config(ctx: Context<InitializeConfig>, start: u64, end: u64, price: u64) -> Result<()> {
    ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
    ctx.accounts.token_lottery.start_time = start;
    ctx.accounts.token_lottery.end_time = end;
    ctx.accounts.token_lottery.ticket_price = price;
    ctx.accounts.token_lottery.authority = *ctx.accounts.payer.key;
    ctx.accounts.token_lottery.lottery_pot_amount = 0;
    ctx.accounts.token_lottery.total_tickets = 0;
    ctx.accounts.token_lottery.randomness_account = Pubkey::default();
    ctx.accounts.token_lottery.winner_chosen = false;

    Ok(())
  }

  pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {

    let signer_seeds: &[&[&[u8]]] = &[&[
      b"collection_mint".as_ref(),
      &[ctx.bumps.collection_mint],
    ]];

    msg!("Creating mint accounts");
    mint_to(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        MintTo {
            mint: ctx.accounts.collection_mint.to_account_info(),
            to: ctx.accounts.collection_token_account.to_account_info(),
            authority: ctx.accounts.collection_mint.to_account_info(),
        }, 
        &signer_seeds
      ),
      1,
    )?;

    msg!("Creating metadata accounts");
    create_metadata_accounts_v3(
      CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(), 
        CreateMetadataAccountsV3 {
          metadata: ctx.accounts.metadata.to_account_info(),
          mint: ctx.accounts.collection_mint.to_account_info(),
          mint_authority: ctx.accounts.collection_mint.to_account_info(),
          payer: ctx.accounts.payer.to_account_info(),
          update_authority: ctx.accounts.collection_mint.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info()
        }, 
        &signer_seeds,
      ), 
      DataV2 {
        name: NAME.to_string(),
        symbol: SYMBOL.to_string(),
        uri: URI.to_string(),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
          address: ctx.accounts.collection_mint.key(),
          verified: false,
          share: 100,
        }]),
        collection: None,
        uses: None,
      }, 
      true, 
      true, 
      Some(CollectionDetails::V1 { size: 0 }),  //set as collection nft
    )?;

    msg!("Creating Master edition accounts");

    create_master_edition_v3(
      CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(), 
        CreateMasterEditionV3 {
          payer: ctx.accounts.payer.to_account_info(),
          mint: ctx.accounts.collection_mint.to_account_info(),
          edition: ctx.accounts.master_edition.to_account_info(),
          update_authority: ctx.accounts.collection_mint.to_account_info(),
          mint_authority: ctx.accounts.collection_mint.to_account_info(),
          metadata: ctx.accounts.metadata.to_account_info(),
          token_program: ctx.accounts.token_program.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info(),
        }, 
        &signer_seeds,
      ), 
      Some(0)
    )?;

    msg!("verifying collection");
    sign_metadata(CpiContext::new_with_signer(
      ctx.accounts.token_metadata_program.to_account_info(), 
      SignMetadata {
        creator: ctx.accounts.collection_mint.to_account_info(),
        metadata: ctx.accounts.metadata.to_account_info(),
      }, 
      &signer_seeds
    ))?;

    Ok(())
  }

  pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
    let clock = Clock::get()?;
    let ticket_name = NAME.to_owned() + ctx.accounts.token_lottery.total_tickets.to_string().as_str();

    if clock.slot < ctx.accounts.token_lottery.start_time || clock.slot > ctx.accounts.token_lottery.end_time {
      return Err(ErrorCode::LotteryNotOpen.into());
    }

    system_program::transfer(
      CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer { 
          from: ctx.accounts.payer.to_account_info(), 
          to: ctx.accounts.token_lottery.to_account_info() 
        },
      ), 
      ctx.accounts.token_lottery.ticket_price
    )?;

    ctx.accounts.token_lottery.lottery_pot_amount += ctx.accounts.token_lottery.ticket_price;

    let signer_seeds: &[&[&[u8]]] = &[&[
      b"collection_mint".as_ref(),
      &[ctx.bumps.collection_mint],
    ]];

    mint_to(
      CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        MintTo {
          mint: ctx.accounts.ticket_mint.to_account_info(),
          to: ctx.accounts.destination.to_account_info(),
          authority: ctx.accounts.collection_mint.to_account_info(),
        }, 
        &signer_seeds,
      ), 
      1
    )?;

    create_metadata_accounts_v3(
      CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(), 
        CreateMetadataAccountsV3 {
          metadata: ctx.accounts.ticket_metadata.to_account_info(),
          mint: ctx.accounts.ticket_mint.to_account_info(),
          mint_authority: ctx.accounts.collection_mint.to_account_info(),
          payer: ctx.accounts.payer.to_account_info(),
          update_authority: ctx.accounts.collection_mint.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info(),
        }, 
        &signer_seeds
      ), 
      DataV2 {
        name: ticket_name,
        symbol: SYMBOL.to_string(),
        uri: URI.to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
      },
      true,
      true,
      None
    )?;

    create_master_edition_v3(
      CpiContext::new_with_signer( 
        ctx.accounts.token_metadata_program.to_account_info(), 
        CreateMasterEditionV3 {
          payer: ctx.accounts.payer.to_account_info(),
          edition: ctx.accounts.ticket_master_edition.to_account_info(),
          mint: ctx.accounts.ticket_mint.to_account_info(),
          update_authority: ctx.accounts.collection_mint.to_account_info(),
          mint_authority: ctx.accounts.collection_mint.to_account_info(),
          metadata: ctx.accounts.ticket_metadata.to_account_info(),
          token_program: ctx.accounts.token_program.to_account_info(),
          system_program: ctx.accounts.system_program.to_account_info(),
          rent: ctx.accounts.rent.to_account_info(),
        }, 
        &signer_seeds
      ), 
      Some(0),
    )?;

    set_and_verify_sized_collection_item(
      CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(), 
        SetAndVerifySizedCollectionItem {
          metadata: ctx.accounts.ticket_metadata.to_account_info(),
          collection_authority: ctx.accounts.collection_mint.to_account_info(),
          payer: ctx.accounts.payer.to_account_info(),
          update_authority: ctx.accounts.collection_mint.to_account_info(),
          collection_mint: ctx.accounts.collection_mint.to_account_info(),
          collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
          collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
        }, 
        &signer_seeds
      ), 
      None,
    )?;

    ctx.accounts.token_lottery.total_tickets += 1;

    Ok(())
  }

  pub fn commit_randomness(ctx: Context<CommitRandomness>) -> Result<()> {
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    if ctx.accounts.payer.key() != token_lottery.authority {
      return Err(ErrorCode::NotAuthorized.into());
    }

    let randomness_data = RandomnessAccountData::parse(
      ctx.accounts.randomness_account.data.borrow()).unwrap();

    if randomness_data.seed_slot != clock.slot {
      return Err(ErrorCode::RandomnessAlreadyRevealed.into());
    }

    token_lottery.randomness_account = ctx.accounts.randomness_account.key();

    Ok(())
  }

  pub fn reveal_winner(ctx: Context<RevealWinner>) -> Result<()> {
    let clock = Clock::get()?;
    let token_lottery = &mut ctx.accounts.token_lottery;

    if ctx.accounts.randomness_account.key() != token_lottery.randomness_account {
      return Err(ErrorCode::IncorrectRandomnessAccount.into());
    }
    if ctx.accounts.payer.key() != token_lottery.authority {
      return Err(ErrorCode::NotAuthorized.into());
    }
    if clock.slot < token_lottery.end_time {
      msg!("Current slot: {}", clock.slot);
      msg!("End slot: {}", token_lottery.end_time);
      return Err(ErrorCode::LotteryNotCompleted.into());
    }
    require!(token_lottery.winner_chosen == false, ErrorCode::WinnerChosen);

    let randomness_data = RandomnessAccountData::parse(ctx.accounts.randomness_account.data.borrow()).unwrap();
    let reveal_random_value = randomness_data.get_value(&clock).map_err(|_| ErrorCode::RandomnessNotReveal)?;

    msg!("Randomness result: {}", reveal_random_value[0]);
    msg!("Ticket num: {}", token_lottery.total_tickets);

    let randomness_result = reveal_random_value[0] as u64 % token_lottery.total_tickets;

    msg!("Winner: {}", randomness_result);

    token_lottery.winner = randomness_result;
    token_lottery.winner_chosen = true;

    Ok(())
  }

  pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
    msg!("Winner chosen: {}", ctx.accounts.token_lottery.winner_chosen);
    require!(ctx.accounts.token_lottery.winner_chosen, ErrorCode::WinnerNotChosen);

    require!(ctx.accounts.metadata.collection.as_ref().unwrap().verified, ErrorCode::NotVerifiedTicket);
    require!(ctx.accounts.metadata.collection.as_ref().unwrap().key == ctx.accounts.collection_mint.key(), ErrorCode::IncorrectTicket);

    let ticket_name = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
    let metadata_name = ctx.accounts.metadata.name.replace("\u{0}", "");

    msg!("Ticket name: {}", ticket_name);
    msg!("Metadata name: {}", metadata_name);

    require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);
    require!(ctx.accounts.destination.amount > 0, ErrorCode::IncorrectTicket);

    **ctx.accounts.token_lottery.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.token_lottery.lottery_pot_amount;
    **ctx.accounts.payer.try_borrow_mut_lamports()? += ctx.accounts.token_lottery.lottery_pot_amount;

    ctx.accounts.token_lottery.lottery_pot_amount = 0;

    Ok(())
  }

}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    init,
    payer = payer,
    space = 8 + Tokenlottery::INIT_SPACE,
    seeds = [b"token_lottery".as_ref()],
    bump
  )]
  pub token_lottery: Account<'info, Tokenlottery>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    init,
    payer = payer,
    mint::decimals = 0,
    mint::authority = collection_mint,
    mint::freeze_authority = collection_mint,
    seeds = [b"collection_mint".as_ref()],
    bump
  )]
  pub collection_mint: InterfaceAccount<'info, Mint>,

  /// CHECK: This account will be initialized by the metaplex program
  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key.as_ref(),
      collection_mint.key().as_ref()
      ],
    bump,
    seeds::program = token_metadata_program.key()
  )]
  pub metadata: UncheckedAccount<'info>,

  /// CHECK: This account will be initialized by the metaplex program
  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key.as_ref(), 
      collection_mint.key().as_ref(),
      b"edition"
      ],
    bump,
    seeds::program = token_metadata_program.key()
  )]
  pub master_edition: UncheckedAccount<'info>,

  #[account(
    init,
    payer = payer,
    token::mint = collection_mint,
    token::authority = collection_token_account,
    seeds = [b"collection_associated_token".as_ref()],
    bump,
  )]
  pub collection_token_account: InterfaceAccount<'info, TokenAccount>,

  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_metadata_program: Program<'info, Metadata>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)] 
pub struct BuyTicket<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    mut,
    seeds = [b"token_lottery".as_ref()],
    bump = token_lottery.bump
  )]
  pub token_lottery: Account<'info, Tokenlottery>,

  #[account(
    init,
    payer = payer,
    seeds = [token_lottery.total_tickets.to_le_bytes().as_ref()],
    bump,
    mint::decimals = 0,
    mint::authority = collection_mint,
    mint::freeze_authority = collection_mint,
    mint::token_program = token_program,
  )]
  pub ticket_mint: InterfaceAccount<'info, Mint>,

  #[account(
    mut, 
    seeds = [b"collection_mint".as_ref()],
    bump
  )]
  pub collection_mint: InterfaceAccount<'info, Mint>,

  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key().as_ref(), 
      collection_mint.key().as_ref()
      ],
    bump,
    seeds::program = token_metadata_program.key(),
  )]
  /// CHECK: This account will be initialized by the metaplex program
  pub collection_metadata: UncheckedAccount<'info>,

  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key().as_ref(), 
      collection_mint.key().as_ref(), 
      b"edition"
      ],
    bump,
    seeds::program = token_metadata_program.key(),
  )]
  /// CHECK: This account will be initialized by the metaplex program
  pub collection_master_edition: UncheckedAccount<'info>,

  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key().as_ref(), 
      ticket_mint.key().as_ref()
      ],
    bump,
    seeds::program = token_metadata_program.key(),
  )]
  /// CHECK: This account will be initialized by the metaplex program
  pub ticket_metadata: UncheckedAccount<'info>,

  #[account(
    mut,
    seeds = [
      b"metadata", 
      token_metadata_program.key().as_ref(), 
      ticket_mint.key().as_ref(), 
      b"edition"
      ],
    bump,
    seeds::program = token_metadata_program.key(),
  )]
  /// CHECK: This account will be initialized by the metaplex program
  pub ticket_master_edition: UncheckedAccount<'info>,

  #[account(
    init,
    payer = payer,
    associated_token::mint = ticket_mint,
    associated_token::authority = payer,
    associated_token::token_program = token_program,
  )]
  pub destination: InterfaceAccount<'info, TokenAccount>,

  pub associated_token_program: Program<'info, AssociatedToken>,
  pub token_metadata_program: Program<'info, Metadata>,
  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CommitRandomness<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    mut,
    seeds = [b"token_lottery".as_ref()],
    bump = token_lottery.bump
  )]
  pub token_lottery: Account<'info, Tokenlottery>,

  /// CHECK: this account will be checked by Switchboard smart contract
  pub randomness_account: UncheckedAccount<'info>,

  pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevealWinner<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    mut,
    seeds = [b"token_lottery".as_ref()],
    bump = token_lottery.bump
  )]
  pub token_lottery: Account<'info, Tokenlottery>,

  /// CHECK: this account will be checked by Switchboard smart contract
  pub randomness_account: UncheckedAccount<'info>,
  pub system_program: Program<'info, System>,
}

#[derive(Accounts)] 
pub struct ClaimPrize<'info> {
  #[account(mut)]
  pub payer: Signer<'info>,

  #[account(
    mut,
    seeds = [b"token_lottery".as_ref()],
    bump = token_lottery.bump
  )]
  pub token_lottery: Account<'info, Tokenlottery>,

  #[account(
    mut,
    seeds = [b"collection_mint".as_ref()],
    bump
  )]
  pub collection_mint: InterfaceAccount<'info, Mint>,

  #[account(
    seeds = [token_lottery.winner.to_le_bytes().as_ref()],
    bump
  )]
  pub ticket_mint: InterfaceAccount<'info, Mint>,

  #[account(
    seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
    bump,
    seeds::program = token_metadata_program.key(),
  )]
  pub metadata: Account<'info, MetadataAccount>,

  #[account(
      associated_token::mint = ticket_mint,
      associated_token::authority = payer,
      associated_token::token_program = token_program,
  )]
  pub destination: InterfaceAccount<'info, TokenAccount>,

  #[account(
      mut,
      seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
      bump,
      seeds::program = token_metadata_program.key(),
  )]
  pub collection_metadata: Account<'info, MetadataAccount>,

  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info, System>,
  pub token_metadata_program: Program<'info, Metadata>,
}

#[account]
#[derive(InitSpace)]
pub struct Tokenlottery {
  pub bump: u8,
  pub winner: u64,
  pub winner_chosen: bool,
  pub start_time: u64,
  pub end_time: u64,
  pub lottery_pot_amount: u64,
  pub total_tickets: u64,
  pub ticket_price: u64,
  pub authority: Pubkey,
  pub randomness_account: Pubkey
}

#[error_code]
pub enum ErrorCode {
  #[msg("Lottery is Not Open")]
  LotteryNotOpen,
  #[msg("Not Authority")]
  NotAuthorized,
  #[msg("Randomness Already Revealed")]
  RandomnessAlreadyRevealed,
  #[msg("Incorrect Randomness Account")]
  IncorrectRandomnessAccount,
  #[msg("Lottery not completed yet")]
  LotteryNotCompleted,
  #[msg("Winner is already chosen")]
  WinnerChosen,
  #[msg("Randomness is not revealed yet")]
  RandomnessNotReveal,
  #[msg("Winner is not chosen")]
  WinnerNotChosen,
  #[msg("Ticket is not Verified Ticket")]
  NotVerifiedTicket,
  #[msg("Ticket is Incorrect")]
  IncorrectTicket,
}