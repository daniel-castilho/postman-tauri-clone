import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Activity,
  ShieldCheck,
  ShieldAlert,
  Clock,
  Plus,
  Trash2,
  Power,
  PowerOff,
} from 'lucide-react';
import { toast } from 'sonner';
import type { MonitorDefinition, MonitorReport } from '../types/ipc';
import './MonitorsPanel.css';

export const MonitorsPanel: React.FC = () => {
  const [monitors, setMonitors] = useState<MonitorDefinition[]>([]);
  const [reports, setReports] = useState<Record<string, MonitorReport>>({});
  const [showAdd, setShowAdd] = useState(false);
  const [newMon, setNewMon] = useState({ name: '', url: '', interval: 60 });

  useEffect(() => {
    // Listens for status events from the backend
    const unlisten = listen<MonitorReport>('monitor-check', (event) => {
      setReports((prev) => ({
        ...prev,
        [event.payload.monitor_id]: event.payload,
      }));
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const addMonitor = async () => {
    if (!newMon.name || !newMon.url) return;

    const monitor: MonitorDefinition = {
      id: `mon_${Date.now()}`,
      name: newMon.name,
      url: newMon.url,
      interval_seconds: newMon.interval,
      enabled: true,
    };

    try {
      await invoke('start_monitor', { monitor });
      setMonitors([...monitors, monitor]);
      setShowAdd(false);
      setNewMon({ name: '', url: '', interval: 60 });
      toast.success('Monitor iniciado!');
    } catch {
      toast.error('Falha ao iniciar monitor');
    }
  };

  const toggleMonitor = async (m: MonitorDefinition) => {
    try {
      if (m.enabled) {
        await invoke('stop_monitor', { monitorId: m.id });
      } else {
        await invoke('start_monitor', { monitor: { ...m, enabled: true } });
      }
      setMonitors(
        monitors.map((mon) => (mon.id === m.id ? { ...mon, enabled: !mon.enabled } : mon)),
      );
    } catch {
      toast.error('Erro ao alterar estado do monitor');
    }
  };

  const removeMonitor = async (id: string) => {
    await invoke('stop_monitor', { monitorId: id });
    setMonitors(monitors.filter((m) => m.id !== id));
    const newReports = { ...reports };
    delete newReports[id];
    setReports(newReports);
  };

  return (
    <div className="monitors-container">
      <div className="monitors-header">
        <h2>Health Check Monitors</h2>
        <button className="add-mon-btn" onClick={() => setShowAdd(true)}>
          <Plus size={18} />
          Add Monitor
        </button>
      </div>

      {showAdd && (
        <div className="add-mon-form animate-fade-in">
          <input
            placeholder="Monitor Name"
            value={newMon.name}
            onChange={(e) => setNewMon({ ...newMon, name: e.target.value })}
          />
          <input
            placeholder="https://api.example.com/health"
            value={newMon.url}
            onChange={(e) => setNewMon({ ...newMon, url: e.target.value })}
          />
          <div className="interval-select">
            <label>Interval (s)</label>
            <input
              type="number"
              value={newMon.interval}
              onChange={(e) => setNewMon({ ...newMon, interval: parseInt(e.target.value) || 60 })}
            />
          </div>
          <div className="form-actions">
            <button className="cancel-btn" onClick={() => setShowAdd(false)}>
              Cancel
            </button>
            <button className="confirm-btn" onClick={addMonitor}>
              Start Protecting
            </button>
          </div>
        </div>
      )}

      <div className="monitors-grid">
        {monitors.map((m) => {
          const r = reports[m.id];
          return (
            <div
              key={m.id}
              className={`monitor-card ${r ? (r.is_healthy ? 'healthy' : 'unhealthy') : ''}`}
            >
              <div className="card-top">
                <div className="mon-info">
                  <span className="mon-name">{m.name}</span>
                  <span className="mon-url">{m.url}</span>
                </div>
                <div className="mon-actions">
                  <button onClick={() => toggleMonitor(m)} title={m.enabled ? 'Stop' : 'Start'}>
                    {m.enabled ? <Power size={18} className="active" /> : <PowerOff size={18} />}
                  </button>
                  <button onClick={() => removeMonitor(m.id)} title="Remove">
                    <Trash2 size={18} />
                  </button>
                </div>
              </div>

              {r ? (
                <div className="mon-status-area">
                  <div className="status-badge">
                    {r.is_healthy ? <ShieldCheck size={16} /> : <ShieldAlert size={16} />}
                    <span>{r.is_healthy ? 'Operational' : 'Down'}</span>
                  </div>
                  <div className="status-metrics">
                    <div className="metric">
                      <Clock size={14} />
                      {r.response_time_ms}ms
                    </div>
                    <div className="metric code">HTTP {r.status}</div>
                  </div>
                  <div className="last-check">
                    Last check: {new Date(r.last_check).toLocaleTimeString()}
                  </div>
                </div>
              ) : (
                <div className="mon-pending">
                  <Activity size={24} className="spin" />
                  <span>Aguardando primeiro check...</span>
                </div>
              )}
            </div>
          );
        })}

        {monitors.length === 0 && !showAdd && (
          <div className="monitors-empty">
            <ShieldCheck size={48} />
            <p>Seus serviços estão desprotegidos!</p>
            <span>
              Crie monitores para acompanhar o uptime e performance das suas APIs em tempo real.
            </span>
          </div>
        )}
      </div>
    </div>
  );
};
