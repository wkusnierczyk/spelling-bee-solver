import {validateWords} from '../../src/services/validator';

const mockFetch = jest.fn();
global.fetch = mockFetch as any;

beforeEach(() => {
  mockFetch.mockReset();
  jest.useFakeTimers();
});

afterEach(() => {
  jest.useRealTimers();
});

function flushTimersAndMicrotasks() {
  jest.advanceTimersByTime(200);
}

describe('validateWords', () => {
  it('looks up words via Free Dictionary', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [{meanings: [{definitions: [{definition: 'a greeting'}]}]}],
    });

    const promise = validateWords(['hello'], 'free-dictionary', '', '');
    // Flush throttle timers
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.candidates).toBe(1);
    expect(result.validated).toBe(1);
    expect(result.entries[0].word).toBe('hello');
    expect(result.entries[0].definition).toBe('a greeting');
    expect(mockFetch).toHaveBeenCalledWith(
      'https://api.dictionaryapi.dev/api/v2/entries/en/hello',
      expect.any(Object),
    );
  });

  it('looks up words via Merriam-Webster', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [{shortdef: ['a test definition']}],
    });

    const promise = validateWords(['test'], 'merriam-webster', 'mykey', '');
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.validated).toBe(1);
    expect(result.entries[0].definition).toBe('a test definition');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('key=mykey'),
      expect.any(Object),
    );
  });

  it('looks up words via Wordnik', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [{text: 'wordnik def'}],
    });

    const promise = validateWords(['cat'], 'wordnik', 'wkey', '');
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.validated).toBe(1);
    expect(result.entries[0].definition).toBe('wordnik def');
    expect(mockFetch).toHaveBeenCalledWith(
      expect.stringContaining('api_key=wkey'),
      expect.any(Object),
    );
  });

  it('looks up words via Custom URL', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [{meanings: [{definitions: [{definition: 'custom def'}]}]}],
    });

    const promise = validateWords(['word'], 'custom', '', 'https://my-api.example.com/v2/');
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.validated).toBe(1);
    expect(mockFetch).toHaveBeenCalledWith(
      'https://my-api.example.com/v2/word',
      expect.any(Object),
    );
  });

  it('returns null for 404 responses (Free Dictionary)', async () => {
    mockFetch.mockResolvedValue({ok: false, status: 404});

    const promise = validateWords(['zzzzz'], 'free-dictionary', '', '');
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.candidates).toBe(1);
    expect(result.validated).toBe(0);
    expect(result.entries).toHaveLength(0);
  });

  it('skips words on network error', async () => {
    mockFetch.mockRejectedValue(new Error('Network error'));

    const promise = validateWords(['hello'], 'free-dictionary', '', '');
    await jest.runAllTimersAsync();
    const result = await promise;

    expect(result.candidates).toBe(1);
    expect(result.validated).toBe(0);
  });

  it('calls progress callback', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      status: 200,
      json: async () => [{meanings: [{definitions: [{definition: 'def'}]}]}],
    });

    const onProgress = jest.fn();
    const promise = validateWords(['a', 'b'], 'free-dictionary', '', '', onProgress);
    await jest.runAllTimersAsync();
    await promise;

    expect(onProgress).toHaveBeenCalledWith(1, 2);
    expect(onProgress).toHaveBeenCalledWith(2, 2);
  });
});
