import {ResultItem, WordEntry, isWordEntry} from '../components/ResultsList';

export function formatPlaintext(results: ResultItem[]): string {
  return results
    .map(item => {
      if (isWordEntry(item)) {
        return `${item.word}\t${item.definition}`;
      }
      return item;
    })
    .join('\n');
}

export function formatJson(results: ResultItem[]): string {
  if (results.length === 0) return '[]';
  if (isWordEntry(results[0])) {
    return JSON.stringify(
      (results as WordEntry[]).map(({word, definition, url}) => ({
        word,
        definition,
        url,
      })),
      null,
      2,
    );
  }
  return JSON.stringify(results, null, 2);
}

export function formatMarkdown(results: ResultItem[]): string {
  return results
    .map(item => {
      if (isWordEntry(item)) {
        return `**${item.word}**\n${item.definition}`;
      }
      return `**${item}**`;
    })
    .join('\n\n');
}
