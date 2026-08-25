import { X } from 'lucide-react';
import { useWorkspaceStore } from '../store/workspaceStore';
import './TabBar.css';

export function TabBar() {
  const { openTabs, activeTabId, setActiveRequestTab, closeTab } = useWorkspaceStore();

  if (openTabs.length === 0) return null;

  return (
    <div className="tab-bar">
      {openTabs.map((tab) => (
        <div
          key={tab.id}
          className={`tab-item ${activeTabId === tab.id ? 'active' : ''}`}
          onClick={() => setActiveRequestTab(tab.id)}
        >
          <span className="tab-method" data-method={String(tab.method)}>
            {String(tab.method)}
          </span>
          <span className="tab-name" title={tab.url}>
            {tab.url || 'New Request'}
          </span>
          <button
            className="close-tab-btn"
            onClick={(e) => {
              e.stopPropagation();
              closeTab(tab.id);
            }}
          >
            <X size={12} />
          </button>
        </div>
      ))}
    </div>
  );
}
