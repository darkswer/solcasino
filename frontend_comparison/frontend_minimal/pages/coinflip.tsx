import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useState } from 'react'
import CreateModal from '../components/CreateModal'
import GameCard from '../components/GameCard'

interface Game {
  id: string
  creator: string
  amount: number
}

export default function CoinFlipPage() {
  const { publicKey } = useWallet()
  const [isModalOpen, setModalOpen] = useState(false)
  const [games, setGames] = useState<Game[]>([
    { id: "1", creator: "Alice", amount: 0.5 },
    { id: "2", creator: "Bob", amount: 1.2 },
  ])

  return (
    <div className="container">
      <WalletMultiButton />
      
      {publicKey && (
        <button onClick={() => setModalOpen(true)} className="create-btn">
          Create Game
        </button>
      )}

      <div className="games-list">
        {games.map(game => (
          <GameCard key={game.id} game={game} />
        ))}
      </div>

      <CreateModal isOpen={isModalOpen} onClose={() => setModalOpen(false)} />
    </div>
  )
}
