import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, Square, Plus, Trash2, Globe } from 'lucide-react';
import { toast } from 'sonner';
import type { HttpMethod, MockRule, MockServerStatus } from '../types/ipc';

export const MockServer: React.FC = () => {
  const [rules, setRules] = useState<MockRule[]>([
    {
      id: '1',
      path: '/api/hello',
      method: 'GET',
      status: 200,
      headers: [],
      body: '{"message": "Hello from Mock!"}',
    },
  ]);
  const [status, setStatus] = useState<MockServerStatus>({
    is_running: false,
    port: 3000,
    active_rules: 0,
  });
  const [port, setPort] = useState(3000);

  const fetchStatus = async () => {
    try {
      const s = await invoke<MockServerStatus>('get_mock_server_status');
      setStatus(s);
    } catch {
      // Best effort: failures here must not break the surrounding flow.
    }
  };

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleStart = async () => {
    try {
      await invoke('start_mock_server', { port, rules });
      toast.success('Mock Server iniciado!');
      fetchStatus();
    } catch (e) {
      toast.error('Erro ao iniciar server', { description: String(e) });
    }
  };

  const handleStop = async () => {
    try {
      await invoke('stop_mock_server');
      toast.success('Mock Server parado.');
      fetchStatus();
    } catch {
      // Best effort: failures here must not break the surrounding flow.
    }
  };

  const addRule = () => {
    setRules([
      ...rules,
      {
        id: Math.random().toString(36).substr(2, 9),
        path: '/',
        method: 'GET',
        status: 200,
        headers: [],
        body: '',
      },
    ]);
  };

  const removeRule = (id: string) => {
    setRules(rules.filter((r) => r.id !== id));
  };

  return (
    <div className="mock-server-view">
      <div className="mock-controls">
        <div className="mock-header-row">
          <div className="mock-status-pill">
            <div className={`status-dot ${status.is_running ? 'active' : ''}`} />
            {status.is_running ? 'Running' : 'Stopped'}
          </div>
          <div className="mock-actions-row">
            <div className="port-input-group">
              <Globe size={14} />
              <span>localhost:</span>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(parseInt(e.target.value))}
                className="port-input"
                disabled={status.is_running}
              />
            </div>
            {status.is_running ? (
              <button className="mock-btn stop" onClick={handleStop}>
                <Square size={16} /> Stop Server
              </button>
            ) : (
              <button className="mock-btn start" onClick={handleStart}>
                <Play size={16} /> Start Server
              </button>
            )}
          </div>
        </div>
      </div>

      <div className="mock-rules-container">
        <div className="mock-rules-header">
          <h3>Ações / Regras ({rules.length})</h3>
          <button className="add-rule-btn" onClick={addRule}>
            <Plus size={16} /> Add Rule
          </button>
        </div>

        <div className="mock-rules-list">
          {rules.length === 0 && <div className="empty-mock">Nenhuma regra definida.</div>}
          {rules.map((rule, idx) => (
            <div key={rule.id} className="mock-rule-card">
              <div className="rule-top">
                <select
                  value={rule.method as string}
                  onChange={(e) => {
                    const newRules = [...rules];
                    newRules[idx].method = e.target.value as HttpMethod;
                    setRules(newRules);
                  }}
                  className="rule-method"
                >
                  <option>GET</option>
                  <option>POST</option>
                  <option>PUT</option>
                  <option>DELETE</option>
                </select>
                <input
                  type="text"
                  value={rule.path}
                  onChange={(e) => {
                    const newRules = [...rules];
                    newRules[idx].path = e.target.value;
                    setRules(newRules);
                  }}
                  className="rule-path"
                  placeholder="/api/v1/resource"
                />
                <input
                  type="number"
                  value={rule.status}
                  onChange={(e) => {
                    const newRules = [...rules];
                    newRules[idx].status = parseInt(e.target.value);
                    setRules(newRules);
                  }}
                  className="rule-status"
                  placeholder="200"
                />
                <button className="rule-delete" onClick={() => removeRule(rule.id)}>
                  <Trash2 size={16} />
                </button>
              </div>
              <textarea
                className="rule-body"
                value={rule.body}
                onChange={(e) => {
                  const newRules = [...rules];
                  newRules[idx].body = e.target.value;
                  setRules(newRules);
                }}
                placeholder="Response Body (JSON, Text, etc)"
                spellCheck={false}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
