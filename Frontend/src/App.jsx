import { useEffect, useState } from 'react'
import MemberList from './components/MemberList.jsx'
import { getHello } from './services/api.js'

export default function App() {
  const [data, setData] = useState(null)
  const [error, setError] = useState('')

  useEffect(() => {
    getHello()
      .then(setData)
      .catch((err) => setError(err.message))
  }, [])

  return (
    <main className="container">
      <section className="hero">
        <p className="eyebrow">GO PRODUCT SKELETON</p>
        <h1>{data?.message ?? 'Loading...'}</h1>
        <p className="subtitle">
          A minimal full-stack test product: Go + Gin + GORM + PostgreSQL + React + Vite.
        </p>
      </section>

      <section className="panel">
        <div className="panel-header">
          <h2>Project Members</h2>
          <span>Loaded from PostgreSQL</span>
        </div>

        {error && <p className="error">{error}</p>}
        {data && <MemberList members={data.members} />}
      </section>
    </main>
  )
}
