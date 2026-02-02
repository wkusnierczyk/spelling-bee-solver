import React from 'react';
import {StyleSheet, Text, TextInput, View} from 'react-native';

interface LetterInputProps {
  letters: string;
  present: string;
  caseSensitive: boolean;
  onLettersChange: (value: string) => void;
  onPresentChange: (value: string) => void;
}

export default function LetterInput({
  letters,
  present,
  caseSensitive,
  onLettersChange,
  onPresentChange,
}: LetterInputProps) {
  const autoCapitalize = caseSensitive ? 'sentences' : 'none';
  return (
    <View>
      <View style={styles.inputGroup}>
        <Text style={styles.label}>Available Letters</Text>
        <TextInput
          style={styles.input}
          placeholder="e.g. abcdefg"
          placeholderTextColor="#999"
          value={letters}
          onChangeText={onLettersChange}
          autoCapitalize={autoCapitalize}
          autoCorrect={false}
        />
      </View>

      <View style={styles.inputGroup}>
        <Text style={styles.label}>Required Letters</Text>
        <TextInput
          style={styles.input}
          placeholder={letters.length > 1 ? `e.g. ${letters[0]} or ${letters.slice(0, 2)}` : ''}
          placeholderTextColor="#999"
          value={present}
          onChangeText={onPresentChange}
          autoCapitalize={autoCapitalize}
          autoCorrect={false}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  inputGroup: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
  },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 8,
    padding: 12,
    fontSize: 16,
    color: '#333',
    backgroundColor: '#fff',
  },
});
