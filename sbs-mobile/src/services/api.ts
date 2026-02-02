export interface WordEntry {
  word: string;
  definition: string;
  url: string;
}

export interface ValidationSummary {
  candidates: number;
  validated: number;
  entries: WordEntry[];
}

export type SolveResponse = string[] | ValidationSummary;

function isValidationSummary(data: unknown): data is ValidationSummary {
  return (
    typeof data === 'object' &&
    data !== null &&
    !Array.isArray(data) &&
    'entries' in data
  );
}

/**
 * Call the backend /solve endpoint.
 *
 * @param backendUrl - Base URL of the backend (e.g. "http://10.0.2.2:8080")
 * @param letters - Available letters
 * @param present - Required letter(s)
 * @param repeats - Max repeats (null = unlimited)
 * @param validator - Optional validator name
 * @param apiKey - Optional API key for the validator
 * @param validatorUrl - Optional custom validator URL
 */
export async function solveOnline(
  backendUrl: string,
  letters: string,
  present: string,
  repeats: number | null,
  validator?: string,
  apiKey?: string,
  validatorUrl?: string,
  minLength?: number,
  maxLength?: number,
): Promise<{
  results: string[] | WordEntry[];
  candidateCount: number | null;
}> {
  const payload: Record<string, unknown> = {letters, present, repeats};

  if (minLength && minLength > 0) {
    payload['minimal-word-length'] = minLength;
  }
  if (maxLength && maxLength > 0) {
    payload['maximal-word-length'] = maxLength;
  }

  if (validator) {
    payload.validator = validator;
    if (validator === 'custom' && validatorUrl) {
      payload['validator-url'] = validatorUrl;
    }
    if ((validator === 'merriam-webster' || validator === 'wordnik') && apiKey) {
      payload['api-key'] = apiKey;
    }
  }

  const response = await fetch(`${backendUrl.replace(/\/+$/, '')}/solve`, {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error(`Backend returned ${response.status}`);
  }

  const data = await response.json();

  if (isValidationSummary(data)) {
    return {
      results: data.entries,
      candidateCount: data.candidates,
    };
  }

  return {results: data as string[], candidateCount: null};
}
