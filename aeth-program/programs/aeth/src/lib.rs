use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("Aeth111111111111111111111111111111111111111");

#[program]
pub mod aeth_protocol {
    use super::*;

    pub fn initialize_economy(ctx: Context<InitializeEconomy>, fee_bps: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.authority = ctx.accounts.authority.key();
        pool.fee_bps = fee_bps;
        pool.total_staked = 0;
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        
        // Transfer AETH from user to pool vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token.to_account_info(),
            to: ctx.accounts.pool_vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        stake_account.owner = ctx.accounts.user.key();
        stake_account.amount += amount;
        stake_account.last_stake_ts = Clock::get()?.unix_timestamp;
        
        // Reset or boost gratitude on new stake
        if stake_account.gratitude_score == 0 {
            stake_account.gratitude_score = 100; // Starting gratitude (Base 1.0x)
        }
        
        ctx.accounts.pool.total_staked += amount;
        
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let pool = &ctx.accounts.pool;

        let elapsed_time = Clock::get()?.unix_timestamp - stake_account.last_stake_ts;
        
        // Calculate Base Reward (simplified: 1 token per hour per 1000 staked)
        let base_reward = (stake_account.amount / 1000) * (elapsed_time as u64 / 3600);
        
        // Apply Gratitude Multiplier (e.g. 500 score = 1.5x)
        let gratitude_multiplier = 1000 + stake_account.gratitude_score;
        let total_reward = (base_reward * gratitude_multiplier) / 1000;

        msg!("Claiming reward: {} (Multiplier: {}x)", total_reward, gratitude_multiplier as f64 / 1000.0);

        // Transfer rewards from pool vault to user (CPI omitted for brevity)
        stake_account.last_stake_ts = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    pub fn commit_batch(ctx: Context<CommitBatch>, batch_id: u64, proof_hash: [u8; 32]) -> Result<()> {
        let batch = &mut ctx.accounts.batch_record;
        batch.sequencer = ctx.accounts.sequencer.key();
        batch.batch_id = batch_id;
        batch.proof_hash = proof_hash;
        batch.ts = Clock::get()?.unix_timestamp;

        // Increase Gratitude for successful contribution
        let stake_account = &mut ctx.accounts.sequencer_stake;
        stake_account.gratitude_score += 10; // +1% boost per batch
        
        Ok(())
    }

    pub fn slash_node(ctx: Context<SlashNode>, penalty_amount: u64) -> Result<()> {
        // Governance only
        require!(
            ctx.accounts.authority.key() == ctx.accounts.pool.authority,
            AethError::Unauthorized
        );

        let stake_account = &mut ctx.accounts.target_stake;
        let amount_to_slash = std::cmp::min(stake_account.amount, penalty_amount);
        
        stake_account.amount -= amount_to_slash;
        ctx.accounts.pool.total_staked -= amount_to_slash;
        
        // Burnt or sent to DAO treasury
        Ok(())
    }
}

#[account]
pub struct StakingPool {
    pub authority: Pubkey,
    pub fee_bps: u64,
    pub total_staked: u64,
    pub bump: u8,
}

#[account]
pub struct StakeAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub last_stake_ts: i64,
    pub gratitude_score: u64, // The Supreme System Multiplier
}

#[account]
pub struct BatchRecord {
    pub sequencer: Pubkey,
    pub batch_id: u64,
    pub proof_hash: [u8; 32],
    pub ts: i64,
}

#[derive(Accounts)]
#[instruction(fee_bps: u64)]
pub struct InitializeEconomy<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + 32 + 8 + 8 + 1,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 32 + 8 + 8 + 8, // Added gratitude_score
        seeds = [b"stake", user.key().as_ref()],
        bump
    )]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"stake", user.key().as_ref()], bump)]
    pub stake_account: Account<'info, StakeAccount>,
    #[account(seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, StakingPool>,
    #[account(mut)]
    pub user_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(batch_id: u64)]
pub struct CommitBatch<'info> {
    #[account(mut)]
    pub sequencer: Signer<'info>,
    #[account(mut, seeds = [b"stake", sequencer.key().as_ref()], bump)]
    pub sequencer_stake: Account<'info, StakeAccount>,
    #[account(
        init,
        payer = sequencer,
        space = 8 + 32 + 8 + 32 + 8,
        seeds = [b"batch", batch_id.to_le_bytes().as_ref()],
        bump
    )]
    pub batch_record: Account<'info, BatchRecord>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SlashNode<'info> {
    pub authority: Signer<'info>,
    #[account(mut)]
    pub target_stake: Account<'info, StakeAccount>,
    #[account(mut, seeds = [b"pool"], bump = pool.bump)]
    pub pool: Account<'info, StakingPool>,
}

#[error_code]
pub enum AethError {
    #[msg("Unauthorized access")]
    Unauthorized,
}
