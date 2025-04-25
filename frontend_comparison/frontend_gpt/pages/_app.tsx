import { WalletAdapterNetwork } from '@solana/wallet-adapter-base'
import { WalletProvider } from '@solana/wallet-adapter-react'
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui'
import { PhantomWalletAdapter } from '@solana/wallet-adapter-wallets'
import type { AppProps } from 'next/app'
import { useMemo } from 'react'
import '../styles/globals.css'

require('@solana/wallet-adapter-react-ui/styles.css')

function MyApp({ Component, pageProps }: AppProps) {
  const wallets = useMemo(() => [new PhantomWalletAdapter()], [])

  return (
    <WalletProvider wallets={wallets} autoConnect>
      <WalletModalProvider>
        <Component {...pageProps} />
      </WalletModalProvider>
    </WalletProvider>
  )
}

export default MyApp
