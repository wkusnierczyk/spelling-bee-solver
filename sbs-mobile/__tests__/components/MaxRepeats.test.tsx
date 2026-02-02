import React from 'react';
import ReactTestRenderer, {act} from 'react-test-renderer';
import {Switch} from 'react-native';
import MaxRepeats from '../../src/components/MaxRepeats';

describe('MaxRepeats', () => {
  const defaultProps = {
    enabled: false,
    repeats: '',
    onToggle: jest.fn(),
    onRepeatsChange: jest.fn(),
  };

  it('hides input when disabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(<MaxRepeats {...defaultProps} />);
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    expect(inputs).toHaveLength(0);
  });

  it('shows input when enabled', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <MaxRepeats {...defaultProps} enabled={true} />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    expect(inputs).toHaveLength(1);
    expect(inputs[0].props.keyboardType).toBe('number-pad');
  });

  it('calls onRepeatsChange when value changes', async () => {
    const onRepeatsChange = jest.fn();
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <MaxRepeats {...defaultProps} enabled={true} onRepeatsChange={onRepeatsChange} />,
      );
    });
    const input = tree!.root.findByType('TextInput' as any);
    act(() => {
      input.props.onChangeText('3');
    });
    expect(onRepeatsChange).toHaveBeenCalledWith('3');
  });

  it('calls onToggle when switch is toggled', async () => {
    const onToggle = jest.fn();
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <MaxRepeats {...defaultProps} onToggle={onToggle} />,
      );
    });
    const toggle = tree!.root.findByType(Switch);
    act(() => {
      toggle.props.onValueChange(true);
    });
    expect(onToggle).toHaveBeenCalledWith(true);
  });
});
