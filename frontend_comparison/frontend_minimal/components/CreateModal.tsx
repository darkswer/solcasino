import { useState } from 'react'

export default function CreateModal({ isOpen, onClose }: { isOpen: boolean, onClose: () => void }) {
  const [amount, setAmount] = useState(0.1)

  if (!isOpen) return null

  return (
    <div className="modal-overlay">
      <div className="modal">
        <h2>Create Game</h2>
        <input 
          type="number" 
          value={amount} 
          onChange={(e) => setAmount(parseFloat(e.target.value))}
          min="0.1"
          step="0.1"
        />
        <button onClick={onClose}>Cancel</button>
        <button onClick={() => alert(`Создаем игру на ${amount} SOL`)}>
          Confirm
        </button>
      </div>
    </div>
  )
}
