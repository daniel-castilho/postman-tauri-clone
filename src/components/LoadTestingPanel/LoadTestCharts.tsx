import React from 'react';
import type { LatencyPercentilesDto } from '../../types/ipc';

export interface LoadTestChartPoint {
  elapsedSeconds: number;
  currentRps: number;
  percentiles: LatencyPercentilesDto;
}

interface SeriesSpec {
  label: string;
  color: string;
  values: number[];
}

const VIEW_WIDTH = 600;
const VIEW_HEIGHT = 180;

function buildPoints(values: number[], min: number, max: number): string {
  if (values.length === 0) return '';
  const span = max - min || 1;
  const stepX = values.length > 1 ? VIEW_WIDTH / (values.length - 1) : VIEW_WIDTH;
  return values
    .map((value, index) => {
      const x = index * stepX;
      const y = VIEW_HEIGHT - ((value - min) / span) * VIEW_HEIGHT;
      return `${x.toFixed(2)},${y.toFixed(2)}`;
    })
    .join(' ');
}

function axisBounds(series: SeriesSpec[]): { min: number; max: number } {
  const all = series.flatMap((entry) => entry.values);
  if (all.length === 0) return { min: 0, max: 1 };
  const rawMin = Math.min(...all);
  const rawMax = Math.max(...all);
  if (rawMax === rawMin) {
    return { min: Math.max(0, rawMin - 1), max: rawMax + 1 };
  }
  return { min: 0, max: rawMax * 1.1 };
}

interface LineChartProps {
  title: string;
  unit: string;
  series: SeriesSpec[];
}

export const MultiLineChart: React.FC<LineChartProps> = ({ title, unit, series }) => {
  const bounds = axisBounds(series);
  return (
    <div className="lt-chart-card">
      <div className="lt-chart-header">
        <span className="lt-chart-title">{title}</span>
        <span className="lt-chart-unit">{unit}</span>
      </div>
      <svg viewBox={`0 0 ${VIEW_WIDTH} ${VIEW_HEIGHT}`} className="lt-chart-svg" role="img" aria-label={title}>
        {[0.25, 0.5, 0.75].map((fraction) => (
          <line
            key={fraction}
            x1={0}
            x2={VIEW_WIDTH}
            y1={VIEW_HEIGHT * fraction}
            y2={VIEW_HEIGHT * fraction}
            className="lt-grid-line"
          />
        ))}
        {series.map((entry) => (
          <polyline
            key={entry.label}
            points={buildPoints(entry.values, bounds.min, bounds.max)}
            stroke={entry.color}
            className="lt-chart-line"
          />
        ))}
      </svg>
      <div className="lt-chart-legend">
        {series.map((entry) => (
          <span key={entry.label} className="lt-legend-item">
            <span className="lt-legend-dot" style={{ background: entry.color }} />
            {entry.label}
          </span>
        ))}
      </div>
    </div>
  );
};

const STATUS_BUCKETS: Array<{ label: string; color: string; matches: (code: number) => boolean }> = [
  { label: '2xx', color: '#22c55e', matches: (code) => code >= 200 && code < 300 },
  { label: '3xx', color: '#3b82f6', matches: (code) => code >= 300 && code < 400 },
  { label: '4xx', color: '#f59e0b', matches: (code) => code >= 400 && code < 500 },
  { label: '5xx', color: '#ef4444', matches: (code) => code >= 500 && code < 600 },
];

interface StatusDonutProps {
  statusCodes: Array<{ code: number; count: number }>;
}

export const StatusCodeDonut: React.FC<StatusDonutProps> = ({ statusCodes }) => {
  const buckets = STATUS_BUCKETS.map((bucket) => ({
    label: bucket.label,
    color: bucket.color,
    count: statusCodes
      .filter((entry) => bucket.matches(entry.code))
      .reduce((sum, entry) => sum + entry.count, 0),
  }));
  // Transport failures (status 0) and unmapped codes land in "Errors".
  buckets.push({
    label: 'Errors',
    color: '#71717a',
    count: statusCodes
      .filter((entry) => entry.code === 0 || !STATUS_BUCKETS.some((bucket) => bucket.matches(entry.code)))
      .reduce((sum, entry) => sum + entry.count, 0),
  });

  const total = buckets.reduce((sum, bucket) => sum + bucket.count, 0);
  const radius = 52;
  const circumference = 2 * Math.PI * radius;
  let offsetAccumulator = 0;

  return (
    <div className="lt-chart-card">
      <div className="lt-chart-header">
        <span className="lt-chart-title">Status Code Distribution</span>
        <span className="lt-chart-unit">{total} requests</span>
      </div>
      <div className="lt-donut-row">
        <svg viewBox="0 0 140 140" className="lt-donut-svg" role="img" aria-label="Status code distribution">
          <circle cx="70" cy="70" r={radius} fill="none" stroke="var(--bg-tertiary)" strokeWidth="16" />
          {total > 0 &&
            buckets
              .filter((bucket) => bucket.count > 0)
              .map((bucket) => {
                const dash = (bucket.count / total) * circumference;
                const circle = (
                  <circle
                    key={bucket.label}
                    cx="70"
                    cy="70"
                    r={radius}
                    fill="none"
                    stroke={bucket.color}
                    strokeWidth="16"
                    strokeDasharray={`${dash} ${circumference - dash}`}
                    strokeDashoffset={-offsetAccumulator}
                    transform="rotate(-90 70 70)"
                  />
                );
                offsetAccumulator += dash;
                return circle;
              })}
          <text x="70" y="76" textAnchor="middle" className="lt-donut-total">
            {total > 999 ? `${(total / 1000).toFixed(1)}k` : total}
          </text>
        </svg>
        <div className="lt-donut-legend">
          {buckets
            .filter((bucket) => bucket.count > 0)
            .map((bucket) => (
              <div key={bucket.label} className="lt-donut-legend-item">
                <span className="lt-legend-dot" style={{ background: bucket.color }} />
                <span>{bucket.label}</span>
                <strong>{bucket.count}</strong>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
};
