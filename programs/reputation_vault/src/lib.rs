use anchor_lang::prelude::*;

declare_id!("2EK5uc9E1RkmKFMZMjhxukDcLFa3Bc4LM57RL38ou2LA");

#[program]
pub mod reputation_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
