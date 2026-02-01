import {solveOnline} from '../../src/services/api';

const mockFetch = jest.fn();
global.fetch = mockFetch as any;

beforeEach(() => {
  mockFetch.mockReset();
});

describe('solveOnline', () => {
  it('sends correct payload', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ['apple', 'ape'],
    });

    await solveOnline('http://localhost:8080', 'aple', 'a', 1);

    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/solve',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({letters: 'aple', present: 'a', repeats: 1}),
      }),
    );
  });

  it('returns string array response', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ['apple', 'ape'],
    });

    const result = await solveOnline('http://localhost:8080', 'aple', 'a', null);
    expect(result.results).toEqual(['apple', 'ape']);
    expect(result.candidateCount).toBeNull();
  });

  it('returns ValidationSummary response', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        candidates: 10,
        validated: 2,
        entries: [
          {word: 'apple', definition: 'a fruit', url: 'https://example.com/apple'},
        ],
      }),
    });

    const result = await solveOnline('http://localhost:8080', 'aple', 'a', null, 'free-dictionary');
    expect(result.candidateCount).toBe(10);
    expect(result.results).toHaveLength(1);
  });

  it('includes validator params in payload', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => [],
    });

    await solveOnline('http://localhost:8080', 'abc', 'a', null, 'merriam-webster', 'mykey');

    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.validator).toBe('merriam-webster');
    expect(body['api-key']).toBe('mykey');
  });

  it('includes custom validator URL in payload', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => [],
    });

    await solveOnline('http://localhost:8080', 'abc', 'a', null, 'custom', undefined, 'https://my-api.com');

    const body = JSON.parse(mockFetch.mock.calls[0][1].body);
    expect(body.validator).toBe('custom');
    expect(body['validator-url']).toBe('https://my-api.com');
  });

  it('strips trailing slash from backend URL', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => [],
    });

    await solveOnline('http://localhost:8080///', 'abc', 'a', null);

    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/solve',
      expect.any(Object),
    );
  });

  it('throws on non-ok response', async () => {
    mockFetch.mockResolvedValue({ok: false, status: 500});

    await expect(
      solveOnline('http://localhost:8080', 'abc', 'a', null),
    ).rejects.toThrow('Backend returned 500');
  });
});
