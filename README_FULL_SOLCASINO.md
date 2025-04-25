# Solcasino — Coinflip Game on Solana

A simple on-chain Solana casino that replicates the logic and UX of [Solpot.com](https://solpot.com). This is a hybrid architecture project combining:

- **Anchor-based smart contract**
- **Node.js backend**
- **(Soon) React frontend**

The game is: **2 players enter, 1 winner gets 96% of the pot.**  
Randomness is generated using `serverSeed + blockHash + gameId`.

---

## Project Structure

```
/contracts/           # Solana smart contract (Anchor / Rust)
/backend/             # Node.js backend with Express API
/frontend/            # Placeholder for future Next.js app
/old_versions/        # Archive of early contract versions
README.md             # You're reading this
```

---

## Smart Contract: `solana_casino_final.rs`

- Written with Anchor framework.
- Stores game info (players, sides, serverSeedHash, blockHash).
- Receives SOL deposits from players.
- `resolve_game()` can only be called by admin (backend).
- Sends payout to winner after deducting 4% fee.
- Emits Anchor events for frontend integration.

---

## Backend Overview (`/backend`)

A minimal Express.js backend that:
- Generates `serverSeed` and hash on `/create`
- Accepts joiners via `/join`, calculates winner
- Sends payout to winner on `/claim`
- Uses a local in-memory DB (object) for game tracking
- Has fallback `SystemProgram.transfer` logic

> For real deployment, use persistent DB like PostgreSQL or Firebase.

---

## Environment Variables (.env)

Rename `.env.example` to `.env` and fill in:

```env
ADMIN_PRIVATE_KEY=base64-encoded-solana-private-key
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=your_deployed_program_id
```

---

## Dev Setup Instructions

### 1. Install Anchor

```bash
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### 2. Build and deploy contract

```bash
cd contracts
anchor build
anchor deploy --provider.cluster devnet
```

After deploy, update `.env` with your program ID.

### 3. Run backend

```bash
cd backend
npm install
npm start
```

Backend will run on `http://localhost:3000`.

---

## Game API Endpoints

| Method | Endpoint            | Description               |
|--------|---------------------|---------------------------|
| POST   | `/api/games/create` | Start a new game          |
| POST   | `/api/games/join`   | Join an existing game     |
| POST   | `/api/games/claim`  | Claim winnings (admin)    |
| GET    | `/api/games/:id`    | Get game state            |
| POST   | `/api/games/verify` | Verify fairness (hashes)  |

---

## Deployment Suggestions

- Use **VPS** like VDSina or Hetzner (1–2 GB RAM is enough)
- Use `pm2` for process management
- Secure `.env` and admin key
- Use SSL/HTTPS for frontend + backend API

---

## Author & Contributions

Built by [@darkswer](https://github.com/darkswer) using GPT-4o + Claude  
Architecture reverse-engineered from [Solpot.com](https://solpot.com)