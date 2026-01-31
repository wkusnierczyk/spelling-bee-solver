import { useState } from 'react'
import axios from 'axios'

interface SolveRequest {
  letters: string;
  present: string;
  repeats: number | null;
  validator?: string;
  "api-key"?: string;
  "validator-url"?: string;
}

interface WordEntry {
  word: string;
  definition: string;
  url: string;
}

type ResultItem = string | WordEntry;

function isWordEntry(item: ResultItem): item is WordEntry {
  return typeof item === 'object' && 'word' in item && 'definition' in item && 'url' in item;
}

function App() {
  const [letters, setLetters] = useState('')
  const [present, setPresent] = useState('')
  const [repeats, setRepeats] = useState('')
  const [validator, setValidator] = useState('')
  const [results, setResults] = useState<ResultItem[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handleSolve = async () => {
    setLoading(true);
    setError(null);
    setResults([]);

    try {
      const payload: SolveRequest = {
        letters: letters,
        present: present,
        repeats: repeats ? parseInt(repeats) : null
      };

      if (validator) {
        payload.validator = validator;
      }

      const response = await axios.post('/solve', payload);
      setResults(response.data);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Failed to connect to backend';
      console.error(err);
      setError(message);
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

      <div className="input-group">
        <label>Dictionary Validator (Optional)</label>
        <select value={validator} onChange={(e) => setValidator(e.target.value)}>
          <option value="">None (seed dictionary only)</option>
          <option value="free-dictionary">Free Dictionary</option>
          <option value="merriam-webster">Merriam-Webster</option>
          <option value="wordnik">Wordnik</option>
        </select>
      </div>

      <button onClick={handleSolve} disabled={!isValid || loading}>
        {loading ? 'Solving...' : 'Solve'}
      </button>

      {error && <div className="error">{error}</div>}

      <div className="results">
        {results.length > 0 && <h3>Found {results.length} words:</h3>}
        {results.map((item) => {
          if (isWordEntry(item)) {
            return (
              <div key={item.word} className="word-card">
                <a href={item.url} target="_blank" rel="noopener noreferrer" className="word-link">
                  {item.word}
                </a>
                <span className="word-definition">{item.definition}</span>
              </div>
            );
          }
          return <div key={item} className="word-card">{item}</div>;
        })}
      </div>
    </div>
  )
}

export default App
