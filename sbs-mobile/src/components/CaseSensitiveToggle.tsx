import React from 'react';
import {StyleSheet, Switch, Text, View} from 'react-native';

interface CaseSensitiveToggleProps {
  enabled: boolean;
  onToggle: (value: boolean) => void;
}

export default function CaseSensitiveToggle({
  enabled,
  onToggle,
}: CaseSensitiveToggleProps) {
  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.label}>Case Sensitive</Text>
        <Switch value={enabled} onValueChange={onToggle} />
      </View>
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
});
