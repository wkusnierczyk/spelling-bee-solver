import {NativeModules} from 'react-native';
import {solve, version} from '../../src/native/SbsSolver';

const mockSolver = NativeModules.SbsSolver;

beforeEach(() => {
  jest.clearAllMocks();
});

describe('solve', () => {
  it('parses JSON response and returns words', async () => {
    mockSolver.solve.mockResolvedValue('{"words":["apple","ape"]}');

    const result = await solve('aple', 'a', 1);
    expect(result).toEqual(['apple', 'ape']);
  });

  it('forwards parameters to native module', async () => {
    mockSolver.solve.mockResolvedValue('{"words":[]}');

    await solve('xyz', 'x', 2);
    expect(mockSolver.solve).toHaveBeenCalledWith('xyz', 'x', 2, 0, 0, false);
  });

  it('uses default repeats of 0', async () => {
    mockSolver.solve.mockResolvedValue('{"words":[]}');

    await solve('abc', 'a');
    expect(mockSolver.solve).toHaveBeenCalledWith('abc', 'a', 0, 0, 0, false);
  });
});

describe('version', () => {
  it('returns version string from native module', async () => {
    mockSolver.version.mockResolvedValue('1.2.3');

    const v = await version();
    expect(v).toBe('1.2.3');
  });
});
