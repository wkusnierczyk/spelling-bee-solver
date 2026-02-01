import React from 'react';
import ReactTestRenderer, {act} from 'react-test-renderer';
import AsyncStorage from '@react-native-async-storage/async-storage';
import ValidatorPicker from '../../src/components/ValidatorPicker';

const mockAsyncStorage = AsyncStorage as jest.Mocked<typeof AsyncStorage>;

beforeEach(() => {
  jest.clearAllMocks();
});

describe('ValidatorPicker', () => {
  const defaultProps = {
    validator: '' as const,
    apiKey: '',
    validatorUrl: '',
    onValidatorChange: jest.fn(),
    onApiKeyChange: jest.fn(),
    onValidatorUrlChange: jest.fn(),
  };

  it('renders Picker with correct options', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} />,
      );
    });
    const picker = tree!.root.findByType('Picker' as any);
    const items = picker.findAllByType('Picker.Item' as any);
    expect(items).toHaveLength(5);
    expect(items[0].props.value).toBe('');
    expect(items[1].props.value).toBe('free-dictionary');
  });

  it('shows API key input for merriam-webster', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} validator="merriam-webster" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const apiKeyInput = inputs.find(i => i.props.placeholder === 'Enter your API key');
    expect(apiKeyInput).toBeTruthy();
  });

  it('shows custom URL input for custom validator', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} validator="custom" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const urlInput = inputs.find(i => i.props.keyboardType === 'url');
    expect(urlInput).toBeTruthy();
  });

  it('loads persisted API key from AsyncStorage on validator change', async () => {
    mockAsyncStorage.getItem.mockResolvedValue('stored-key');

    await act(async () => {
      ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} validator="merriam-webster" />,
      );
    });

    expect(mockAsyncStorage.getItem).toHaveBeenCalledWith('apiKey:merriam-webster');
    expect(defaultProps.onApiKeyChange).toHaveBeenCalledWith('stored-key');
  });

  it('saves API key to AsyncStorage', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <ValidatorPicker {...defaultProps} validator="wordnik" />,
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
        <ValidatorPicker {...defaultProps} validator="merriam-webster" />,
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
