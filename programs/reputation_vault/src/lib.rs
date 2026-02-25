use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("2EK5uc9E1RkmKFMZMjhxukDcLFa3Bc4LM57RL38ou2LA");

#[program]
pub mod reputation_vault {
    use super::*;

    // Initialize Vault PDA
    pub fn initialize_vault(
        ctx: Context<InitializeVault>,
        required_score: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        vault.owner = ctx.accounts.owner.key();
        vault.required_score = required_score;
        vault.bump = *ctx.bumps.get("vault").unwrap();

        Ok(())
    }

    // Initialize Reputation PDA
    pub fn initialize_reputation(ctx: Context<InitializeReputation>) -> Result<()> {
        let reputation = &mut ctx.accounts.reputation;

        reputation.user = ctx.accounts.user.key();
        reputation.score = 0;
        reputation.bump = *ctx.bumps.get("reputation").unwrap();

        Ok(())
    }

    // Deposit SOL into Vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let transfer_instruction = Transfer {
            from: ctx.accounts.user.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
        );

        transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // Increase Reputation (Owner Only)
    pub fn increase_reputation(
        ctx: Context<IncreaseReputation>,
        points: u64,
    ) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let reputation = &mut ctx.accounts.reputation;

        // Only vault owner can increase reputation
        require!(
            vault.owner == ctx.accounts.owner.key(),
            VaultError::Unauthorized
        );

        reputation.score += points;

        Ok(())
    }

    // Withdraw SOL (Requires Reputation Threshold)
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &ctx.accounts.vault;
        let reputation = &ctx.accounts.reputation;

        // UNIQUE CONSTRAINT
        require!(
            reputation.score >= vault.required_score,
            VaultError::InsufficientReputation
        );

        let seeds = &[
            b"vault",
            vault.owner.as_ref(),
            &[vault.bump],
        ];

        let signer = &[&seeds[..]];

        let transfer_instruction = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            transfer_instruction,
            signer,
        );

        transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

// ACCOUNTS

#[derive(Accounts)]
pub struct InitializeVault<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 8 + 1,
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeReputation<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8 + 1,
        seeds = [b"reputation", user.key().as_ref()],
        bump
    )]
    pub reputation: Account<'info, Reputation>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncreaseReputation<'info> {
    #[account(
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"reputation", reputation.user.as_ref()],
        bump = reputation.bump
    )]
    pub reputation: Account<'info, Reputation>,

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"vault", vault.owner.as_ref()],
        bump = vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [b"reputation", reputation.user.as_ref()],
        bump = reputation.bump
    )]
    pub reputation: Account<'info, Reputation>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}


// STATE


#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub required_score: u64,
    pub bump: u8,
}

#[account]
pub struct Reputation {
    pub user: Pubkey,
    pub score: u64,
    pub bump: u8,
}


// ERRORS


#[error_code]
pub enum VaultError {
    #[msg("Insufficient reputation score")]
    InsufficientReputation,

    #[msg("Unauthorized action")]
    Unauthorized,
}