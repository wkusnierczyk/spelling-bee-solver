import React from 'react';
import ReactTestRenderer, {act} from 'react-test-renderer';
import LetterInput from '../../src/components/LetterInput';

describe('LetterInput', () => {
  const defaultProps = {
    letters: '',
    present: '',
    onLettersChange: jest.fn(),
    onPresentChange: jest.fn(),
  };

  it('shows first available letter as placeholder when letters provided', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <LetterInput {...defaultProps} letters="abcdefg" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const presentInput = inputs[1];
    expect(presentInput.props.placeholder).toBe('e.g. a or ab');
  });

  it('shows empty placeholder when no letters', async () => {
    let tree: ReactTestRenderer.ReactTestRenderer;
    await act(async () => {
      tree = ReactTestRenderer.create(
        <LetterInput {...defaultProps} letters="" />,
      );
    });
    const inputs = tree!.root.findAllByType('TextInput' as any);
    const presentInput = inputs[1];
    expect(presentInput.props.placeholder).toBe('');
  });
});
