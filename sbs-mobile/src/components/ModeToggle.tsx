import React from 'react';
import {StyleSheet, Switch, Text, TextInput, View} from 'react-native';

interface ModeToggleProps {
  online: boolean;
  backendUrl: string;
  onToggle: (value: boolean) => void;
  onUrlChange: (value: string) => void;
}

export default function ModeToggle({
  online,
  backendUrl,
  onToggle,
  onUrlChange,
}: ModeToggleProps) {
  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.label}>Online mode</Text>
        <Switch value={online} onValueChange={onToggle} />
      </View>
      {online && (
        <TextInput
          style={styles.input}
          placeholder="http://10.0.2.2:8080"
          placeholderTextColor="#999"
          value={backendUrl}
          onChangeText={onUrlChange}
          autoCapitalize="none"
          autoCorrect={false}
          keyboardType="url"
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
