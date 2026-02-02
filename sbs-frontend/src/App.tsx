import { useState, useEffect, useRef } from 'react'
import axios from 'axios'
import { formatPlaintext, formatJson, formatMarkdown } from './formats'

interface SolveRequest {
  letters: string;
  present: string;
  repeats: number | null;
  validator?: string;
  "api-key"?: string;
  "validator-url"?: string;
  "minimal-word-length"?: number;
  "maximal-word-length"?: number;
  "case-sensitive"?: boolean;
}

interface WordEntry {
  word: string;
  definition: string;
  url: string;
}

interface ValidationSummary {
  candidates: number;
  validated: number;
  entries: WordEntry[];
}

type ResultItem = string | WordEntry;

function isWordEntry(item: ResultItem): item is WordEntry {
  return typeof item === 'object' && 'word' in item && 'definition' in item && 'url' in item;
}

function App() {
  const [letters, setLetters] = useState('')
  const [present, setPresent] = useState('')
  const [repeatsEnabled, setRepeatsEnabled] = useState(false)
  const [repeats, setRepeats] = useState('1')
  const [validatorEnabled, setValidatorEnabled] = useState(false)
  const [validator, setValidator] = useState('free-dictionary')
  const [validatorUrl, setValidatorUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [results, setResults] = useState<ResultItem[]>([])
  const [candidateCount, setCandidateCount] = useState<number | null>(null)
  const [lengthLimits, setLengthLimits] = useState(false)
  const [minLength, setMinLength] = useState('4')
  const [maxLength, setMaxLength] = useState('')
  const [caseSensitive, setCaseSensitive] = useState(false)
  const [loading, setLoading] = useState(false)
  const [progress, setProgress] = useState('')
  const [error, setError] = useState<string | null>(null)

  const clearResults = () => {
    setResults([]);
    setCandidateCount(null);
    setError(null);
    setProgress('');
  };

  // Load API key from localStorage when validator changes
  useEffect(() => {
    if (validator) {
      const saved = localStorage.getItem(`apiKey:${validator}`);
      setApiKey(saved ?? '');
    } else {
      setApiKey('');
    }
  }, [validator]);

  const sanitizePositiveInt = (value: string): string => {
    const digits = value.replace(/[^0-9]/g, '');
    if (!digits) return '';
    const n = parseInt(digits, 10);
    return n > 0 ? String(n) : '';
  };

  const handleLettersChange = (value: string) => {
    const normalized = caseSensitive ? value : value.toLowerCase();
    const unique = [...new Set(normalized.split(''))].join('');
    setLetters(unique);
    const filtered = present.split('').filter(c => unique.includes(c)).join('');
    if (filtered !== present) setPresent(filtered);
    clearResults();
  };

  const handlePresentChange = (value: string) => {
    const normalized = caseSensitive ? value : value.toLowerCase();
    const unique = [...new Set(normalized.split(''))].join('');
    if (caseSensitive) {
      const uppercaseCount = unique.split('').filter(c => c !== c.toLowerCase()).length;
      if (uppercaseCount > 1) return;
    }
    if (unique.split('').every(c => letters.includes(c))) {
      setPresent(unique);
      clearResults();
    }
  };

  const handleCaseSensitiveToggle = (checked: boolean) => {
    setCaseSensitive(checked);
    if (!checked) {
      setLetters(letters.toLowerCase());
      setPresent(present.toLowerCase());
    }
    clearResults();
  };

  const handleRepeatsChange = (value: string) => {
    setRepeats(sanitizePositiveInt(value));
    clearResults();
  };

  const handleMinLengthChange = (value: string) => {
    const sanitized = sanitizePositiveInt(value);
    setMinLength(sanitized);
    if (sanitized && maxLength) {
      const min = parseInt(sanitized, 10);
      const max = parseInt(maxLength, 10);
      if (min > max) {
        setMaxLength(sanitized);
      }
    }
    clearResults();
  };

  const handleMaxLengthChange = (value: string) => {
    const sanitized = sanitizePositiveInt(value);
    setMaxLength(sanitized);
    if (sanitized && minLength) {
      const max = parseInt(sanitized, 10);
      const min = parseInt(minLength, 10);
      if (max < min) {
        setMinLength(sanitized);
      }
    }
    clearResults();
  };

  const handleApiKeyChange = (value: string) => {
    setApiKey(value);
    if (validator) {
      if (value) {
        localStorage.setItem(`apiKey:${validator}`, value);
      } else {
        localStorage.removeItem(`apiKey:${validator}`);
      }
    }
  };

  const handleSolve = async () => {
    setLoading(true);
    setError(null);
    setResults([]);
    setCandidateCount(null);
    setProgress('');

    const payload: SolveRequest = {
      letters: letters,
      present: present,
      repeats: repeatsEnabled && repeats ? parseInt(repeats, 10) : null
    };

    if (caseSensitive) {
      payload["case-sensitive"] = true;
    }

    if (lengthLimits) {
      if (minLength) {
        payload["minimal-word-length"] = parseInt(minLength);
      }
      if (maxLength) {
        payload["maximal-word-length"] = parseInt(maxLength);
      }
    }

    if (validatorEnabled) {
      payload.validator = validator;
      if (validator === 'custom' && validatorUrl) {
        payload["validator-url"] = validatorUrl;
      }
      if ((validator === 'merriam-webster' || validator === 'wordnik') && apiKey) {
        payload["api-key"] = apiKey;
      }
    }

    // Use SSE streaming endpoint when a validator is selected
    if (validatorEnabled) {
      try {
        const response = await fetch('/solve-stream', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(payload),
        });

        if (!response.ok) {
          throw new Error(await response.text() || 'Request failed');
        }

        const reader = response.body?.getReader();
        if (!reader) throw new Error('No response body');

        const decoder = new TextDecoder();
        let buffer = '';

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() ?? '';

          for (const line of lines) {
            if (!line.startsWith('data: ')) continue;
            const data = JSON.parse(line.slice(6));

            if (data.progress) {
              setProgress(`Validating: ${data.progress.done} / ${data.progress.total}`);
            } else if (data.error) {
              setError(data.error);
            } else if (data.result) {
              const result = data.result;
              if (result.entries) {
                const summary = result as ValidationSummary;
                setCandidateCount(summary.candidates);
                setResults(summary.entries);
              } else {
                setResults(result);
              }
            }
          }
        }
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Failed to connect to backend';
        console.error(err);
        setError(message);
      } finally {
        setProgress('');
        setLoading(false);
      }
    } else {
      // No validator — use the regular endpoint
      try {
        const response = await axios.post('/solve', payload);
        setResults(response.data);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Failed to connect to backend';
        console.error(err);
        setError(message);
      } finally {
        setLoading(false);
      }
    }
  };

  const [downloadOpen, setDownloadOpen] = useState(false);
  const downloadRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (downloadRef.current && !downloadRef.current.contains(e.target as Node)) {
        setDownloadOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  const handleDownload = (format: 'plain' | 'json' | 'markdown') => {
    const formatters = { plain: formatPlaintext, json: formatJson, markdown: formatMarkdown };
    const extensions = { plain: 'txt', json: 'json', markdown: 'md' };
    const mimeTypes = { plain: 'text/plain', json: 'application/json', markdown: 'text/markdown' };

    const content = formatters[format](results);
    const blob = new Blob([content], { type: mimeTypes[format] });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `sbs-results.${extensions[format]}`;
    a.click();
    URL.revokeObjectURL(url);
    setDownloadOpen(false);
  };

  const isValid = letters.length > 0;

  return (
    <div className="container">
      <h1 className="title">Spelling Bee Solver</h1>

      <div className="input-group">
        <label>Available Letters</label>
        <input
          placeholder="e.g. abcdefg"
          value={letters}
          onChange={(e) => handleLettersChange(e.target.value)}
        />
      </div>

      <div className="input-group">
        <label>Required Letters</label>
        <input
          placeholder={letters.length > 1 ? `e.g. ${letters[0]} or ${letters.slice(0, 2)}` : ''}
          value={present}
          onChange={(e) => handlePresentChange(e.target.value)}
        />
      </div>

      <button onClick={handleSolve} disabled={!isValid || loading}>
        {loading ? 'Solving...' : 'Solve'}
      </button>

      {progress !== '' && <div className="progress">{progress}</div>}

      {error && <div className="error">{error}</div>}

      <div className="input-group toggle-row">
        <label>Case Sensitive</label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={caseSensitive}
            onChange={(e) => handleCaseSensitiveToggle(e.target.checked)}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      <div className="input-group toggle-row">
        <label>Max Repeats</label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={repeatsEnabled}
            onChange={(e) => { setRepeatsEnabled(e.target.checked); clearResults(); }}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      {repeatsEnabled && (
        <div className="input-group">
          <input
            type="number"
            min="1"
            value={repeats}
            onChange={(e) => handleRepeatsChange(e.target.value)}
          />
        </div>
      )}

      <div className="input-group toggle-row">
        <label>Word Length Limits</label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={lengthLimits}
            onChange={(e) => { setLengthLimits(e.target.checked); clearResults(); }}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      {lengthLimits && (
        <>
          <div className="input-group">
            <label>Minimum Length</label>
            <input
              type="number"
              min="1"
              placeholder="4"
              value={minLength}
              onChange={(e) => handleMinLengthChange(e.target.value)}
            />
          </div>
          <div className="input-group">
            <label>Maximum Length</label>
            <input
              type="number"
              min="1"
              placeholder="Unlimited"
              value={maxLength}
              onChange={(e) => handleMaxLengthChange(e.target.value)}
            />
          </div>
        </>
      )}

      <div className="input-group toggle-row">
        <label>Dictionary Validator</label>
        <label className="toggle">
          <input
            type="checkbox"
            checked={validatorEnabled}
            onChange={(e) => { setValidatorEnabled(e.target.checked); clearResults(); }}
          />
          <span className="toggle-slider" />
        </label>
      </div>

      {validatorEnabled && (
        <div className="input-group">
          <select value={validator} onChange={(e) => setValidator(e.target.value)}>
            <option value="free-dictionary">Free Dictionary</option>
            <option value="merriam-webster">Merriam-Webster</option>
            <option value="wordnik">Wordnik</option>
            <option value="custom">Custom URL</option>
          </select>
        </div>
      )}

      {validatorEnabled && validator === 'custom' && (
        <div className="input-group">
          <label>Custom Validator URL</label>
          <input
            placeholder="e.g. https://api.dictionaryapi.dev/api/v2/entries/en"
            value={validatorUrl}
            onChange={(e) => setValidatorUrl(e.target.value)}
          />
        </div>
      )}

      {validatorEnabled && (validator === 'merriam-webster' || validator === 'wordnik') && (
        <div className="input-group">
          <label>API Key</label>
          <input
            type="password"
            placeholder="Enter your API key"
            value={apiKey}
            onChange={(e) => handleApiKeyChange(e.target.value)}
          />
        </div>
      )}

      <div className="results">
        {results.length > 0 && (
          <>
            <div className="results-header">
              <h3>
                Found {results.length} words
                {candidateCount !== null && ` (from ${candidateCount} candidates)`}
                :
              </h3>
              <div className="download-dropdown" ref={downloadRef}>
                <button
                  className="download-button"
                  onClick={() => setDownloadOpen(!downloadOpen)}
                >
                  Download ▾
                </button>
                {downloadOpen && (
                  <div className="download-menu">
                    <button onClick={() => handleDownload('plain')}>Plaintext (.txt)</button>
                    <button onClick={() => handleDownload('json')}>JSON (.json)</button>
                    <button onClick={() => handleDownload('markdown')}>Markdown (.md)</button>
                  </div>
                )}
              </div>
            </div>
          </>
        )}
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
