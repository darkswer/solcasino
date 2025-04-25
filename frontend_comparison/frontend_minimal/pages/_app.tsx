import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets'
import { WalletProvider } from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import type { AppProps } from 'next/app'
import { useMemo } from 'react'
import '../styles/globals.css'

require('@solana/wallet-adapter-react-ui/styles.css')

export default function App({ Component, pageProps }: AppProps) {
  const wallets = useMemo(() => [new PhantomWalletAdapter()], [])

  return (
    <WalletProvider wallets={wallets} autoConnect>
      <WalletModalProvider>
        <Component {...pageProps} />
      </WalletModalProvider>
    </WalletProvider>
  )
}
