import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, X, CheckCircle2, XCircle, Loader2, BarChart3 } from "lucide-react";
import "./CollectionRunner.css";

interface CollectionRunnerProps {
  items: any[];
  environment: any;
  onClose: () => void;
}

export function CollectionRunner({ items, environment, onClose }: CollectionRunnerProps) {
  const [running, setRunning] = useState(false);
  const [report, setReport] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);

  const handleRun = async () => {
    setRunning(true);
    setReport(null);
    setError(null);
    try {
      const result = await invoke<any>("run_collection", {
        items,
        environment: environment || { id: "default", name: "No Env", variables: {} }
      });
      setReport(result);
    } catch (e: any) {
      setError(typeof e === 'string' ? e : JSON.stringify(e));
    } finally {
      setRunning(false);
    }
  };

  return (
    <div className="runner-overlay">
      <div className="runner-container">
        <div className="runner-header">
          <div className="runner-title">
            <BarChart3 size={20} className="runner-icon" />
            <h2>Collection Runner</h2>
          </div>
          <button onClick={onClose} className="close-runner-btn">
            <X size={20} />
          </button>
        </div>

        <div className="runner-content">
          {!report && !running && (
            <div className="runner-prepare">
              <p>Você está prestes a executar <strong>{items.length} itens</strong> sequencialmente.</p>
              <div className="runner-env-hint">
                Ambiente Ativo: <span>{environment?.name || "Nenhum"}</span>
              </div>
              <button onClick={handleRun} className="start-run-btn">
                <Play size={16} fill="currentColor" />
                Iniciar Execução
              </button>
            </div>
          )}

          {running && (
            <div className="runner-progress">
              <Loader2 size={40} className="spinner-icon" />
              <p>Executando testes automatizados...</p>
            </div>
          )}

          {error && <div className="runner-error">{error}</div>}

          {report && (
            <div className="runner-report">
              <div className="report-summary">
                <div className="summary-card">
                  <span className="summary-label">Requests</span>
                  <span className="summary-value">{report.totalRequests}</span>
                </div>
                <div className="summary-card">
                  <span className="summary-label">Tests</span>
                  <span className="summary-value">{report.totalTests}</span>
                </div>
                <div className="summary-card passed">
                  <span className="summary-label">Passed</span>
                  <span className="summary-value">{report.passedTests}</span>
                </div>
                <div className="summary-card failed">
                  <span className="summary-label">Failed</span>
                  <span className="summary-value">{report.totalTests - report.passedTests}</span>
                </div>
              </div>

              <div className="report-details">
                {report.results.map((res: any, i: number) => (
                  <div key={i} className="report-item">
                    <div className="report-item-header">
                      <span className="report-item-name">{res.requestName}</span>
                      <div className="report-item-meta">
                        <span className={`status-badge ${res.status < 300 ? 'ok' : 'err'}`}>{res.status}</span>
                        <span className="time-badge">{res.timeMs}ms</span>
                      </div>
                    </div>
                    <div className="report-item-tests">
                      {res.tests.map((t: any, ti: number) => (
                        <div key={ti} className={`test-row ${t.passed ? 'passed' : 'failed'}`}>
                          {t.passed ? <CheckCircle2 size={12} /> : <XCircle size={12} />}
                          <span>{t.name}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
              
              <button onClick={onClose} className="finish-run-btn">Fechar Relatório</button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
