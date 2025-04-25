import Link from 'next/link'

export default function Home() {
  return (
    <div style={{ textAlign: 'center', marginTop: '100px' }}>
      <h1>Welcome to Solcasino</h1>
      <Link href="/coinflip">
        <button style={{ marginTop: '20px', padding: '10px 20px' }}>Play CoinFlip</button>
      </Link>
    </div>
  )
}
