// frontend_minimal/lib/solana.ts

import { Program, AnchorProvider } from '@project-serum/anchor'
import { PublicKey } from '@solana/web3.js'
import idl from './idl/solana_casino.json'

// 1. Адрес программы
export const programID = new PublicKey(idl.metadata.address)

// 2. Функция инициализации Anchor Program
export const setupProgram = (provider: AnchorProvider) => {
  return new Program(idl as any, programID, provider)
}
