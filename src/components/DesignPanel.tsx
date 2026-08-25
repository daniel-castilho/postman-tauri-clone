import React, { useCallback, useState } from 'react';
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

  const runLint = useCallback(async (content: string) => {
    try {
      const issues: LintIssue[] = await invoke('lint_spec', { content });
      setLintIssues(issues);
    } catch (err) {
      console.error('Lint failed:', err);
    }
  }, []);

  // Re-sync the local draft whenever the active design id or the store's
  // design list changes, using React's sanctioned adjust-state-during-render
  // pattern instead of a cascading effect.
  const [lastSync, setLastSync] = useState<{ id: string | null; designs: DesignSpec[] } | null>(
    null,
  );
  if (!lastSync || lastSync.id !== (activeDesignId ?? null) || lastSync.designs !== designs) {
    setLastSync({ id: activeDesignId ?? null, designs });
    const design = activeDesignId ? designs.find((d) => d.id === activeDesignId) : undefined;
    if (design) {
      setActiveSpec({ ...design });
      void runLint(design.content);
    } else {
      setActiveSpec(null);
    }
  }

  const handleSave = async () => {
    if (activeSpec) {
      await saveDesign(activeSpec);
      toast.success('Design saved successfully!');
    }
  };

  if (!activeDesignId && designs.length > 0) {
    return (
      <div className="design-empty-state">
        <Code size={48} />
        <h2>SpecHub: API Design Hub</h2>
        <p>Select a design from the sidebar or create a new one to get started.</p>
        <button className="primary-btn" onClick={() => setShowNewModal(true)}>
          <Plus size={16} /> New Design
        </button>
      </div>
    );
  }

  if (!activeSpec) return <div className="design-loading">No design selected.</div>;

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
            <Save size={16} /> Save
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
                <p>Your API follows all governance standards!</p>
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
