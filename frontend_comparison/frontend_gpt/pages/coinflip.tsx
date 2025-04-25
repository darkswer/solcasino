import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useState } from 'react'
import CreateModal from '../components/CreateModal'
import GameCard from '../components/GameCard'

export default function CoinFlipPage() {
  const { publicKey } = useWallet()
  const [isModalOpen, setModalOpen] = useState(false)
  const [games, setGames] = useState<any[]>([]) // Заглушка

  return (
    <div className="solpot-style-container">
      <WalletMultiButton />
      {publicKey && (
        <button onClick={() => setModalOpen(true)} className="solpot-create-btn">
          Create Game
        </button>
      )}
      <div className="games-list">
        {games.map((game, i) => (
          <GameCard key={i} game={game} />
        ))}
      </div>
      <CreateModal isOpen={isModalOpen} onClose={() => setModalOpen(false)} />
    </div>
  )
}
