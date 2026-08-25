import { FolderOpen } from 'lucide-react';
import { useWorkspaceStore } from '../store/workspaceStore';
import './WorkspaceSelector.css';

export function WorkspaceSelector() {
  const { openWorkspace, error } = useWorkspaceStore();

  return (
    <div className="workspace-selector-container">
      <div className="workspace-selector-card">
        <h2>Bem-vindo ao Tyny Pulse</h2>
        <p>Para começar, abra uma pasta para usar como Workspace.</p>
        <button className="open-workspace-btn" onClick={openWorkspace}>
          <FolderOpen size={20} />
          <span>Abrir Workspace</span>
        </button>
        {error && (
          <div style={{ marginTop: '16px', color: '#fca5a5', fontSize: '12px', background: '#3f1515', padding: '8px', borderRadius: '4px' }}>
            Erro: {error}
          </div>
        )}
      </div>
    </div>
  );
}
