import React from 'react';
import ReactTestRenderer, {act} from 'react-test-renderer';
import {Switch} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import ValidatorPicker from '../../src/components/ValidatorPicker';

const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

beforeEach(() => {
  jest.clearAllMocks();
});

describe('ValidatorPicker', () => {
  const defaultProps = {
    enabled: false,
    validator: '' as const,
    apiKey: '',
    validatorUrl: '',
    onToggle: jest.fn(),
    onValidatorChange: jest.fn(),
    onApiKeyChange: jest.fn(),
    onValidatorUrlChange: jest.fn(),
  };

  it('hides picker when disabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} />,
      );
    });
    const pickers = tree!.root.findAllByType('Picker' as any);
    expect(pickers).toHaveLength(0);
  });

  it('shows picker when enabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="free-dictionary" />,
      );
    });
    const picker = tree!.root.findByType('Picker' as any);
    const items = picker.findAllByType('Picker.Item' as any);
    expect(items).toHaveLength(4);
    expect(items[0].props.value).toBe('free-dictionary');
  });

  it('shows API key input for merriam-webster when enabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="merriam-webster" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const apiKeyInput = inputs.find(i => i.props.placeholder === 'Enter your API key');
    expect(apiKeyInput).toBeTruthy();
  });

  it('shows custom URL input for custom validator when enabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="custom" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const urlInput = inputs.find(i => i.props.keyboardType === 'url');
    expect(urlInput).toBeTruthy();
  });

  it('loads persisted validator and enabled state on mount', async () => {
    mockAsyncStorage.getItem.mockImplementation((key) => {
      if (key === 'lastValidator') {
        return Promise.resolve('wordnik');
      }
      if (key === 'validatorEnabled') {
        return Promise.resolve('true');
      }
      return Promise.resolve(null);
    });

    await act(async () => {
      ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} />,
      );
    });

    expect(defaultProps.onValidatorChange).toHaveBeenCalledWith('wordnik');
    expect(defaultProps.onToggle).toHaveBeenCalledWith(true);
  });

  it('loads persisted API key from AsyncStorage on validator change', async () => {
    mockAsyncStorage.getItem.mockImplementation((key) => {
      if (key === 'apiKey:merriam-webster') {
        return Promise.resolve('stored-key');
      }
      return Promise.resolve(null);
    });

    await act(async () => {
      ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="merriam-webster" />,
      );
    });

    expect(mockAsyncStorage.getItem).toHaveBeenCalledWith('apiKey:merriam-webster');
    expect(defaultProps.onApiKeyChange).toHaveBeenCalledWith('stored-key');
  });

  it('persists validator choice to AsyncStorage', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="free-dictionary" />,
      );
    });
    const picker = tree!.root.findByType('Picker' as any);
    act(() => {
      picker.props.onValueChange('wordnik');
    });
    expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('lastValidator', 'wordnik');
  });

  it('persists toggle state to AsyncStorage', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} />,
      );
    });
    const toggle = tree!.root.findByType(Switch);
    act(() => {
      toggle.props.onValueChange(true);
    });
    expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('validatorEnabled', 'true');
  });

  it('saves API key to AsyncStorage', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="wordnik" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const apiKeyInput = inputs.find(i => i.props.secureTextEntry);
    act(() => {
      apiKeyInput!.props.onChangeText('new-key');
    });

    expect(mockAsyncStorage.setItem).toHaveBeenCalledWith('apiKey:wordnik', 'new-key');
  });

  it('removes API key from AsyncStorage when cleared', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} enabled={true} validator="merriam-webster" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const apiKeyInput = inputs.find(i => i.props.secureTextEntry);
    act(() => {
      apiKeyInput!.props.onChangeText('');
    });

    expect(mockAsyncStorage.removeItem).toHaveBeenCalledWith('apiKey:merriam-webster');
  });
});
