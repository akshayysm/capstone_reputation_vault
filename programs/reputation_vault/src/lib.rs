use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, transfer, Transfer};

declare_id!("2EK5uc9E1RkmKFMZMjhxukDcLFa3Bc4LM57RL38ou2LA");

#[program]
pub mod reputation_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, required_score: u64) -> Result<()> {
        ctx.accounts.initialize(required_score, &ctx.bumps)
    }

    pub fn initialize_reputation(ctx: Context<InitializeReputation>) -> Result<()> {
        ctx.accounts.initialize_reputation(&ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn increase_reputation(ctx: Context<IncreaseReputation>, points: u64) -> Result<()> {
        ctx.accounts.increase_reputation(points)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [b"state", owner.key().as_ref()],
        bump,
        space = 8 + ReputationVaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'info, ReputationVaultState>,

    /// CHECK: This is a PDA used only for SOL storage.
    /// It is created via system_program::create_account
    /// and signed using PDA seeds.
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    fn initialize(&mut self, required_score: u64, bumps: &InitializeBumps) -> Result<()> {
        let rent = Rent::get()?;
        let rent_exempt = rent.minimum_balance(0);

        let state_key = self.vault_state.key();
        let seeds = &[b"vault", state_key.as_ref(), &[bumps.vault]];
        let signer = &[&seeds[..]];

        let create_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            system_program::CreateAccount {
                from: self.owner.to_account_info(),
                to: self.vault.to_account_info(),
            },
            signer,
        );

        system_program::create_account(
            create_ctx,
            rent_exempt,
            0,
            &system_program::ID,
        )?;

        self.vault_state.owner = self.owner.key();
        self.vault_state.required_score = required_score;
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeReputation<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"reputation", user.key().as_ref()],
        bump,
        space = 8 + Reputation::INIT_SPACE,
    )]
    pub reputation: Account<'info, Reputation>,

    pub system_program: Program<'info, System>,
}

impl<'info> InitializeReputation<'info> {
    fn initialize_reputation(
        &mut self,
        bumps: &InitializeReputationBumps,
    ) -> Result<()> {
        self.reputation.user = self.user.key();
        self.reputation.score = 0;
        self.reputation.bump = bumps.reputation;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", vault_state.owner.as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, ReputationVaultState>,

    /// CHECK: SOL vault PDA
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.system_program.to_account_info(),
            Transfer {
                from: self.user.to_account_info(),
                to: self.vault.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
pub struct IncreaseReputation<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"state", vault_state.owner.as_ref()],
        bump = vault_state.state_bump,
        constraint = vault_state.owner == owner.key()
    )]
    pub vault_state: Account<'info, ReputationVaultState>,

    #[account(
        mut,
        seeds = [b"reputation", reputation.user.as_ref()],
        bump = reputation.bump,
    )]
    pub reputation: Account<'info, Reputation>,
}

impl<'info> IncreaseReputation<'info> {
    fn increase_reputation(&mut self, points: u64) -> Result<()> {
        self.reputation.score += points;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"state", vault_state.owner.as_ref()],
        bump = vault_state.state_bump,
    )]
    pub vault_state: Account<'info, ReputationVaultState>,

    /// CHECK: SOL vault PDA signed by program
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        seeds = [b"reputation", user.key().as_ref()],
        bump = reputation.bump,
    )]
    pub reputation: Account<'info, Reputation>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    fn withdraw(&mut self, amount: u64) -> Result<()> {
        require!(
            self.reputation.score >= self.vault_state.required_score,
            VaultError::InsufficientReputation
        );

        let state_key = self.vault_state.key();
        let seeds = &[b"vault", state_key.as_ref(), &[self.vault_state.vault_bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.user.to_account_info(),
            },
            signer,
        );

        transfer(cpi_ctx, amount)
    }
}

#[account]
#[derive(InitSpace)]
pub struct ReputationVaultState {
    pub owner: Pubkey,
    pub required_score: u64,
    pub state_bump: u8,
    pub vault_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Reputation {
    pub user: Pubkey,
    pub score: u64,
    pub bump: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("Insufficient reputation score")]
    InsufficientReputation,
}