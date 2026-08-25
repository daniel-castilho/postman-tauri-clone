import React, { useState, useEffect } from 'react';
import { useWorkspaceStore, DesignSpec } from '../store/workspaceStore';
import type { LintIssue } from '../types/ipc';
import { Save, AlertTriangle, CheckCircle, Info, Trash2, Plus, Code } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import './DesignPanel.css';

export const DesignPanel: React.FC = () => {
  const { designs, activeDesignId, saveDesign, deleteDesign } = useWorkspaceStore();

  const [activeSpec, setActiveSpec] = useState<DesignSpec | null>(null);
  const [lintIssues, setLintIssues] = useState<LintIssue[]>([]);
  const [, setShowNewModal] = useState(false);

  useEffect(() => {
    if (activeDesignId) {
      const design = designs.find((d) => d.id === activeDesignId);
      if (design) {
        setActiveSpec({ ...design });
        runLint(design.content);
      }
    } else {
      setActiveSpec(null);
    }
  }, [activeDesignId, designs]);

  const runLint = async (content: string) => {
    try {
      const issues: LintIssue[] = await invoke('lint_spec', { content });
      setLintIssues(issues);
    } catch (err) {
      console.error('Lint failed:', err);
    }
  };

  const handleSave = async () => {
    if (activeSpec) {
      await saveDesign(activeSpec);
      toast.success('Design salvo com sucesso!');
    }
  };

  if (!activeDesignId && designs.length > 0) {
    return (
      <div className="design-empty-state">
        <Code size={48} />
        <h2>SpecHub: API Design Hub</h2>
        <p>Selecione um design na barra lateral ou crie um novo para começar.</p>
        <button className="primary-btn" onClick={() => setShowNewModal(true)}>
          <Plus size={16} /> Novo Design
        </button>
      </div>
    );
  }

  if (!activeSpec) return <div className="design-loading">Nenhum design selecionado.</div>;

  return (
    <div className="design-container">
      <div className="design-header">
        <div className="design-info">
          <Code size={18} className="design-icon" />
          <input
            className="design-name-input"
            value={activeSpec.name}
            onChange={(e) => setActiveSpec({ ...activeSpec, name: e.target.value })}
          />
          <span className="design-version-badge">{activeSpec.version}</span>
        </div>
        <div className="design-actions">
          <button className="secondary-btn" onClick={handleSave}>
            <Save size={16} /> Salvar
          </button>
          <button className="danger-btn-ghost" onClick={() => deleteDesign(activeSpec.id)}>
            <Trash2 size={16} />
          </button>
        </div>
      </div>

      <div className="design-content">
        <div className="design-editor-wrapper">
          <textarea
            className="spec-editor"
            placeholder="Paste your OpenAPI YAML here..."
            value={activeSpec.content}
            onChange={(e) => {
              setActiveSpec({ ...activeSpec, content: e.target.value });
              runLint(e.target.value);
            }}
          />
        </div>

        <div className="lint-panel">
          <div className="lint-header">
            <h3>Governance & Rules</h3>
            <div className="lint-stats">
              <span className="error">
                <AlertTriangle size={12} />{' '}
                {lintIssues.filter((i) => i.severity === 'Error').length}
              </span>
              <span className="warning">
                <Info size={12} /> {lintIssues.filter((i) => i.severity === 'Warning').length}
              </span>
            </div>
          </div>
          <div className="lint-list">
            {lintIssues.length === 0 ? (
              <div className="lint-perfect">
                <CheckCircle size={32} />
                <p>Sua API segue todos os padrões de Governança!</p>
              </div>
            ) : (
              lintIssues.map((issue, idx) => (
                <div key={idx} className={`lint-item ${issue.severity.toLowerCase()}`}>
                  <div className="lint-item-header">
                    <span className="lint-severity">{issue.severity}</span>
                    <span className="lint-path">{issue.path}</span>
                  </div>
                  <p className="lint-message">{issue.message}</p>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
