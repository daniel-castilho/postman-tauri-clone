import { FolderOpen } from 'lucide-react';
import { useWorkspaceStore } from '../store/workspaceStore';
import './WorkspaceSelector.css';

export function WorkspaceSelector() {
  const { openWorkspace, error } = useWorkspaceStore();

  return (
    <div className="workspace-selector-container">
      <div className="workspace-selector-card">
        <h2>Welcome to Tyny Pulse</h2>
        <p>To get started, open a folder to use as your workspace.</p>
        <button className="open-workspace-btn" onClick={openWorkspace}>
          <FolderOpen size={20} />
          <span>Open Workspace</span>
        </button>
        {error && (
          <div
            style={{
              marginTop: '16px',
              color: '#fca5a5',
              fontSize: '12px',
              background: '#3f1515',
              padding: '8px',
              borderRadius: '4px',
            }}
          >
            Error: {error}
          </div>
        )}
      </div>
    </div>
  );
}
