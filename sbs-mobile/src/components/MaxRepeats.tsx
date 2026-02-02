import React from 'react';
import {StyleSheet, Switch, Text, TextInput, View} from 'react-native';

interface MaxRepeatsProps {
  enabled: boolean;
  repeats: string;
  onToggle: (value: boolean) => void;
  onRepeatsChange: (value: string) => void;
}

export default function MaxRepeats({
  enabled,
  repeats,
  onToggle,
  onRepeatsChange,
}: MaxRepeatsProps) {
  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.label}>Max Repeats</Text>
        <Switch value={enabled} onValueChange={onToggle} />
      </View>
      {enabled && (
        <TextInput
          style={styles.input}
          placeholder="e.g. 2"
          placeholderTextColor="#999"
          value={repeats}
          onChangeText={onRepeatsChange}
          keyboardType="number-pad"
        />
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
  },
});
