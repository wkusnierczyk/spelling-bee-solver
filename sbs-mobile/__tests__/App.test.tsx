/**
 * @format
 */

import React from 'react';
import ReactTestRenderer, {act} from 'react-test-renderer';
import {NativeModules} from 'react-native';
import App from '../App';

const mockSolver = NativeModules.SbsSolver;

const mockFetch = jest.fn();
global.fetch = mockFetch as any;

beforeEach(() => {
  jest.clearAllMocks();
  mockFetch.mockReset();
  mockSolver.solve.mockResolvedValue('{"words":["test"]}');
});

describe('App', () => {
  it('renders correctly', async () => {
    await act(async () => {
      ReactTestRenderer.create(<App />);
    });
  });

  it('deduplicates letters on input', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    // Find the LetterInput's first TextInput (available letters)
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const lettersInput = inputs[0];

    await act(async () => {
      lettersInput.props.onChangeText('aabb');
    });

    // After dedup, value should be "ab"
    expect(lettersInput.props.value).toBe('ab');
  });

  it('constrains present to available letters', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    const inputs = tree!.root.findAllByType('TextInput' as any);
    const lettersInput = inputs[0];
    const presentInput = inputs[1];

    // Set letters first
    await act(async () => {
      lettersInput.props.onChangeText('abc');
    });

    // Try to set present to a letter not in available
    await act(async () => {
      presentInput.props.onChangeText('z');
    });

    // Present should remain empty since 'z' is not in 'abc'
    expect(presentInput.props.value).toBe('');

    // Set present to a valid letter
    await act(async () => {
      presentInput.props.onChangeText('a');
    });

    expect(presentInput.props.value).toBe('a');
  });

  it('clears results on input change', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    const inputs = tree!.root.findAllByType('TextInput' as any);
    const lettersInput = inputs[0];
    const presentInput = inputs[1];

    // Set up valid input and solve
    await act(async () => {
      lettersInput.props.onChangeText('abcdefg');
    });
    await act(async () => {
      presentInput.props.onChangeText('a');
    });

    // Find Pressable
    const pressables = tree!.root.findAllByProps({disabled: false}).filter(
      n => n.props.onPress,
    );

    if (pressables.length > 0) {
      await act(async () => {
        await pressables[0].props.onPress();
      });
    }

    // Change letters — results should clear
    await act(async () => {
      lettersInput.props.onChangeText('xyz');
    });

    // No results text should be absent (results cleared)
    const allText = tree!.root.findAllByType('Text' as any).map(t => t.props.children);
    expect(allText).not.toContain(expect.stringContaining('Found'));
  });

  it('disables solve button when input is invalid', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    // Find the Pressable solve button (disabled when no letters/present)
    const pressables = tree!.root.findAllByProps({disabled: true}).filter(
      n => n.props.onPress,
    );
    expect(pressables.length).toBeGreaterThan(0);
  });

  it('performs offline solve', async () => {
    mockSolver.solve.mockResolvedValue('{"words":["abc","abcd"]}');

    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    const inputs = tree!.root.findAllByType('TextInput' as any);
    await act(async () => {
      inputs[0].props.onChangeText('abcdefg');
    });
    await act(async () => {
      inputs[1].props.onChangeText('a');
    });

    // Find and press solve
    const pressables = tree!.root.findAllByProps({disabled: false}).filter(
      n => n.props.onPress,
    );
    await act(async () => {
      await pressables[0].props.onPress();
    });

    expect(mockSolver.solve).toHaveBeenCalledWith('abcdefg', 'a', 0, 0, 0, 0);
  });

  it('performs case-sensitive offline solve with preserved uppercase', async () => {
    mockSolver.solve.mockResolvedValue('{"words":["wall","walrus"]}');

    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    // Toggle case-sensitive ON first
    const switches = tree!.root.findAll(
      n => n.props && typeof n.props.onValueChange === 'function',
    );
    // Case-sensitive toggle is the first Switch after LetterInput
    const caseSensitiveSwitch = switches[0];
    await act(async () => {
      caseSensitiveSwitch.props.onValueChange(true);
    });

    // Now enter mixed-case letters
    const inputs = tree!.root.findAllByType('TextInput' as any);
    await act(async () => {
      inputs[0].props.onChangeText('Walrus');
    });
    await act(async () => {
      inputs[1].props.onChangeText('Wl');
    });

    // Find and press solve
    const pressables = tree!.root.findAllByProps({disabled: false}).filter(
      n => n.props.onPress,
    );
    await act(async () => {
      await pressables[0].props.onPress();
    });

    expect(mockSolver.solve).toHaveBeenCalledWith('Walrus', 'Wl', 0, 0, 0, 1);
  });

  it('falls back to offline when online fails', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));
    mockSolver.solve.mockResolvedValue('{"words":["fallback"]}');

    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<App />);
    });

    const inputs = tree!.root.findAllByType('TextInput' as any);
    await act(async () => {
      inputs[0].props.onChangeText('abcdefg');
    });
    await act(async () => {
      inputs[1].props.onChangeText('a');
    });

    // Toggle online mode (last switch — after length limits toggle)
    const switches = tree!.root.findAll(
      n => n.props && typeof n.props.onValueChange === 'function',
    );
    if (switches.length > 0) {
      const onlineSwitch = switches[switches.length - 1];
      await act(async () => {
        onlineSwitch.props.onValueChange(true);
      });
    }

    const pressables = tree!.root.findAllByProps({disabled: false}).filter(
      n => n.props.onPress,
    );
    await act(async () => {
      await pressables[0].props.onPress();
    });

    // Should have fallen back to offline solve
    expect(mockSolver.solve).toHaveBeenCalled();
  });
});
