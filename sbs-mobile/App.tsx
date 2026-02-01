import React, {useState} from 'react';
import {
  ActivityIndicator,
  ScrollView,
  StatusBar,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from 'react-native';
import {SafeAreaProvider, SafeAreaView} from 'react-native-safe-area-context';
import LetterInput from './src/components/LetterInput';
import ResultsList, {ResultItem} from './src/components/ResultsList';
import {solve} from './src/native/SbsSolver';

function App() {
  const [letters, setLetters] = useState('');
  const [present, setPresent] = useState('');
  const [repeats, setRepeats] = useState('');
  const [results, setResults] = useState<ResultItem[]>([]);
  const [candidateCount, setCandidateCount] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isValid = letters.length > 0 && present.length > 0;

  const handleSolve = async () => {
    setLoading(true);
    setError(null);
    setResults([]);
    setCandidateCount(null);

    try {
      const words = await solve(letters, present, repeats ? parseInt(repeats, 10) : 0);
      setResults(words);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Solve failed';
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <SafeAreaProvider>
      <StatusBar barStyle="dark-content" />
      <SafeAreaView style={styles.safe}>
        <ScrollView contentContainerStyle={styles.container}>
          <Text style={styles.title}>Spelling Bee Solver</Text>

          <LetterInput
            letters={letters}
            present={present}
            repeats={repeats}
            onLettersChange={setLetters}
            onPresentChange={setPresent}
            onRepeatsChange={setRepeats}
          />

          <TouchableOpacity
            style={[styles.button, (!isValid || loading) && styles.buttonDisabled]}
            onPress={handleSolve}
            disabled={!isValid || loading}>
            {loading ? (
              <ActivityIndicator color="#fff" />
            ) : (
              <Text style={styles.buttonText}>Solve</Text>
            )}
          </TouchableOpacity>

          {error && <Text style={styles.error}>{error}</Text>}

          <ResultsList results={results} candidateCount={candidateCount} />
        </ScrollView>
      </SafeAreaView>
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  safe: {
    flex: 1,
    backgroundColor: '#fff',
  },
  container: {
    padding: 20,
    flexGrow: 1,
  },
  title: {
    fontSize: 24,
    fontWeight: '700',
    color: '#333',
    textAlign: 'center',
    marginBottom: 24,
  },
  button: {
    backgroundColor: '#007bff',
    borderRadius: 8,
    padding: 14,
    alignItems: 'center',
    marginBottom: 16,
  },
  buttonDisabled: {
    backgroundColor: '#99c2ff',
  },
  buttonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  error: {
    color: '#dc3545',
    fontSize: 14,
    marginBottom: 12,
  },
});

export default App;
