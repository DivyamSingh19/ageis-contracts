use anchor_lang::prelude::*;

declare_id!("4fRvr5yrDNTqnSXv8yFb9CSj3MwnYuade8UUmgb8cg3H");

#[program]
pub mod ageis {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
