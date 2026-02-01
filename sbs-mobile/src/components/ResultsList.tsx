import React from 'react';
import {FlatList, Linking, StyleSheet, Text, TouchableOpacity, View} from 'react-native';

export interface WordEntry {
  word: string;
  definition: string;
  url: string;
}

export type ResultItem = string | WordEntry;

export function isWordEntry(item: ResultItem): item is WordEntry {
  return typeof item === 'object' && 'word' in item && 'definition' in item;
}

interface ResultsListProps {
  results: ResultItem[];
  candidateCount: number | null;
}

function WordCard({item}: {item: ResultItem}) {
  if (isWordEntry(item)) {
    return (
      <View style={styles.card}>
        <TouchableOpacity onPress={() => Linking.openURL(item.url)}>
          <Text style={styles.wordLink}>{item.word}</Text>
        </TouchableOpacity>
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

export default function ResultsList({results, candidateCount}: ResultsListProps) {
  if (results.length === 0) {
    return null;
  }

  const header = candidateCount !== null
    ? `Found ${results.length} words (from ${candidateCount} candidates):`
    : `Found ${results.length} words:`;

  return (
    <View style={styles.container}>
      <Text style={styles.header}>{header}</Text>
      <FlatList
        data={results}
        keyExtractor={(item, index) =>
          typeof item === 'string' ? item : item.word || String(index)
        }
        renderItem={({item}) => <WordCard item={item} />}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
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
