import React, {useEffect} from 'react';
import {StyleSheet, Switch, Text, TextInput, View} from 'react-native';
import {Picker} from '@react-native-picker/picker';
import AsyncStorage from '@react-native-async-storage/async-storage';

export type ValidatorKind = '' | 'free-dictionary' | 'merriam-webster' | 'wordnik' | 'custom';

const STORAGE_KEY_MW = 'apiKey:merriam-webster';
const STORAGE_KEY_WORDNIK = 'apiKey:wordnik';
const STORAGE_KEY_VALIDATOR = 'lastValidator';
const STORAGE_KEY_ENABLED = 'validatorEnabled';

interface ValidatorPickerProps {
  enabled: boolean;
  validator: ValidatorKind;
  apiKey: string;
  validatorUrl: string;
  onToggle: (value: boolean) => void;
  onValidatorChange: (value: ValidatorKind) => void;
  onApiKeyChange: (value: string) => void;
  onValidatorUrlChange: (value: string) => void;
}

export default function ValidatorPicker({
  enabled,
  validator,
  apiKey,
  validatorUrl,
  onToggle,
  onValidatorChange,
  onApiKeyChange,
  onValidatorUrlChange,
}: ValidatorPickerProps) {
  // Load persisted validator on mount
  useEffect(() => {
    AsyncStorage.getItem(STORAGE_KEY_VALIDATOR).then(stored => {
      if (stored) {
        onValidatorChange(stored as ValidatorKind);
      }
    });
    AsyncStorage.getItem(STORAGE_KEY_ENABLED).then(stored => {
      if (stored === 'true') {
        onToggle(true);
      }
    });
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // Load persisted API key when validator changes
  useEffect(() => {
    const key =
      validator === 'merriam-webster' ? STORAGE_KEY_MW :
      validator === 'wordnik' ? STORAGE_KEY_WORDNIK : null;
    if (key) {
      AsyncStorage.getItem(key).then(stored => {
        if (stored) {
          onApiKeyChange(stored);
        }
      });
    }
  }, [validator]); // eslint-disable-line react-hooks/exhaustive-deps

  const handleToggle = (value: boolean) => {
    onToggle(value);
    AsyncStorage.setItem(STORAGE_KEY_ENABLED, String(value));
  };

  const handleValidatorChange = (value: ValidatorKind) => {
    onValidatorChange(value);
    AsyncStorage.setItem(STORAGE_KEY_VALIDATOR, value);
  };

  // Persist API key on change
  const handleApiKeyChange = (value: string) => {
    onApiKeyChange(value);
    const key =
      validator === 'merriam-webster' ? STORAGE_KEY_MW :
      validator === 'wordnik' ? STORAGE_KEY_WORDNIK : null;
    if (key) {
      if (value) {
        AsyncStorage.setItem(key, value);
      } else {
        AsyncStorage.removeItem(key);
      }
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.row}>
        <Text style={styles.label}>Dictionary Validator</Text>
        <Switch value={enabled} onValueChange={handleToggle} />
      </View>

      {enabled && (
        <>
          <View style={styles.pickerWrapper}>
            <Picker
              selectedValue={validator}
              onValueChange={handleValidatorChange}
              style={styles.picker}>
              <Picker.Item label="Free Dictionary" value="free-dictionary" />
              <Picker.Item label="Merriam-Webster" value="merriam-webster" />
              <Picker.Item label="Wordnik" value="wordnik" />
              <Picker.Item label="Custom URL" value="custom" />
            </Picker>
          </View>

          {validator === 'custom' && (
            <TextInput
              style={styles.input}
              placeholder="e.g. https://api.dictionaryapi.dev/api/v2/entries/en"
              placeholderTextColor="#999"
              value={validatorUrl}
              onChangeText={onValidatorUrlChange}
              autoCapitalize="none"
              autoCorrect={false}
              keyboardType="url"
            />
          )}

          {(validator === 'merriam-webster' || validator === 'wordnik') && (
            <TextInput
              style={styles.input}
              placeholder="Enter your API key"
              placeholderTextColor="#999"
              value={apiKey}
              onChangeText={handleApiKeyChange}
              autoCapitalize="none"
              autoCorrect={false}
              secureTextEntry
            />
          )}
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
  pickerWrapper: {
    borderWidth: 1,
    borderColor: '#ccc',
    borderRadius: 8,
    backgroundColor: '#fff',
    marginBottom: 8,
  },
  picker: {
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
});
