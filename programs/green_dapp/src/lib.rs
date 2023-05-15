use anchor_lang::prelude::*;

use anchor_spl::token::{
     self, Mint, Token, Transfer, TokenAccount,
};

declare_id!("43wWGRGGehT2SuNzfC1CBwgjLEZC1uBY8fB6KXAB5HGE");

#[program]
pub mod green_dapp {
    use super::*;

    const _AUTHORITY_SEED: &[u8] = b"vault";

    pub fn initialize(
        ctx: Context<Initialize>,
        city_name: String,
    ) -> Result<()> {
        let city_state: &mut Account<CityState> = &mut ctx.accounts.city_state;

        let signer: &Signer = &ctx.accounts.signer;
        
        if city_name.chars().count() > 50 {
            return Err(ErrorCodes::CityNameTooLong.into())
        }

        city_state.initializer_key = *signer.key;
        city_state.city_name = city_name;
        city_state.green_mint_key = ctx.accounts.mint_green.key();
        city_state.red_mint_key = ctx.accounts.mint_red.key();

        Ok(())
    }

    pub fn withdraw_from_token_account(ctx: Context<WithdrawFromTokenAccount>, amount:u64) -> Result<()> {

        // obter as pdas
        // verificar o 80%
        // caso passe transferir

        let (_vault_authority_green, vault_authority_bump_green) =
            Pubkey::find_program_address(&[b"vault".as_ref(), ctx.accounts.signer.key().as_ref(), ctx.accounts.mint_green.key().as_ref()], ctx.program_id);
       
        let (_vault_authority_red, _vault_authority_bump_red) =
            Pubkey::find_program_address(&[b"vault".as_ref(), ctx.accounts.signer.key().as_ref(), ctx.accounts.mint_red.key().as_ref()], ctx.program_id);


        let green_tokens_amount = ctx.accounts.green_token_account_vault.amount;        
        msg!("account green!! {:?}", &green_tokens_amount);

        if  green_tokens_amount < amount{
            return Err(ErrorCodes::NotEnoughGreenTokens.into())
        }else if  (green_tokens_amount - amount) / 5  < ctx.accounts.red_token_account_vault.amount{
            return Err(ErrorCodes::NotEnoughGreenTokenPercentage.into())
        }

        token::transfer(
            CpiContext::<Transfer>::new_with_signer(
                ctx.accounts.green_token_account_vault.to_account_info(),
                Transfer {
                    from: ctx.accounts.green_token_account_vault.to_account_info(),
                    to: ctx.accounts.taker_deposit_token_account.to_account_info(),
                    authority: ctx.accounts.green_token_account_vault.to_account_info()
                },
                &[&[b"vault".as_ref(), ctx.accounts.signer.key().as_ref(), ctx.accounts.mint_green.key().as_ref(), &[vault_authority_bump_green]]]), amount)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        constraint = signer_green_token_account.mint == mint_green.key() @ ErrorCodes::MintAccountMismatch,
        constraint = signer_green_token_account.owner == signer.key() @ ErrorCodes::OwnerAccountMismatch
    )]
    pub signer_green_token_account: Box<Account<'info, TokenAccount>>,
    #[account(init, payer=signer, seeds=[b"vault".as_ref(), signer.key().as_ref(), mint_green.key().as_ref()], bump, token::mint = mint_green, token::authority = green_token_account_vault)]
    pub green_token_account_vault: Account<'info, TokenAccount>,
    pub mint_green: Account<'info, Mint>,
    #[account(
        mut,
        constraint = signer_red_token_account.mint == mint_red.key() @ ErrorCodes::MintAccountMismatch,
        constraint = signer_red_token_account.owner == signer.key() @ ErrorCodes::OwnerAccountMismatch
    )]
    pub signer_red_token_account: Account<'info, TokenAccount>,
    #[account(init, payer=signer, seeds=[b"vault".as_ref(), signer.key().as_ref(), mint_red.key().as_ref()], bump, token::mint = mint_red, token::authority = red_token_account_vault)]
    pub red_token_account_vault: Account<'info, TokenAccount>,
    pub mint_red: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
    #[account(init, seeds = [b"cityStateSeed".as_ref(), signer.key().as_ref()], bump, payer = signer, space = CityState::LEN)]
    pub city_state: Account<'info, CityState>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFromTokenAccount<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub signer: Signer<'info>,
    #[account(mut)]
    pub taker_deposit_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut,
        seeds=[b"vault".as_ref(), signer.key().as_ref(), mint_green.key().as_ref()], bump
    )]
    pub green_token_account_vault: Account<'info, TokenAccount>,
    #[account(
        seeds=[b"vault".as_ref(), signer.key().as_ref(), mint_red.key().as_ref()], bump
    )]
    pub red_token_account_vault: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
    )]
    pub city_state: Box<Account<'info, CityState>>,
    #[account(mut)]
    pub mint_green: Account<'info, Mint>,
    pub mint_red: Account<'info, Mint>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
}


#[account]
pub struct CityState {
    pub initializer_key: Pubkey,
    pub green_mint_key: Pubkey,
    pub red_mint_key: Pubkey,
    pub city_name: String,
}


// 2. Add some useful constants for sizing propeties.
const DISCRIMINATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const STRING_LENGTH_PREFIX: usize = 4; // Stores the size of the string.
const MAX_NAME_LENGTH: usize = 50 * 4; // 50 chars max.

// 3. Add a constant on the Tweet account that provides its total size.
impl CityState {
    const LEN: usize = DISCRIMINATOR_LENGTH 
        + PUBLIC_KEY_LENGTH // Author.
        + PUBLIC_KEY_LENGTH // Mint public address
        + PUBLIC_KEY_LENGTH // Mint public address
        + STRING_LENGTH_PREFIX + MAX_NAME_LENGTH; // Topic.

}

#[error_code]
pub enum ErrorCodes {
    #[msg("Mint account mismatch")]
    MintAccountMismatch,
    #[msg("owner account mismatch")]
    OwnerAccountMismatch,
    #[msg("The provided city name should be 50 characters long maximum.")]
    CityNameTooLong,
    #[msg("The city doesn't have the amount of green tokens it tried to withdraw")]
    NotEnoughGreenTokens,
    #[msg("The city does not have the necessary amount of green tokens in relation to red tokens")]
    NotEnoughGreenTokenPercentage,
}

/*


cidade inicializa.

inicializar 


*/