import {NativeModules} from 'react-native';

const {SbsSolver} = NativeModules;

export interface SolveResult {
  words: string[];
}

/**
 * Solve a Spelling Bee puzzle using the native FFI library.
 *
 * @param letters - Available letters
 * @param present - Required letter(s)
 * @param repeats - Max letter repetitions (0 = unlimited)
 * @returns Array of matching words
 */
export async function solve(
  letters: string,
  present: string,
  repeats: number = 0,
  minLength: number = 0,
  maxLength: number = 0,
): Promise<string[]> {
  const json: string = await SbsSolver.solve(letters, present, repeats, minLength, maxLength);
  const result: SolveResult = JSON.parse(json);
  return result.words;
}

/**
 * Get the native FFI library version.
 */
export async function version(): Promise<string> {
  return SbsSolver.version();
}
