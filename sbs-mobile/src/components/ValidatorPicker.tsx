import React, {useEffect} from 'react';
import {StyleSheet, Text, TextInput, View} from 'react-native';
import {Picker} from '@react-native-picker/picker';
import AsyncStorage from '@react-native-async-storage/async-storage';

export type ValidatorKind = '' | 'free-dictionary' | 'merriam-webster' | 'wordnik' | 'custom';

const STORAGE_KEY_MW = 'apiKey:merriam-webster';
const STORAGE_KEY_WORDNIK = 'apiKey:wordnik';

interface ValidatorPickerProps {
  validator: ValidatorKind;
  apiKey: string;
  validatorUrl: string;
  onValidatorChange: (value: ValidatorKind) => void;
  onApiKeyChange: (value: string) => void;
  onValidatorUrlChange: (value: string) => void;
}

export default function ValidatorPicker({
  validator,
  apiKey,
  validatorUrl,
  onValidatorChange,
  onApiKeyChange,
  onValidatorUrlChange,
}: ValidatorPickerProps) {
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
      <Text style={styles.label}>Dictionary Validator</Text>
      <View style={styles.pickerWrapper}>
        <Picker
          selectedValue={validator}
          onValueChange={onValidatorChange}
          style={styles.picker}>
          <Picker.Item label="None (seed dictionary only)" value="" />
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
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    marginBottom: 16,
  },
  label: {
    fontSize: 14,
    fontWeight: '600',
    color: '#333',
    marginBottom: 6,
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
