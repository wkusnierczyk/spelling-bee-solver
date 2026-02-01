import React from 'react';
import {StyleSheet, Text, TextInput, View} from 'react-native';

interface LetterInputProps {
  letters: string;
  present: string;
  repeats: string;
  onLettersChange: (value: string) => void;
  onPresentChange: (value: string) => void;
  onRepeatsChange: (value: string) => void;
}

export default function LetterInput({
  letters,
  present,
  repeats,
  onLettersChange,
  onPresentChange,
  onRepeatsChange,
}: LetterInputProps) {
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
          autoCapitalize="none"
          autoCorrect={false}
        />
      </View>

      <View style={styles.inputGroup}>
        <Text style={styles.label}>Required Letter</Text>
        <TextInput
          style={styles.input}
          placeholder="e.g. a"
          placeholderTextColor="#999"
          value={present}
          onChangeText={onPresentChange}
          autoCapitalize="none"
          autoCorrect={false}
          maxLength={1}
        />
      </View>

      <View style={styles.inputGroup}>
        <Text style={styles.label}>Max Repeats (Optional)</Text>
        <TextInput
          style={styles.input}
          placeholder="Unlimited"
          placeholderTextColor="#999"
          value={repeats}
          onChangeText={onRepeatsChange}
          keyboardType="number-pad"
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
