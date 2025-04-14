#![allow(clippy::result_large_err)]
#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,  
  token_interface::{Mint, TokenAccount, TokenInterface, mint_to, MintTo},
};
use anchor_spl::metadata::{
  Metadata,
  create_master_edition_v3, 
  create_metadata_accounts_v3, 
  mpl_token_metadata::types::{CollectionDetails, Creator, DataV2}, 
  sign_metadata, 
  CreateMasterEditionV3, 
  CreateMetadataAccountsV3, 
  SignMetadata
};


declare_id!("6MHBjpe9hhEF3kL56qLdxtuku5pwmQAe7qHq7q4FD1YS");

#[constant]
pub const NAME: &str = "Token Lottery Ticket #";

#[constant]
pub const SYMBOL: &str = "TICKET";

#[constant]
pub const URI: &str = "TICKET";

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