import React from 'react';
import {StyleSheet, Switch, Text, TextInput, View} from 'react-native';

interface LengthLimitsProps {
  enabled: boolean;
  minLength: string;
  maxLength: string;
  onToggle: (value: boolean) => void;
  onMinChange: (value: string) => void;
  onMaxChange: (value: string) => void;
}

export default function LengthLimits({
  enabled,
  minLength,
  maxLength,
  onToggle,
  onMinChange,
  onMaxChange,
}: LengthLimitsProps) {
  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.label}>Word length limits</Text>
        <Switch value={enabled} onValueChange={onToggle} />
      </View>
      {enabled && (
        <>
          <TextInput
            style={styles.input}
            placeholder="Minimum length (default: 4)"
            placeholderTextColor="#999"
            value={minLength}
            onChangeText={onMinChange}
            keyboardType="number-pad"
          />
          <TextInput
            style={[styles.input, styles.inputLast]}
            placeholder="Maximum length (unlimited)"
            placeholderTextColor="#999"
            value={maxLength}
            onChangeText={onMaxChange}
            keyboardType="number-pad"
          />
        </>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  row: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
  },
  input: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 8,
    padding: 12,
    fontSize: 14,
    color: '#333',
    backgroundColor: '#fff',
    marginBottom: 8,
  },
  inputLast: {
    marginBottom: 0,
  },
});
