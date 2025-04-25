import { useWallet, useConnection } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useState } from 'react'
import { setupProgram } from '../lib/solana'
import * as anchor from '@project-serum/anchor'

export default function CoinFlipPage() {
  const { publicKey, wallet } = useWallet()
  const { connection } = useConnection()
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleCreateGame = async () => {
    if (!publicKey || !wallet) {
      setError('Connect your wallet first')
      return
    }

    setLoading(true)
    setError('')

    try {
      const provider = new anchor.AnchorProvider(connection, wallet, {})
      const program = setupProgram(provider)

      const tx = await program.methods
        .createGame(
          new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL), // 0.1 SOL
          0, // Side choice
          'd6a09b9b3546a...' // Заглушка сид
        )
        .rpc()

      console.log('Transaction signature:', tx)
      alert(`Game created! TX: ${tx.slice(0, 10)}...`)
    } catch (err: any) {
      console.error(err)
      setError(err.message)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="container">
      <WalletMultiButton />
      {error && <div className="error">{error}</div>}

      <button 
        onClick={handleCreateGame}
        disabled={loading}
        className={`create-btn ${loading ? 'loading' : ''}`}
      >
        {loading ? 'Creating...' : 'Create Game (0.1 SOL)'}
      </button>
    </div>
  )
}
