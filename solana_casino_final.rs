
// Final smart contract for Solpot-style Coinflip casino
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};
use solana_program::{program::invoke, system_instruction};

declare_id!("CoinF1ipHVCxXGj7bfoShbFNoMwAU9a1DTVaJYfMXAUMQ");

#[program]
pub mod solana_casino {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, commission_fee: u8) -> Result<()> {
        require!(commission_fee <= 10, ErrorCode::CommissionTooHigh);
        let casino = &mut ctx.accounts.casino;
        casino.admin = ctx.accounts.admin.key();
        casino.vault = ctx.accounts.vault.key();
        casino.commission_fee = commission_fee;
        casino.total_games = 0;
        casino.total_volume = 0;
        Ok(())
    }

    pub fn create_game(
        ctx: Context<CreateGame>,
        bet_amount: u64,
        side_choice: u8,
        server_seed_hash: String,
    ) -> Result<()> {
        require!(bet_amount >= 1_000_000, ErrorCode::BetTooSmall); // 0.001 SOL
        require!(side_choice == 0 || side_choice == 1, ErrorCode::InvalidSide);
        require!(server_seed_hash.len() == 64, ErrorCode::InvalidHash);

        // Transfer SOL to vault
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.player.key(),
                &ctx.accounts.vault.key(),
                bet_amount,
            ),
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        let game = &mut ctx.accounts.game;
        let casino = &mut ctx.accounts.casino;

        game.creator = ctx.accounts.player.key();
        game.bet_amount = bet_amount;
        game.creator_side = side_choice;
        game.server_seed_hash = server_seed_hash;
        game.state = GameState::Created;
        game.joiner = None;
        game.winner = None;
        game.server_seed = None;
        game.block_hash = None;

        casino.total_games += 1;
        casino.total_volume = casino.total_volume.checked_add(bet_amount).unwrap();

        emit!(GameCreatedEvent {
            game_id: ctx.accounts.game.key(),
            creator: ctx.accounts.player.key(),
            bet_amount,
            creator_side: side_choice,
        });

        Ok(())
    }

    pub fn join_game(ctx: Context<JoinGame>, selected_block_hash: String) -> Result<()> {
        let game = &mut ctx.accounts.game;
        require!(game.state == GameState::Created, ErrorCode::InvalidGameState);
        require!(game.creator != ctx.accounts.player.key(), ErrorCode::CannotJoinOwnGame);

        game.joiner = Some(ctx.accounts.player.key());
        game.block_hash = Some(selected_block_hash);
        game.state = GameState::Joined;

        emit!(GameJoinedEvent {
            game_id: ctx.accounts.game.key(),
            joiner: ctx.accounts.player.key(),
            block_hash: selected_block_hash,
        });

        Ok(())
    }

    pub fn resolve_game(
        ctx: Context<ResolveGame>,
        server_seed: String,
        winner: Pubkey,
    ) -> Result<()> {
        let game = &mut ctx.accounts.game;
        let casino = &mut ctx.accounts.casino;

        require!(game.state == GameState::Joined, ErrorCode::InvalidGameState);
        require!(ctx.accounts.admin.key() == casino.admin, ErrorCode::Unauthorized);
        require!(game.creator == winner || game.joiner.unwrap() == winner, ErrorCode::InvalidWinner);

        game.server_seed = Some(server_seed.clone());
        game.winner = Some(winner);
        game.state = GameState::Completed;

        let total_pot = game.bet_amount.checked_mul(2).unwrap();
        let commission = total_pot.checked_mul(casino.commission_fee as u64).unwrap() / 100;
        let payout = total_pot.checked_sub(commission).unwrap();

        invoke(
            &system_instruction::transfer(
                &ctx.accounts.vault.key(),
                &winner,
                payout,
            ),
            &[
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.winner.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;

        emit!(GameResolvedEvent {
            game_id: ctx.accounts.game.key(),
            winner,
            prize_amount: payout,
            commission_amount: commission,
            server_seed,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = admin, space = 8 + 32 + 32 + 1 + 8 + 8 + 1)]
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateGame<'info> {
    #[account(init, payer = player, space = 8 + 32 + 8 + 1 + 64 + 1 + 32 + 32 + 64 + 64)]
    pub game: Account<'info, Game>,
    #[account(mut)]
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub player: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ResolveGame<'info> {
    #[account(mut)]
    pub game: Account<'info, Game>,
    pub casino: Account<'info, Casino>,
    #[account(mut)]
    pub vault: SystemAccount<'info>,
    #[account(mut)]
    pub winner: SystemAccount<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Casino {
    pub admin: Pubkey,
    pub vault: Pubkey,
    pub commission_fee: u8,
    pub total_games: u64,
    pub total_volume: u64,
}

#[account]
pub struct Game {
    pub creator: Pubkey,
    pub bet_amount: u64,
    pub creator_side: u8,
    pub server_seed_hash: String,
    pub state: GameState,
    pub joiner: Option<Pubkey>,
    pub winner: Option<Pubkey>,
    pub server_seed: Option<String>,
    pub block_hash: Option<String>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GameState {
    Created,
    Joined,
    Completed,
    Cancelled,
}

#[event]
pub struct GameCreatedEvent {
    pub game_id: Pubkey,
    pub creator: Pubkey,
    pub bet_amount: u64,
    pub creator_side: u8,
}

#[event]
pub struct GameJoinedEvent {
    pub game_id: Pubkey,
    pub joiner: Pubkey,
    pub block_hash: String,
}

#[event]
pub struct GameResolvedEvent {
    pub game_id: Pubkey,
    pub winner: Pubkey,
    pub prize_amount: u64,
    pub commission_amount: u64,
    pub server_seed: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Bet amount too small")]
    BetTooSmall,
    #[msg("Invalid side")]
    InvalidSide,
    #[msg("Invalid hash")]
    InvalidHash,
    #[msg("Invalid game state")]
    InvalidGameState,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid winner")]
    InvalidWinner,
    #[msg("Commission fee too high")]
    CommissionTooHigh,
}
