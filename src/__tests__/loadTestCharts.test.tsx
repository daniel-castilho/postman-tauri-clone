import { afterEach, describe, expect, it } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import { MultiLineChart, StatusCodeDonut } from '../components/LoadTestingPanel/LoadTestCharts';
import type { LatencyPercentilesDto } from '../types/ipc';

function percentiles(overrides: Partial<LatencyPercentilesDto> = {}): LatencyPercentilesDto {
  return {
    p50Ms: 10,
    p90Ms: 20,
    p95Ms: 30,
    p99Ms: 40,
    minMs: 5,
    maxMs: 50,
    meanMs: 15,
    ...overrides,
  };
}

describe('MultiLineChart', () => {
  afterEach(cleanup);

  it('renders title, unit and one polyline per series', () => {
    render(
      <MultiLineChart
        title="Requests per Second"
        unit="req/s over time"
        series={[{ label: 'RPS', color: '#8b5cf6', values: [10, 20, 30] }]}
      />,
    );

    expect(screen.getByText('Requests per Second')).toBeTruthy();
    expect(screen.getByText('req/s over time')).toBeTruthy();
    expect(screen.getByText('RPS')).toBeTruthy();

    const polylines = document.querySelectorAll('polyline');
    expect(polylines).toHaveLength(1);
    // Three points produce "x,y x,y x,y" coordinates.
    expect((polylines[0] as SVGPolylineElement).getAttribute('points')?.split(' ')).toHaveLength(3);
  });

  it('renders every series label for multi-line charts', () => {
    render(
      <MultiLineChart
        title="Latency Percentiles"
        unit="ms"
        series={[
          { label: 'p50', color: '#22c55e', values: [1] },
          { label: 'p95', color: '#f59e0b', values: [2] },
          { label: 'p99', color: '#ef4444', values: [3] },
        ]}
      />,
    );

    expect(screen.getByText('p50')).toBeTruthy();
    expect(screen.getByText('p95')).toBeTruthy();
    expect(screen.getByText('p99')).toBeTruthy();
  });

  it('renders grid lines even without data points', () => {
    const { container } = render(
      <MultiLineChart
        title="Empty"
        unit=""
        series={[{ label: 'RPS', color: '#000', values: [] }]}
      />,
    );
    expect(container.querySelectorAll('.lt-grid-line')).toHaveLength(3);
    const polyline = container.querySelector('polyline');
    expect(polyline?.getAttribute('points') ?? '').toBe('');
  });
});

describe('StatusCodeDonut', () => {
  it('buckets status codes into class groups plus transport errors', () => {
    render(
      <StatusCodeDonut
        statusCodes={[
          { code: 200, count: 3 },
          { code: 301, count: 1 },
          { code: 500, count: 2 },
          { code: 0, count: 4 },
        ]}
      />,
    );

    const legend = document.querySelectorAll('.lt-donut-legend-item');
    const byLabel = new Map<string, string>();
    legend.forEach((item) => {
      const spans = item.querySelectorAll('span');
      const label = spans[1]?.textContent;
      const value = item.querySelector('strong')?.textContent;
      if (label && value !== undefined) byLabel.set(label, value);
    });
    expect(byLabel.get('2xx')).toBe('3');
    expect(byLabel.get('3xx')).toBe('1');
    expect(byLabel.get('5xx')).toBe('2');
    expect(byLabel.get('Errors')).toBe('4');
  });

  it('shows a zero total when there are no samples', () => {
    render(<StatusCodeDonut statusCodes={[]} />);
    expect(screen.getByText('0 requests')).toBeTruthy();
  });

  it('abbreviates large totals', () => {
    render(
      <StatusCodeDonut
        statusCodes={[
          { code: 200, count: 1500 },
          { code: 404, count: 500 },
        ]}
      />,
    );
    expect(screen.getByText('2.0k')).toBeTruthy();
  });
});
