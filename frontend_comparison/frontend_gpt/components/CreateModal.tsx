import { useState } from 'react'

export default function CreateModal({ isOpen, onClose }: { isOpen: boolean, onClose: () => void }) {
  const [amount, setAmount] = useState(0.1)

  if (!isOpen) return null

  return (
    <div className="solpot-modal-overlay">
      <div className="solpot-modal">
        <h2>Create Game</h2>
        <input
          type="number"
          value={amount}
          onChange={(e) => setAmount(parseFloat(e.target.value))}
          min="0.1"
          step="0.1"
        />
        <button onClick={onClose}>Cancel</button>
        <button onClick={() => alert(`Creating game with ${amount} SOL`)}>
          Confirm
        </button>
      </div>
    </div>
  )
}
