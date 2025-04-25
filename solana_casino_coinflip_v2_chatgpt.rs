
use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::program::invoke_signed;
use sha2::{Sha256, Digest};
use rand::{SeedableRng, Rng};
use rand_chacha::ChaChaRng;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_casino {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, house_edge_percentage: u8) -> Result<()> {
        require!(house_edge_percentage <= 10, ErrorCode::HouseEdgeTooHigh);
        let casino = &mut ctx.accounts.casino;
        casino.authority = ctx.accounts.authority.key();
        casino.house_edge_percentage = house_edge_percentage;
        casino.total_games_played = 0;
        Ok(())
    }

    pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.authority = ctx.accounts.authority.key();
        game.server_seed = generate_random_seed(&ctx.accounts.authority.key().to_bytes());
        game.server_seed_hash = hash_seed(&game.server_seed);
        game.is_completed = false;
        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, bet_amount: u64, public_seed: String, choice: bool) -> Result<()> {
        require!(bet_amount >= 100000, ErrorCode::BetTooSmall);
        let game = &mut ctx.accounts.game;
        require!(!game.is_completed, ErrorCode::GameAlreadyCompleted);

        let transfer = system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.game_account.key(),
            bet_amount,
        );
        invoke_signed(
            &transfer,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.game_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;

        game.player = ctx.accounts.player.key();
        game.bet_amount = bet_amount;
        game.public_seed = public_seed;
        game.player_choice = choice;
        game.future_slot = Clock::get()?.slot + 3;

        Ok(())
    }

    pub fn complete_game(ctx: Context<CompleteGame>, current_slot: u64, block_hash: [u8; 32]) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(!game.is_completed, ErrorCode::GameAlreadyCompleted);
        require!(current_slot >= game.future_slot, ErrorCode::SlotNotReached);

        let combined_hash = generate_combined_hash(&game.server_seed, &game.public_seed, &block_hash);
        let random_value = get_seeded_random_number(&combined_hash, 0, 999_999);
        let outcome = random_value < 500_000;
        let player_wins = outcome == game.player_choice;

        game.outcome = outcome;
        game.is_completed = true;
        game.server_seed_revealed = game.server_seed.clone();
        game.block_hash = block_hash;

        let casino = &mut ctx.accounts.casino;
        casino.total_games_played += 1;

        let house_edge = game.bet_amount * casino.house_edge_percentage as u64 / 100;

        if player_wins {
            let payout = game.bet_amount * 2 - house_edge;
            let transfer = system_instruction::transfer(
                &ctx.accounts.game_account.key(),
                &ctx.accounts.player.key(),
                payout,
            );
            invoke_signed(
                &transfer,
                &[
                    ctx.accounts.game_account.to_account_info(),
                    ctx.accounts.player.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[b"game", game.key().as_ref(), &[ctx.bumps.game_account]]],
            )?;
        }

        Ok(())
    }
}

#[account]
pub struct Casino {
    pub authority: Pubkey,
    pub house_edge_percentage: u8,
    pub total_games_played: u64,
}

#[account]
pub struct Game {
    pub authority: Pubkey,
    pub player: Pubkey,
    pub bet_amount: u64,
    pub server_seed: String,
    pub server_seed_hash: String,
    pub server_seed_revealed: String,
    pub public_seed: String,
    pub future_slot: u64,
    pub block_hash: [u8; 32],
    pub player_choice: bool,
    pub outcome: bool,
    pub is_completed: bool,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 1 + 8)]
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 32 + 8 + 200 + 200 + 200 + 200 + 8 + 32 + 1 + 1 + 1)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub player: Signer<'info>,
    #[account(init_if_needed, payer = player, space = 8, seeds = [b"game", game.key().as_ref()], bump)]
    pub game_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompleteGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub player: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"game", game.key().as_ref()], bump)]
    pub game_account: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bet amount is too small")]
    BetTooSmall,
    #[msg("House edge too high")]
    HouseEdgeTooHigh,
    #[msg("Game already completed")]
    GameAlreadyCompleted,
    #[msg("Slot not reached")]
    SlotNotReached,
}

fn generate_random_seed(entropy: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    hasher.update(Clock::get().unwrap().unix_timestamp.to_be_bytes());
    hex::encode(hasher.finalize())
}

fn hash_seed(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hex::encode(hasher.finalize())
}

fn generate_combined_hash(server_seed: &str, public_seed: &str, block_hash: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(server_seed.as_bytes());
    hasher.update(public_seed.as_bytes());
    hasher.update(block_hash);
    hex::encode(hasher.finalize())
}

fn get_seeded_random_number(hash: &str, min: u64, max: u64) -> u64 {
    let seed_bytes = hex::decode(&hash[0..32]).unwrap();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_bytes);
    let mut rng = ChaChaRng::from_seed(seed);
    min + (rng.gen::<u64>() % (max - min + 1))
}
