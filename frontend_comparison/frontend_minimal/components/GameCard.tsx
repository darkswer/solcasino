import { useWallet } from '@solana/wallet-adapter-react'

interface Game {
  id: string
  creator: string
  amount: number
}

export default function GameCard({ game }: { game: Game }) {
  const { publicKey } = useWallet()

  return (
    <div className="game-card">
      <div className="players">
        <span>{game.creator.slice(0, 4)}... vs ???</span>
      </div>
      <div className="bet-amount">{game.amount} SOL</div>
      {publicKey && <button className="join-btn">Join</button>}
    </div>
  )
}
