import React, { useState, useEffect, useRef } from 'react';
import { Search, Globe, Terminal, Play, Save, Settings, Hash, Folder, X } from 'lucide-react';
import { useWorkspaceStore } from '../store/workspaceStore';
import './CommandPalette.css';

interface CommandItem {
  id: string;
  name: string;
  type: 'Request' | 'Action' | 'Environment' | 'Collection';
  icon: React.ReactNode;
  shortcut?: string;
  action: () => void;
}

interface Props {
  onClose: () => void;
}

export const CommandPalette: React.FC<Props> = ({ onClose }) => {
  const [search, setSearch] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  
  const { 
    collections, environments, setActiveRequest, 
    setActiveEnvironment, updateRequest, activeRequest 
  } = useWorkspaceStore();

  // Build a dynamic item list based on the current state
  const getItems = (): CommandItem[] => {
    const items: CommandItem[] = [];

    // Quick actions
    items.push({ 
      id: 'act_send', name: 'Send Request', type: 'Action', 
      icon: <Play size={16} />, shortcut: 'Ctrl + Enter', 
      action: () => { window.dispatchEvent(new CustomEvent('trigger-send')); onClose(); } 
    });
    items.push({ 
      id: 'act_save', name: 'Save Request', type: 'Action', 
      icon: <Save size={16} />, shortcut: 'Ctrl + S', 
      action: () => { if (activeRequest) updateRequest(activeRequest); onClose(); } 
    });

    // Environments
    environments.forEach(env => {
      items.push({ 
        id: `env_${env.id}`, name: `Switch to: ${env.name}`, type: 'Environment', 
        icon: <Globe size={16} />, 
        action: () => { setActiveEnvironment(env.id); onClose(); } 
      });
    });

    // Collection requests (recursive)
    const addReqs = (collItems: any[]) => {
      collItems.forEach(item => {
        if (item.Request) {
          items.push({ 
            id: `req_${item.Request.id}`, name: item.Request.name, type: 'Request', 
            icon: <div className={`method-tag method-${item.Request.method}`}>{item.Request.method}</div>, 
            action: () => { setActiveRequest(item.Request); onClose(); } 
          });
        } else if (item.Folder) {
          addReqs(item.Folder.items);
        }
      });
    };
    collections.forEach(c => addReqs(c.items));

    return items.filter(item => 
      item.name.toLowerCase().includes(search.toLowerCase()) || 
      item.type.toLowerCase().includes(search.toLowerCase())
    );
  };

  const filteredItems = getItems();

  useEffect(() => {
    inputRef.current?.focus();
    const handleDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex(i => Math.min(i + 1, filteredItems.length - 1));
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex(i => Math.max(i - 1, 0));
      } else if (e.key === 'Enter') {
        if (filteredItems[selectedIndex]) filteredItems[selectedIndex].action();
      } else if (e.key === 'Escape') {
        onClose();
      }
    };
    window.addEventListener('keydown', handleDown);
    return () => window.removeEventListener('keydown', handleDown);
  }, [filteredItems, selectedIndex]);

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={e => e.stopPropagation()}>
        <div className="cp-search-bar">
          <Search size={20} className="cp-search-icon" />
          <input 
            ref={inputRef}
            placeholder="Type a command or search requests..." 
            value={search}
            onChange={e => { setSearch(e.target.value); setSelectedIndex(0); }}
          />
          <div className="cp-esc-tag">ESC</div>
        </div>

        <div className="cp-results">
            {filteredItems.length === 0 ? (
              <div className="cp-empty">No results found for "{search}"</div>
            ) : (
              filteredItems.map((item, idx) => (
                <div 
                  key={item.id} 
                  className={`cp-item ${idx === selectedIndex ? 'active' : ''}`}
                  onClick={() => item.action()}
                  onMouseEnter={() => setSelectedIndex(idx)}
                >
                  <div className="cp-item-left">
                    <span className="cp-item-icon">{item.icon}</span>
                    <span className="cp-item-name">{item.name}</span>
                    <span className="cp-item-type">{item.type}</span>
                  </div>
                  {item.shortcut && (
                    <div className="cp-item-shortcut">{item.shortcut}</div>
                  )}
                </div>
              ))
            )}
        </div>

        <div className="cp-footer">
          <span><b>↑↓</b> to navigate</span>
          <span><b>ENTER</b> to select</span>
          <span><b>ESC</b> to close</span>
        </div>
      </div>
    </div>
  );
};
