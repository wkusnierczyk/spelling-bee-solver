import { useState } from 'react'
import axios from 'axios'

interface SolveRequest {
  letters: string;
  present: string;
  repeats: number | null;
}

function App() {
  const [letters, setLetters] = useState('')
  const [present, setPresent] = useState('')
  const [repeats, setRepeats] = useState('')
  const [results, setResults] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSolve = async () => {
    setLoading(true);
    setError(null);
    setResults([]);

    try {
      // Relative URL:
      // - In Local Dev: Vite proxies this to localhost:8080
      // - In Cloud Prod: Nginx proxies this to backend-service
      const payload: SolveRequest = {
        letters: letters,
        present: present,
        repeats: repeats ? parseInt(repeats) : null
      };
      
      const response = await axios.post('/solve', payload);
      setResults(response.data);
    } catch (err: any) {
      console.error(err);
      setError(err.message || 'Failed to connect to backend');
    } finally {
      setLoading(false);
    }
  };

  const isValid = letters.length > 0 && present.length > 0;

  return (
    <div className="container">
      <h1 style={{textAlign: 'center', marginBottom: '1.5rem'}}>Spelling Bee Solver</h1>
      
      <div className="input-group">
        <label>Available Letters</label>
        <input 
          placeholder="e.g. abcdefg" 
          value={letters}
          onChange={(e) => setLetters(e.target.value)}
        />
      </div>

      <div className="input-group">
        <label>Obligatory Letter</label>
        <input 
          placeholder="e.g. a" 
          value={present}
          onChange={(e) => setPresent(e.target.value)}
        />
      </div>

      <div className="input-group">
        <label>Max Repeats (Optional)</label>
        <input 
          type="number"
          placeholder="Unlimited" 
          value={repeats}
          onChange={(e) => setRepeats(e.target.value)}
        />
      </div>

      <button onClick={handleSolve} disabled={!isValid || loading}>
        {loading ? 'Solving...' : 'Solve'}
      </button>

      {error && <div className="error">{error}</div>}

      <div className="results">
        {results.length > 0 && <h3>Found {results.length} words:</h3>}
        {results.map((word) => (
          <div key={word} className="word-card">{word}</div>
        ))}
      </div>
    </div>
  )
}

export default App