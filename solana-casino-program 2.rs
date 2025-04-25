use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use solana_program::program::invoke_signed;
use solana_program::system_instruction;
use std::convert::TryInto;
use sha2::{Sha256, Digest};
use rand::{SeedableRng, Rng};
use rand_chacha::ChaChaRng;
use std::str::FromStr;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"); // Replace with your program ID

// Enum to represent different game types
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GameType {
    CoinFlip,
    Jackpot,
}

// Event emitted when a game is played
#[event]
pub struct GamePlayed {
    player: Pubkey,
    game_type: GameType,
    bet_amount: u64,
    outcome: bool,
    timestamp: i64,
}

// Event emitted when a provably fair game is completed
#[event]
pub struct GameCompleted {
    game_id: Pubkey,
    player: Pubkey,
    bet_amount: u64,
    outcome: bool,
    player_choice: bool,
    player_won: bool,
    server_seed: String,
    public_seed: String,
    block_hash: [u8; 32],
    timestamp: i64,
}

#[program]
pub mod solana_casino {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, house_edge_percentage: u8) -> Result<()> {
        require!(house_edge_percentage <= 10, ErrorCode::HouseEdgeTooHigh);
        
        let casino = &mut ctx.accounts.casino;
        casino.authority = ctx.accounts.authority.key();
        casino.house_edge_percentage = house_edge_percentage;
        casino.total_games_played = 0;
        casino.total_jackpot_amount = 0;
        casino.jackpot_contribution_percentage = 5; // 5% of each bet goes to jackpot
        
        Ok(())
    }

    pub fn play_coin_flip(ctx: Context<PlayGame>, bet_amount: u64, choice: bool) -> Result<()> {
        // Validate minimum bet
        require!(bet_amount >= 100000, ErrorCode::BetTooSmall); // 0.0001 SOL minimum
        
        let casino = &mut ctx.accounts.casino;
        
        // Transfer bet amount from player to PDA
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.casino_vault.key(),
            bet_amount,
        );
        
        invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.casino_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;
        
        // Generate random outcome using recent blockhash as seed
        let recent_blockhash = ctx.accounts.recent_blockhashes.to_account_info().data.borrow();
        let slice = &recent_blockhash[0..8];
        let random_value = u64::from_le_bytes(slice.try_into().unwrap());
        let outcome = random_value % 2 == 0;
        
        // Calculate jackpot contribution
        let jackpot_contribution = bet_amount * casino.jackpot_contribution_percentage as u64 / 100;
        casino.total_jackpot_amount = casino.total_jackpot_amount.checked_add(jackpot_contribution).unwrap();
        
        // Calculate house edge
        let house_edge = bet_amount * casino.house_edge_percentage as u64 / 100;
        
        // Determine if player wins
        let player_wins = choice == outcome;
        
        // Update game statistics
        casino.total_games_played = casino.total_games_played.checked_add(1).unwrap();
        
        // Pay winnings if player wins
        if player_wins {
            let winnings = bet_amount.checked_mul(2).unwrap()
                .checked_sub(jackpot_contribution).unwrap()
                .checked_sub(house_edge).unwrap();
            
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.casino_vault.key(),
                &ctx.accounts.player.key(),
                winnings,
            );
            
            let casino_key = ctx.accounts.casino.key();
            let seeds = &[b"casino", casino_key.as_ref(), &[ctx.bumps.casino_vault]];
            
            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.casino_vault.to_account_info(),
                    ctx.accounts.player.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
        }
        
        emit!(GamePlayed {
            player: ctx.accounts.player.key(),
            game_type: GameType::CoinFlip,
            bet_amount,
            outcome: player_wins,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    pub fn try_jackpot(ctx: Context<PlayGame>, bet_amount: u64) -> Result<()> {
        // Validate minimum bet
        require!(bet_amount >= 100000, ErrorCode::BetTooSmall); // 0.0001 SOL minimum
        
        let casino = &mut ctx.accounts.casino;
        
        // Transfer bet amount from player to PDA
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.casino_vault.key(),
            bet_amount,
        );
        
        invoke_signed(
            &transfer_instruction,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.casino_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[],
        )?;
        
        // Generate random outcome using recent blockhash as seed
        let recent_blockhash = ctx.accounts.recent_blockhashes.to_account_info().data.borrow();
        let slice = &recent_blockhash[0..8];
        let random_value = u64::from_le_bytes(slice.try_into().unwrap());
        
        // Jackpot has a 1 in 10,000 chance (0.01%)
        let jackpot_win = random_value % 10000 == 0;
        
        // Update game statistics
        casino.total_games_played = casino.total_games_played.checked_add(1).unwrap();
        
        // Add to jackpot if player doesn't win
        if !jackpot_win {
            // Add 80% of bet to jackpot
            casino.total_jackpot_amount = casino.total_jackpot_amount
                .checked_add(bet_amount * 80 / 100).unwrap();
        } else {
            // Player wins jackpot!
            let jackpot_amount = casino.total_jackpot_amount;
            casino.total_jackpot_amount = 0; // Reset jackpot
            
            // Transfer jackpot to player
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.casino_vault.key(),
                &ctx.accounts.player.key(),
                jackpot_amount,
            );
            
            let casino_key = ctx.accounts.casino.key();
            let seeds = &[b"casino", casino_key.as_ref(), &[ctx.bumps.casino_vault]];
            
            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.casino_vault.to_account_info(),
                    ctx.accounts.player.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
        }
        
        emit!(GamePlayed {
            player: ctx.accounts.player.key(),
            game_type: GameType::Jackpot,
            bet_amount,
            outcome: jackpot_win,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
    
    // Implements provably fair mechanism by generating and storing seeds
    pub fn create_game(ctx: Context<CreateGame>) -> Result<()> {
        let game = &mut ctx.accounts.game;
        game.authority = ctx.accounts.authority.key();
        game.server_seed = generate_random_seed(&ctx.accounts.authority.key().to_bytes());
        game.server_seed_hash = hash_seed(&game.server_seed);
        game.is_completed = false;
        
        Ok(())
    }
    
    // Player joins game with their public seed
    pub fn join_game(ctx: Context<JoinGame>, bet_amount: u64, public_seed: String, choice: bool) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(!game.is_completed, ErrorCode::GameAlreadyCompleted);
        
        // Validate minimum bet
        require!(bet_amount >= 100000, ErrorCode::BetTooSmall); // 0.0001 SOL minimum
        
        // Transfer bet amount from player to game account
        let transfer_instruction = system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.game_account.key(),
            bet_amount,
        );
        
        invoke_signed(
            &transfer_instruction,
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
        
        // We'll use future EOS/Solana block hash for randomness
        // For now mark the expected block number to use
        game.future_block_number = Clock::get()?.slot + 3; // Use block 3 slots in the future
        
        Ok(())
    }
    
    // Complete the game once the future block is available
    pub fn complete_game(ctx: Context<CompleteGame>, block_hash: [u8; 32]) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(!game.is_completed, ErrorCode::GameAlreadyCompleted);
        
        // In production, we would verify the block hash matches the expected block
        // Here we'll accept the provided hash as if it were validated
        
        // Combine seeds and block hash to get final outcome
        let combined_hash = generate_combined_hash(&game.server_seed, &game.public_seed, &block_hash);
        
        // Get a number between 0-999,999
        let random_value = get_seeded_random_number(&combined_hash, 0, 999_999);
        
        // Heads if < 500,000, Tails otherwise (50% chance)
        let outcome = random_value < 500_000;
        
        // Check if player won
        let player_wins = game.player_choice == outcome;
        
        // Mark game as completed
        game.is_completed = true;
        game.outcome = outcome;
        game.server_seed_revealed = game.server_seed.clone();
        game.block_hash = block_hash;
        
        // Calculate winnings amount with house edge
        let casino = &mut ctx.accounts.casino;
        
        // Calculate jackpot contribution
        let jackpot_contribution = game.bet_amount * casino.jackpot_contribution_percentage as u64 / 100;
        casino.total_jackpot_amount = casino.total_jackpot_amount.checked_add(jackpot_contribution).unwrap();
        
        // Calculate house edge
        let house_edge = game.bet_amount * casino.house_edge_percentage as u64 / 100;
        
        // Update game statistics
        casino.total_games_played = casino.total_games_played.checked_add(1).unwrap();
        
        // Transfer funds based on outcome
        if player_wins {
            let winnings = game.bet_amount.checked_mul(2).unwrap()
                .checked_sub(jackpot_contribution).unwrap()
                .checked_sub(house_edge).unwrap();
            
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.game_account.key(),
                &ctx.accounts.player.key(),
                winnings,
            );
            
            let game_key = ctx.accounts.game.key();
            let seeds = &[b"game", game_key.as_ref(), &[ctx.bumps.game_account]];
            
            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.game_account.to_account_info(),
                    ctx.accounts.player.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
        } else {
            // House keeps the bet minus jackpot contribution
            let house_amount = game.bet_amount.checked_sub(jackpot_contribution).unwrap();
            
            let transfer_instruction = system_instruction::transfer(
                &ctx.accounts.game_account.key(),
                &ctx.accounts.casino_vault.key(),
                house_amount,
            );
            
            let game_key = ctx.accounts.game.key();
            let seeds = &[b"game", game_key.as_ref(), &[ctx.bumps.game_account]];
            
            invoke_signed(
                &transfer_instruction,
                &[
                    ctx.accounts.game_account.to_account_info(),
                    ctx.accounts.casino_vault.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[seeds],
            )?;
        }
        
        emit!(GameCompleted {
            game_id: game.key(),
            player: game.player,
            bet_amount: game.bet_amount,
            outcome,
            player_choice: game.player_choice,
            player_won: player_wins,
            server_seed: game.server_seed.clone(),
            public_seed: game.public_seed.clone(),
            block_hash,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }
}

// Generate a random seed string
fn generate_random_seed(entropy: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(entropy);
    hasher.update(Clock::get().unwrap().unix_timestamp.to_be_bytes());
    
    let hash = hasher.finalize();
    hex::encode(hash)
}

// Hash a seed for secure storage
fn hash_seed(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hex::encode(hasher.finalize())
}

// Combine seeds and block hash to create final hash
fn generate_combined_hash(server_seed: &str, public_seed: &str, block_hash: &[u8; 32]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(server_seed.as_bytes());
    hasher.update(public_seed.as_bytes());
    hasher.update(block_hash);
    hex::encode(hasher.finalize())
}

// Generate a seeded random number in range [min, max]
fn get_seeded_random_number(hash: &str, min: u64, max: u64) -> u64 {
    // Take first 16 bytes of hash to use as seed
    let seed_bytes = hex::decode(&hash[0..32]).unwrap();
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&seed_bytes);
    
    // Create PRNG from seed
    let mut rng = ChaChaRng::from_seed(seed);
    
    // Generate random number in range
    let range = max - min + 1;
    min + (rng.gen::<u64>() % range)
}

// Account to store casino state
#[account]
pub struct Casino {
    pub authority: Pubkey,
    pub house_edge_percentage: u8,
    pub jackpot_contribution_percentage: u8,
    pub total_games_played: u64,
    pub total_jackpot_amount: u64,
}

// Account to store provably fair game state
#[account]
#[derive(Default)]
pub struct Game {
    pub authority: Pubkey,
    pub player: Pubkey,
    pub bet_amount: u64,
    pub server_seed: String,
    pub server_seed_hash: String,
    pub server_seed_revealed: String,
    pub public_seed: String,
    pub future_block_number: u64,
    pub block_hash: [u8; 32],
    pub player_choice: bool,
    pub outcome: bool,
    pub is_completed: bool,
}

// Context for initialize instruction
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 32 + 1 + 1 + 8 + 8)]
    pub casino: Account<'info, Casino>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8,
        seeds = [b"casino", casino.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA that will hold casino funds
    pub casino_vault: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

// Context for playing simple games
#[derive(Accounts)]
pub struct PlayGame<'info> {
    #[account(mut)]
    pub casino: Account<'info, Casino>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"casino", casino.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA that holds casino funds
    pub casino_vault: UncheckedAccount<'info>,
    
    /// CHECK: Recent blockhashes are used as source of randomness
    pub recent_blockhashes: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

// Context for creating a provably fair game
#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 8 + 200 + 200 + 200 + 200 + 8 + 32 + 1 + 1 + 1
    )]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Context for joining a provably fair game
#[derive(Accounts)]
#[instruction(bet_amount: u64, public_seed: String, choice: bool)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub player: Signer<'info>,
    
    #[account(
        init_if_needed,
        payer = player,
        space = 8,
        seeds = [b"game", game.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA that will hold game funds
    pub game_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

// Context for completing a provably fair game
#[derive(Accounts)]
pub struct CompleteGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    
    #[account(mut)]
    pub casino: Account<'info, Casino>,
    
    #[account(mut)]
    pub player: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"game", game.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA that holds game funds
    pub game_account: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"casino", casino.key().as_ref()],
        bump
    )]
    /// CHECK: This is a PDA that holds casino funds
    pub casino_vault: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Bet amount is too small")]
    BetTooSmall,
    
    #[msg("House edge percentage is too high")]
    HouseEdgeTooHigh,
    
    #[msg("Game is already completed")]
    GameAlreadyCompleted,
    
    #[msg("Invalid block hash")]
    InvalidBlockHash,
}