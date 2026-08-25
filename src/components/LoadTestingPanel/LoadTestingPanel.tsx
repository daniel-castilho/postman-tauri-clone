import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Activity, Play, Square, Gauge, Zap, CheckCircle2, XCircle, Timer } from 'lucide-react';
import { toast } from 'sonner';
import { useWorkspaceStore } from '../../store/workspaceStore';
import type {
  CollectionItem,
  HttpRequest,
  LoadTestConfigDto,
  LoadTestProgressEventDto,
} from '../../types/ipc';
import { MultiLineChart, StatusCodeDonut } from './LoadTestCharts';
import type { LoadTestChartPoint } from './LoadTestCharts';
import { LoadTestReport } from './LoadTestReport';
import './LoadTestingPanel.css';

type PanelPhase = 'idle' | 'running' | 'finished';

const MAX_CHART_POINTS = 240;

function flattenRequests(items: CollectionItem[]): HttpRequest[] {
  return items.flatMap((item) =>
    'Request' in item
      ? [item.Request]
      : flattenRequests(item.Folder.items),
  );
}

function formatMethod(method: HttpRequest['method']): string {
  return typeof method === 'string' ? method : `CUSTOM(${method.CUSTOM})`;
}

export const LoadTestingPanel: React.FC = () => {
  const { activeRequest, collections, environments, activeEnvironmentId, globals } =
    useWorkspaceStore();

  const workspaceRequests = useMemo(
    () => collections.flatMap((collection) => flattenRequests(collection.items)),
    [collections],
  );
  const requestOptions = useMemo(() => {
    if (activeRequest && !workspaceRequests.some((request) => request.id === activeRequest.id)) {
      return [activeRequest, ...workspaceRequests];
    }
    return workspaceRequests;
  }, [activeRequest, workspaceRequests]);

  const [selectedRequestId, setSelectedRequestId] = useState<string>('');
  const selectedRequest = useMemo(
    () =>
      requestOptions.find((request) => request.id === selectedRequestId) ??
      activeRequest ??
      requestOptions[0] ??
      null,
    [requestOptions, selectedRequestId, activeRequest],
  );

  const [config, setConfig] = useState({
    virtualUsers: 50,
    durationSeconds: 10,
    rampUpSeconds: 2,
    timeoutMs: 5000,
  });
  const [phase, setPhase] = useState<PanelPhase>('idle');
  const [snapshot, setSnapshot] = useState<LoadTestProgressEventDto | null>(null);
  const [chartPoints, setChartPoints] = useState<LoadTestChartPoint[]>([]);
  const [runningConfig, setRunningConfig] = useState<LoadTestConfigDto | null>(null);

  const activeEnvironment =
    environments?.find((environment) => environment.id === activeEnvironmentId) ?? {
      id: 'env_default',
      name: 'No Environment',
      variables: [],
    };

  const handleProgressEvent = useCallback((event: LoadTestProgressEventDto) => {
    setSnapshot(event);
    setChartPoints((previous) => {
      const next = [
        ...previous,
        {
          elapsedSeconds: event.elapsedSeconds,
          currentRps: event.currentRps,
          percentiles: event.percentiles,
        },
      ];
      return next.length > MAX_CHART_POINTS ? next.slice(-MAX_CHART_POINTS) : next;
    });
    if (event.isFinished) {
      setPhase('finished');
    }
  }, []);

  // Recover the status of a possibly running test when the panel mounts.
  useEffect(() => {
    let cancelled = false;
    invoke<LoadTestProgressEventDto | null>('get_load_test_status')
      .then((status) => {
        if (!cancelled && status) {
          handleProgressEvent(status);
          setPhase(status.isFinished ? 'finished' : 'running');
        }
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [handleProgressEvent]);

  useEffect(() => {
    const unlistenPromise = listen<LoadTestProgressEventDto>(
      'load_test_progress',
      (event) => handleProgressEvent(event.payload),
    );
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [handleProgressEvent]);

  const startTest = async (): Promise<void> => {
    if (!selectedRequest) {
      toast.error('Select a target request first');
      return;
    }
    const loadTestConfig: LoadTestConfigDto = {
      targetRequest: selectedRequest,
      virtualUsers: config.virtualUsers,
      durationSeconds: config.durationSeconds,
      rampUpSeconds: config.rampUpSeconds,
      timeoutMs: config.timeoutMs,
    };
    setRunningConfig(loadTestConfig);
    setChartPoints([]);
    setSnapshot(null);
    setPhase('running');

    try {
      await invoke<string>('start_load_test', {
        config: loadTestConfig,
        environment: activeEnvironment,
        globals: globals ?? { variables: {} },
      });
      toast.success(`Load test started with ${config.virtualUsers} VUs`);
    } catch (error) {
      setPhase('idle');
      toast.error(String(error));
    }
  };

  const stopTest = async (): Promise<void> => {
    try {
      const finalSnapshot = await invoke<LoadTestProgressEventDto | null>('stop_load_test');
      if (finalSnapshot) {
        handleProgressEvent(finalSnapshot);
      } else {
        setPhase('finished');
      }
      toast.info('Stop signal sent — workers halted');
    } catch (error) {
      toast.error(String(error));
    }
  };

  const isRunning = phase === 'running';
  const percentiles = snapshot?.percentiles;

  return (
    <div className="lt-panel">
      <div className="lt-config-card">
        <div className="config-group lt-request-picker">
          <label>Target Request</label>
          <select
            value={selectedRequest?.id ?? ''}
            onChange={(event) => setSelectedRequestId(event.target.value)}
            disabled={isRunning}
          >
            {requestOptions.length === 0 && <option value="">No requests in workspace</option>}
            {requestOptions.map((request) => (
              <option key={request.id} value={request.id}>
                [{formatMethod(request.method)}] {request.name}
              </option>
            ))}
          </select>
        </div>
        <div className="config-group lt-vu-slider">
          <label>
            Virtual Users: <strong>{config.virtualUsers}</strong>
          </label>
          <input
            type="range"
            min={1}
            max={500}
            value={config.virtualUsers}
            disabled={isRunning}
            onChange={(event) =>
              setConfig({ ...config, virtualUsers: Number(event.target.value) || 1 })
            }
          />
        </div>
        <div className="config-group">
          <label>Duration (s)</label>
          <input
            type="number"
            min={1}
            max={3600}
            value={config.durationSeconds}
            disabled={isRunning}
            onChange={(event) =>
              setConfig({ ...config, durationSeconds: Number(event.target.value) || 1 })
            }
          />
        </div>
        <div className="config-group">
          <label>Ramp-up (s)</label>
          <input
            type="number"
            min={0}
            max={600}
            value={config.rampUpSeconds}
            disabled={isRunning}
            onChange={(event) =>
              setConfig({ ...config, rampUpSeconds: Number(event.target.value) || 0 })
            }
          />
        </div>
        <div className="config-group">
          <label>Timeout (ms)</label>
          <input
            type="number"
            min={1}
            max={60000}
            value={config.timeoutMs}
            disabled={isRunning}
            onChange={(event) =>
              setConfig({ ...config, timeoutMs: Number(event.target.value) || 1 })
            }
          />
        </div>
        <div className="lt-controls">
          <button
            className={`run-test-btn ${isRunning ? 'loading' : ''}`}
            onClick={() => void startTest()}
            disabled={isRunning || !selectedRequest}
          >
            {isRunning ? <Activity className="spin" size={18} /> : <Play size={18} />}
            <span>{isRunning ? 'Running…' : 'Start Load Test'}</span>
          </button>
          <button className="stop-test-btn" onClick={() => void stopTest()} disabled={!isRunning}>
            <Square size={16} />
            Stop
          </button>
          <span className={`lt-status-badge ${phase}`}>{phase.toUpperCase()}</span>
        </div>
      </div>

      {snapshot && (
        <div className="results-grid animate-fade-in">
          <MetricCard
            icon={<Gauge size={20} />}
            tone="info"
            label="Current RPS"
            value={`${snapshot.currentRps.toFixed(1)} req/s`}
          />
          <MetricCard
            icon={<Zap size={20} />}
            tone="warning"
            label="p95 Latency"
            value={`${(percentiles?.p95Ms ?? 0).toFixed(0)} ms`}
          />
          <MetricCard
            icon={<Timer size={20} />}
            tone="neutral"
            label="Active VUs"
            value={String(snapshot.activeVus)}
          />
          <MetricCard
            icon={<CheckCircle2 size={20} />}
            tone="success"
            label="Successful"
            value={String(snapshot.successfulRequests)}
          />
          <MetricCard
            icon={<XCircle size={20} />}
            tone="danger"
            label="Failed"
            value={String(snapshot.failedRequests)}
          />
          <MetricCard
            icon={<Activity size={20} />}
            tone="neutral"
            label="Throughput"
            value={`${(snapshot.bytesPerSecond / 1024).toFixed(1)} KiB/s`}
          />
        </div>
      )}

      <LoadTestReport config={runningConfig} snapshot={snapshot} />

      {chartPoints.length > 1 && (
        <div className="lt-charts-grid animate-fade-in">
          <MultiLineChart
            title="Requests per Second"
            unit="req/s over time"
            series={[
              { label: 'RPS', color: '#8b5cf6', values: chartPoints.map((point) => point.currentRps) },
            ]}
          />
          <MultiLineChart
            title="Latency Percentiles"
            unit="milliseconds over time"
            series={[
              { label: 'p50', color: '#22c55e', values: chartPoints.map((point) => point.percentiles.p50Ms) },
              { label: 'p95', color: '#f59e0b', values: chartPoints.map((point) => point.percentiles.p95Ms) },
              { label: 'p99', color: '#ef4444', values: chartPoints.map((point) => point.percentiles.p99Ms) },
            ]}
          />
          <StatusCodeDonut statusCodes={snapshot?.statusCodes ?? []} />
        </div>
      )}

      {!isRunning && !snapshot && (
        <div className="load-test-placeholder-empty">
          <Zap size={48} />
          <p>Ready to stress your API?</p>
          <span>Pick a target request, dial in the concurrency and start the engine.</span>
        </div>
      )}
    </div>
  );
};

interface MetricCardProps {
  icon: React.ReactNode;
  tone: 'success' | 'info' | 'warning' | 'danger' | 'neutral';
  label: string;
  value: string;
}

const MetricCard: React.FC<MetricCardProps> = ({ icon, tone, label, value }) => (
  <div className="result-card">
    <div className={`card-icon ${tone === 'neutral' ? '' : tone}`}>{icon}</div>
    <div className="card-content">
      <span className="label">{label}</span>
      <span className="value">{value}</span>
    </div>
  </div>
);
