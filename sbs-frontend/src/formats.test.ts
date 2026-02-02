import { describe, it, expect } from 'vitest';
import { formatPlaintext, formatJson, formatMarkdown } from './formats';

const unvalidated = ['apple', 'ample', 'maple'];

const validated = [
  { word: 'apple', definition: 'A fruit', url: 'https://example.com/apple' },
  { word: 'maple', definition: 'A tree', url: 'https://example.com/maple' },
];

describe('formatPlaintext', () => {
  it('formats unvalidated results as one word per line', () => {
    expect(formatPlaintext(unvalidated)).toBe('apple\nample\nmaple');
  });

  it('formats validated results as word-tab-definition per line', () => {
    expect(formatPlaintext(validated)).toBe(
      'apple\tA fruit\nmaple\tA tree',
    );
  });

  it('returns empty string for empty results', () => {
    expect(formatPlaintext([])).toBe('');
  });
});

describe('formatJson', () => {
  it('formats unvalidated results as a JSON array of strings', () => {
    const parsed = JSON.parse(formatJson(unvalidated));
    expect(parsed).toEqual(['apple', 'ample', 'maple']);
  });

  it('formats validated results as a JSON array of objects', () => {
    const parsed = JSON.parse(formatJson(validated));
    expect(parsed).toEqual([
      { word: 'apple', definition: 'A fruit', url: 'https://example.com/apple' },
      { word: 'maple', definition: 'A tree', url: 'https://example.com/maple' },
    ]);
  });

  it('returns empty array for empty results', () => {
    expect(formatJson([])).toBe('[]');
  });
});

describe('formatMarkdown', () => {
  it('formats unvalidated results as bold words separated by blank lines', () => {
    expect(formatMarkdown(unvalidated)).toBe(
      '**apple**\n\n**ample**\n\n**maple**',
    );
  });

  it('formats validated results as bold word + definition separated by blank lines', () => {
    expect(formatMarkdown(validated)).toBe(
      '**apple**\nA fruit\n\n**maple**\nA tree',
    );
  });

  it('returns empty string for empty results', () => {
    expect(formatMarkdown([])).toBe('');
  });
});
