import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { LoadTestReport, buildMarkdownReport } from '../components/LoadTestingPanel/LoadTestReport';
import type { HttpRequest, LoadTestConfigDto, LoadTestProgressEventDto } from '../types/ipc';

const config: LoadTestConfigDto = {
  targetRequest: {
    id: 'req_1',
    name: 'Get pets',
    description: null,
    method: 'GET',
    url: 'https://api.example.com/pets',
    headers: [],
    body: null,
    auth: { type: 'NoAuth' },
    variables: {},
    scripts: null,
    grpc_config: null,
  } as HttpRequest,
  virtualUsers: 50,
  durationSeconds: 10,
  rampUpSeconds: 2,
  timeoutMs: 5000,
};

const snapshot: LoadTestProgressEventDto = {
  testId: 'test-1234',
  elapsedSeconds: 9.98,
  activeVus: 0,
  currentRps: 120.5,
  totalRequests: 1000,
  successfulRequests: 980,
  failedRequests: 20,
  bytesPerSecond: 20480,
  percentiles: {
    p50Ms: 12,
    p90Ms: 40,
    p95Ms: 65,
    p99Ms: 90,
    minMs: 3,
    maxMs: 210,
    meanMs: 22,
  },
  statusCodes: [
    { code: 500, count: 5 },
    { code: 200, count: 980 },
    { code: 0, count: 15 },
  ],
  isFinished: true,
};

describe('buildMarkdownReport', () => {
  afterEach(cleanup);

  it('renders configuration, summary, percentiles and sorted status codes', () => {
    const markdown = buildMarkdownReport(config, snapshot);

    expect(markdown).toContain('# Tyny Pulse — Load Test Report');
    expect(markdown).toContain('`test-1234`');
    expect(markdown).toContain('| Method | GET |');
    expect(markdown).toContain('| URL | https://api.example.com/pets |');
    expect(markdown).toContain('| Virtual Users | 50 |');
    expect(markdown).toContain('| Total Requests | 1000 |');
    expect(markdown).toContain('| Success Rate | 98.00% |');
    expect(markdown).toContain('| Effective RPS | 100.20 req/s |');
    expect(markdown).toContain('| p95 | 65.00 |');
    // Status table must be sorted by descending count.
    const statusSection = markdown.slice(markdown.indexOf('## Status Code Breakdown'));
    expect(statusSection).toContain('| 200 |');
    expect(statusSection).toContain('| transport error |');
    expect(statusSection.indexOf('| 200 |')).toBeLessThan(
      statusSection.indexOf('| transport error |'),
    );
    expect(statusSection.indexOf('| transport error |')).toBeLessThan(
      statusSection.indexOf('| 500 |'),
    );
  });

  it('handles a missing config gracefully', () => {
    const markdown = buildMarkdownReport(null, snapshot);
    expect(markdown).toContain('| Method | n/a |');
  });
});

describe('LoadTestReport component', () => {
  let createObjectURLSpy: ReturnType<typeof vi.fn>;
  let clickSpy: ReturnType<typeof vi.fn<() => void>>;

  beforeEach(() => {
    createObjectURLSpy = vi.fn(() => 'blob:mock');
    clickSpy = vi.fn();
    vi.stubGlobal(
      'URL',
      Object.assign(URL, {
        createObjectURL: createObjectURLSpy,
        revokeObjectURL: vi.fn(),
      }),
    );
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(clickSpy);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('hides itself while the run is still in progress', () => {
    const running = { ...snapshot, isFinished: false };
    const { container } = render(<LoadTestReport config={config} snapshot={running} />);
    expect(container.textContent).toBe('');
  });

  it('exports JSON and Markdown reports on click', () => {
    const createdAnchors: HTMLAnchorElement[] = [];
    const createElementOriginal = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation(((tag: string) => {
      const element = createElementOriginal(tag);
      if (tag === 'a') createdAnchors.push(element as HTMLAnchorElement);
      return element;
    }) as typeof document.createElement);

    render(<LoadTestReport config={config} snapshot={snapshot} />);

    fireEvent.click(screen.getByText('Export JSON'));
    fireEvent.click(screen.getByText('Export Markdown'));

    expect(createObjectURLSpy).toHaveBeenCalledTimes(2);
    const [jsonBlob, mdBlob] = createObjectURLSpy.mock.calls.map((call) => call[0] as Blob);
    expect(jsonBlob).toBeInstanceOf(Blob);

    expect(createdAnchors).toHaveLength(2);
    expect(createdAnchors[0]?.getAttribute('download')).toBe('load-test-test-1234.json');
    expect(createdAnchors[1]?.getAttribute('download')).toBe('load-test-test-1234.md');
    expect(mdBlob.type).toBe('text/markdown');
    expect(clickSpy).toHaveBeenCalledTimes(2);
  });
});
