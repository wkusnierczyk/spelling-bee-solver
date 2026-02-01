import {WordEntry} from './api';

export type ValidatorKind = '' | 'free-dictionary' | 'merriam-webster' | 'wordnik' | 'custom';

const HTTP_TIMEOUT = 10_000;
const THROTTLE_DELAY = 100;

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function fetchWithTimeout(url: string, timeout = HTTP_TIMEOUT): Promise<Response> {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeout);
  try {
    return await fetch(url, {signal: controller.signal});
  } finally {
    clearTimeout(id);
  }
}

async function lookupFreeDictionary(
  word: string,
  baseUrl = 'https://api.dictionaryapi.dev/api/v2/entries/en',
): Promise<WordEntry | null> {
  const resp = await fetchWithTimeout(`${baseUrl}/${word}`);
  if (resp.status === 404) {
    return null;
  }
  if (!resp.ok) {
    return null;
  }
  const body = await resp.json();
  const definition =
    body?.[0]?.meanings?.[0]?.definitions?.[0]?.definition ?? 'No definition available';
  return {word, definition, url: `https://en.wiktionary.org/wiki/${word}`};
}

async function lookupMerriamWebster(
  word: string,
  apiKey: string,
): Promise<WordEntry | null> {
  const url = `https://dictionaryapi.com/api/v3/references/collegiate/json/${word}?key=${apiKey}`;
  const resp = await fetchWithTimeout(url);
  if (!resp.ok) {
    return null;
  }
  const body = await resp.json();
  if (!Array.isArray(body) || body.length === 0 || typeof body[0] === 'string') {
    return null;
  }
  const definition = body[0]?.shortdef?.[0] ?? 'No definition available';
  return {word, definition, url: `https://www.merriam-webster.com/dictionary/${word}`};
}

async function lookupWordnik(
  word: string,
  apiKey: string,
): Promise<WordEntry | null> {
  const url = `https://api.wordnik.com/v4/word.json/${word}/definitions?limit=1&api_key=${apiKey}`;
  const resp = await fetchWithTimeout(url);
  if (resp.status === 404) {
    return null;
  }
  if (!resp.ok) {
    return null;
  }
  const body = await resp.json();
  if (!Array.isArray(body) || body.length === 0) {
    return null;
  }
  const definition = body[0]?.text ?? 'No definition available';
  return {word, definition, url: `https://www.wordnik.com/words/${word}`};
}

export interface ValidationResult {
  candidates: number;
  validated: number;
  entries: WordEntry[];
}

export async function validateWords(
  words: string[],
  validator: ValidatorKind,
  apiKey: string,
  customUrl: string,
  onProgress?: (done: number, total: number) => void,
): Promise<ValidationResult> {
  const entries: WordEntry[] = [];

  for (let i = 0; i < words.length; i++) {
    const word = words[i];
    let entry: WordEntry | null = null;

    try {
      switch (validator) {
        case 'free-dictionary':
          entry = await lookupFreeDictionary(word);
          break;
        case 'merriam-webster':
          entry = await lookupMerriamWebster(word, apiKey);
          break;
        case 'wordnik':
          entry = await lookupWordnik(word, apiKey);
          break;
        case 'custom':
          entry = await lookupFreeDictionary(word, customUrl.replace(/\/+$/, ''));
          break;
      }
    } catch {
      // skip word on network error
    }

    if (entry) {
      entries.push(entry);
    }

    onProgress?.(i + 1, words.length);

    if (i < words.length - 1) {
      await sleep(THROTTLE_DELAY);
    }
  }

  return {candidates: words.length, validated: entries.length, entries};
}
