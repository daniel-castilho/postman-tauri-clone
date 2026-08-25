import React from 'react';
import { useWorkspaceStore } from '../store/workspaceStore';
import { X, Plus, Trash2 } from 'lucide-react';
import './GlobalsModal.css';

interface GlobalsModalProps {
  onClose: () => void;
}

export const GlobalsModal: React.FC<GlobalsModalProps> = ({ onClose }) => {
  const { globals, saveGlobals } = useWorkspaceStore();

  const handleUpdate = (key: string, value: string) => {
    const newVars = { ...globals.variables, [key]: value };
    saveGlobals({ variables: newVars });
  };

  const handleRename = (oldKey: string, newKey: string) => {
    if (oldKey === newKey) return;
    const newVars = { ...globals.variables };
    const value = newVars[oldKey];
    delete newVars[oldKey];
    newVars[newKey] = value;
    saveGlobals({ variables: newVars });
  };

  const handleDelete = (key: string) => {
    const newVars = { ...globals.variables };
    delete newVars[key];
    saveGlobals({ variables: newVars });
  };

  const handleAdd = () => {
    const newKey = `variable_${Date.now()}`;
    const newVars = { ...globals.variables, [newKey]: "" };
    saveGlobals({ variables: newVars });
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content globals-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div className="header-title-group">
            <h2 className="premium-title">Global Variables</h2>
            <p className="subtitle">Variables accessible across all environments in this workspace.</p>
          </div>
          <button onClick={onClose} className="close-modal-btn">
            <X size={20} />
          </button>
        </div>
        
        <div className="modal-body">
          <div className="vars-manager">
            <div className="vars-table-header">
              <div className="col-key">Variable</div>
              <div className="col-val">Value</div>
              <div className="col-actions"></div>
            </div>
            
            <div className="vars-list scrollable">
              {Object.entries(globals.variables).length === 0 ? (
                <div className="empty-vars">
                  <p>No global variables defined.</p>
                </div>
              ) : (
                Object.entries(globals.variables).map(([key, val]) => (
                  <div key={key} className="var-row">
                    <div className="col-key">
                      <input 
                        type="text" 
                        defaultValue={key}
                        onBlur={(e) => handleRename(key, e.target.value)}
                        placeholder="Key"
                        className="var-input"
                      />
                    </div>
                    <div className="col-val">
                      <input 
                        type="text" 
                        value={val}
                        onChange={(e) => handleUpdate(key, e.target.value)}
                        placeholder="Value"
                        className="var-input"
                      />
                    </div>
                    <div className="col-actions">
                      <button className="delete-var-btn" onClick={() => handleDelete(key)}>
                        <Trash2 size={14} />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
            
            <button className="add-var-bubble-btn" onClick={handleAdd}>
              <Plus size={16} /> Add Variable
            </button>
          </div>
        </div>
        
        <div className="modal-footer">
          <p className="hint-text">Global variables are persisted in <code>globals.json</code></p>
          <button className="confirm-btn" onClick={onClose}>Done</button>
        </div>
      </div>
    </div>
  );
};
