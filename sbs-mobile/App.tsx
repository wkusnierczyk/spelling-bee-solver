import React, {useState} from 'react';
import {
  ActivityIndicator,
  FlatList,
  Pressable,
  StatusBar,
  StyleSheet,
  Text,
  View,
} from 'react-native';
import {SafeAreaProvider, SafeAreaView} from 'react-native-safe-area-context';
import LetterInput from './src/components/LetterInput';
import ModeToggle from './src/components/ModeToggle';
import ValidatorPicker, {ValidatorKind} from './src/components/ValidatorPicker';
import {ResultItem, isWordEntry} from './src/components/ResultsList';
import {solve} from './src/native/SbsSolver';
import {solveOnline} from './src/services/api';
import {validateWords} from './src/services/validator';
import {Linking} from 'react-native';

function WordCard({item}: {item: ResultItem}) {
  if (isWordEntry(item)) {
    return (
      <View style={styles.card}>
        <Pressable onPress={() => Linking.openURL(item.url)}>
          <Text style={styles.wordLink}>{item.word}</Text>
        </Pressable>
        <Text style={styles.definition}>{item.definition}</Text>
      </View>
    );
  }
  return (
    <View style={styles.card}>
      <Text style={styles.word}>{item}</Text>
    </View>
  );
}

function App() {
  const [letters, setLetters] = useState('');
  const [present, setPresent] = useState('');
  const [repeats, setRepeats] = useState('');
  const [online, setOnline] = useState(false);
  const [backendUrl, setBackendUrl] = useState('http://10.0.2.2:8080');
  const [validator, setValidator] = useState<ValidatorKind>('');
  const [apiKey, setApiKey] = useState('');
  const [validatorUrl, setValidatorUrl] = useState('');
  const [results, setResults] = useState<ResultItem[]>([]);
  const [candidateCount, setCandidateCount] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState('');
  const [error, setError] = useState<string | null>(null);

  const clearResults = () => {
    setResults([]);
    setCandidateCount(null);
    setError(null);
    setProgress('');
  };

  const handleLettersChange = (value: string) => {
    const unique = [...new Set(value.split(''))].join('');
    setLetters(unique);
    if (present && !unique.includes(present)) {
      setPresent('');
    }
    clearResults();
  };

  const handlePresentChange = (value: string) => {
    if (value.length === 0 || letters.includes(value)) {
      setPresent(value);
      clearResults();
    }
  };

  const handleRepeatsChange = (value: string) => {
    setRepeats(value);
    clearResults();
  };

  const isValid = letters.length > 0 && present.length > 0;

  const handleSolve = async () => {
    setLoading(true);
    setError(null);
    setResults([]);
    setCandidateCount(null);
    setProgress('');

    const repeatsNum = repeats ? parseInt(repeats, 10) : 0;
    let words: string[] = [];

    if (online) {
      try {
        const response = await solveOnline(
          backendUrl,
          letters,
          present,
          repeatsNum || null,
          validator || undefined,
          apiKey || undefined,
          validatorUrl || undefined,
        );
        setResults(response.results);
        setCandidateCount(response.candidateCount);
        setLoading(false);
        return;
      } catch (onlineErr: unknown) {
        const msg =
          onlineErr instanceof Error ? onlineErr.message : 'Online request failed';
        setError(`Online failed (${msg}), falling back to offline...`);
      }
    }

    try {
      words = await solve(letters, present, repeatsNum);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Solve failed';
      setError(message);
      setLoading(false);
      return;
    }

    setCandidateCount(words.length);

    if (validator && words.length > 0) {
      setProgress(`Validating: 0 / ${words.length}`);
      try {
        const result = await validateWords(
          words,
          validator,
          apiKey,
          validatorUrl,
          (done, total) => setProgress(`Validating: ${done} / ${total}`),
        );
        setResults(result.entries);
        setCandidateCount(result.candidates);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : 'Validation failed';
        setError(message);
        setResults(words);
      }
    } else {
      setResults(words);
    }

    setProgress('');
    setLoading(false);
  };

  const header = candidateCount !== null
    ? `Found ${results.length} words (from ${candidateCount} candidates):`
    : `Found ${results.length} words:`;

  const ListHeader = (
    <View>
      <Text style={styles.title}>Spelling Bee Solver</Text>

      <LetterInput
        letters={letters}
        present={present}
        repeats={repeats}
        onLettersChange={handleLettersChange}
        onPresentChange={handlePresentChange}
        onRepeatsChange={handleRepeatsChange}
      />

      <ValidatorPicker
        validator={validator}
        apiKey={apiKey}
        validatorUrl={validatorUrl}
        onValidatorChange={setValidator}
        onApiKeyChange={setApiKey}
        onValidatorUrlChange={setValidatorUrl}
      />

      <ModeToggle
        online={online}
        backendUrl={backendUrl}
        onToggle={setOnline}
        onUrlChange={setBackendUrl}
      />

      <Pressable
        style={[styles.button, (!isValid || loading) && styles.buttonDisabled]}
        onPress={handleSolve}
        disabled={!isValid || loading}>
        {loading ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.buttonText}>Solve</Text>
        )}
      </Pressable>

      {progress !== '' && <Text style={styles.progress}>{progress}</Text>}

      {error && <Text style={styles.error}>{error}</Text>}

      {!loading && !error && results.length === 0 && candidateCount !== null && (
        <Text style={styles.noResults}>No words found</Text>
      )}

      {results.length > 0 && <Text style={styles.header}>{header}</Text>}
    </View>
  );

  return (
    <SafeAreaProvider>
      <StatusBar barStyle="dark-content" />
      <SafeAreaView style={styles.safe}>
        <FlatList
          data={results}
          keyExtractor={(item, index) =>
            typeof item === 'string' ? item : item.word || String(index)
          }
          renderItem={({item}) => <WordCard item={item} />}
          ListHeaderComponent={ListHeader}
          keyboardShouldPersistTaps="handled"
          contentContainerStyle={styles.container}
        />
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
  progress: {
    color: '#007bff',
    fontSize: 14,
    textAlign: 'center',
    marginBottom: 12,
  },
  error: {
    color: '#dc3545',
    fontSize: 14,
    marginBottom: 12,
  },
  noResults: {
    color: '#666',
    fontSize: 16,
    textAlign: 'center',
    marginTop: 16,
  },
  header: {
    fontSize: 16,
    fontWeight: '700',
    color: '#333',
    marginBottom: 8,
  },
  card: {
    backgroundColor: '#f8f9fa',
    borderRadius: 8,
    padding: 12,
    marginBottom: 6,
  },
  word: {
    fontSize: 16,
    color: '#333',
  },
  wordLink: {
    fontSize: 16,
    color: '#007bff',
    fontWeight: '600',
  },
  definition: {
    fontSize: 13,
    color: '#666',
    marginTop: 4,
  },
});

export default App;
