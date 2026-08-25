import React, { useState } from 'react';
import { useWorkspaceStore } from '../store/workspaceStore';
import { invoke } from '@tauri-apps/api/core';
import { Play, Activity, Clock, Zap, AlertCircle, CheckCircle2 } from 'lucide-react';
import { toast } from 'sonner';
import type { LoadTestReport } from '../types/ipc';
import './LoadTestingPanel.css';

export const LoadTestingPanel: React.FC = () => {
  const { activeRequest, environments, activeEnvironmentId, globals } = useWorkspaceStore();
  const [loading, setLoading] = useState(false);
  const [config, setConfig] = useState({
    users: 5,
    requestsPerUser: 10,
    delayMs: 0
  });
  const [report, setReport] = useState<LoadTestReport | null>(null);

  const activeEnv = environments?.find((e: any) => e.id === activeEnvironmentId) || { id: "env_default", name: "No Environment", variables: [] };

  const startTest = async () => {
    if (!activeRequest) return;
    setLoading(true);
    setReport(null);
    
    try {
      // Simplifying the request sent to the load test
      // In a real scenario we might want to send the full body etc.
      const result = await invoke<LoadTestReport>("run_load_test", {
        request: activeRequest,
        config: {
          users: config.users,
          requests_per_user: config.requestsPerUser,
          delay_ms: config.delayMs
        },
        environment: activeEnv,
        globals: globals
      });
      
      setReport(result);
      toast.success("Load test concluído!");
    } catch (error: any) {
      toast.error("Falha no load test: " + error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="load-test-container">
      <div className="load-test-config">
        <div className="config-group">
          <label>Concurrent Users</label>
          <input 
            type="number" 
            value={config.users} 
            onChange={e => setConfig({...config, users: parseInt(e.target.value) || 1})}
            min="1"
            max="100"
          />
        </div>
        <div className="config-group">
          <label>Requests per User</label>
          <input 
            type="number" 
            value={config.requestsPerUser} 
            onChange={e => setConfig({...config, requestsPerUser: parseInt(e.target.value) || 1})}
            min="1"
          />
        </div>
        <div className="config-group">
          <label>Delay (ms)</label>
          <input 
            type="number" 
            value={config.delayMs} 
            onChange={e => setConfig({...config, delayMs: parseInt(e.target.value) || 0})}
            min="0"
          />
        </div>
        <button 
          className={`run-test-btn ${loading ? 'loading' : ''}`} 
          onClick={startTest} 
          disabled={loading || !activeRequest}
        >
          {loading ? <Activity className="spin" size={18} /> : <Play size={18} />}
          <span>{loading ? "Running..." : "Start Load Test"}</span>
        </button>
      </div>

      {report && (
        <div className="load-test-results animate-fade-in">
          <div className="results-grid">
            <div className="result-card">
              <div className="card-icon success"><CheckCircle2 size={20} /></div>
              <div className="card-content">
                <span className="label">Throughput</span>
                <span className="value">{report.requestsPerSecond.toFixed(2)} req/s</span>
              </div>
            </div>
            <div className="result-card">
              <div className="card-icon info"><Clock size={20} /></div>
              <div className="card-content">
                <span className="label">Avg Latency</span>
                <span className="value">{report.avgTimeMs.toFixed(2)} ms</span>
              </div>
            </div>
            <div className="result-card">
              <div className="card-icon warning"><Zap size={20} /></div>
              <div className="card-content">
                <span className="label">P95 Latency</span>
                <span className="value">{report.p95TimeMs} ms</span>
              </div>
            </div>
          </div>

          <div className="stats-summary">
            <div className="stat-row">
              <span className="stat-label">Total Requests</span>
              <span className="stat-value">{report.totalRequests}</span>
            </div>
            <div className="stat-row">
              <span className="stat-label">Success Rate</span>
              <span className="stat-value success">
                {((report.successCount / report.totalRequests) * 100).toFixed(1)}%
              </span>
            </div>
            <div className="stat-row">
              <span className="stat-label">Min / Max</span>
              <span className="stat-value">{report.minTimeMs}ms / {report.maxTimeMs}ms</span>
            </div>
          </div>

          <div className="latency-bar-container">
            <div className="latency-label">Latency Breakdown</div>
            <div className="latency-bar">
                <div 
                    className="latency-fill" 
                    style={{ width: `${(report.avgTimeMs / report.maxTimeMs) * 100}%` }}
                >
                    <span className="fill-label">Avg: {report.avgTimeMs.toFixed(0)}ms</span>
                </div>
            </div>
          </div>
        </div>
      )}

      {loading && !report && (
        <div className="load-test-placeholder">
          <Activity size={48} className="spin-slow" />
          <p>Assaltando o servidor com requisições paralelas...</p>
        </div>
      )}
      
      {!loading && !report && (
        <div className="load-test-placeholder-empty">
          <Zap size={48} />
          <p>Pronto para testar os limites da sua API?</p>
          <span>Configure o número de usuários e dispare o teste de carga em massa.</span>
        </div>
      )}
    </div>
  );
};
