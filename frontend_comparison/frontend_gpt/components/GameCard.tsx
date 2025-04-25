import { useWallet } from '@solana/wallet-adapter-react'

export default function GameCard({ game }: { game: any }) {
  const { publicKey } = useWallet()

  return (
    <div className="solpot-game-card">
      <div className="players">
        <span>{game.creator?.slice(0, 4)}... vs ???</span>
      </div>
      <div className="bet-amount">{game.amount} SOL</div>
      {publicKey && <button className="solpot-join-btn">Join</button>}
    </div>
  )
}
