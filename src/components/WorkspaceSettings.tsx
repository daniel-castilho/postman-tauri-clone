import React, { useCallback, useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Users,
  UserPlus,
  Shield,
  User,
  Mail,
  X,
  CheckCircle2,
  Package,
  ToggleLeft,
  ToggleRight,
} from 'lucide-react';
import { toast } from 'sonner';
import type { MemberRole, WorkspaceMember, ScriptLibraryInfo } from '../types/ipc';
import { useWorkspaceStore } from '../store/workspaceStore';
import './WorkspaceSettings.css';

interface Props {
  onClose: () => void;
}

export const WorkspaceSettings: React.FC<Props> = ({ onClose }) => {
  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState<MemberRole>('Viewer');
  const [loading, setLoading] = useState(false);
  const [libraries, setLibraries] = useState<ScriptLibraryInfo[]>([]);
  const workspacePath = useWorkspaceStore(
    (state: { workspacePath: string }) => state.workspacePath,
  );

  const loadMembers = useCallback(async () => {
    try {
      const list = await invoke<WorkspaceMember[]>('get_members');
      setMembers(list);
    } catch {
      toast.error('Failed to load members');
    }
  }, []);

  // Initial load mirrors the guarded inline pattern used by the other
  // panels (Git panel / mock server): state is only committed while the
  // component is still mounted.
  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const list = await invoke<WorkspaceMember[]>('get_members');
        if (!cancelled) setMembers(list);
      } catch {
        toast.error('Failed to load members');
      }
    })();
    void (async () => {
      if (!workspacePath || cancelled) return;
      try {
        await invoke('configure_script_engine', { workspacePath });
        const libs = await invoke<ScriptLibraryInfo[]>('list_script_libraries', { workspacePath });
        if (!cancelled) setLibraries(libs);
      } catch {
        toast.error('Failed to load script libraries');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [workspacePath]);

  const toggleLibrary = async (library: ScriptLibraryInfo) => {
    if (!workspacePath) return;
    try {
      setLibraries(
        await invoke<ScriptLibraryInfo[]>('set_script_library_enabled', {
          workspacePath,
          name: library.name,
          enabled: !library.enabled,
        }),
      );
    } catch {
      toast.error(`Failed to update ${library.name}`);
    }
  };

  const handleInvite = async () => {
    if (!inviteEmail) return;
    setLoading(true);
    try {
      await invoke('invite_user', { email: inviteEmail, role: inviteRole });
      toast.success('Invite sent successfully!');
      setInviteEmail('');
      loadMembers();
    } catch {
      toast.error('Failed to send invite');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content workspace-settings-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div className="title-area">
            <Users size={20} className="accent-icon" />
            <h3>Workspace Collaboration</h3>
          </div>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        <div className="modal-body-content">
          <div className="invite-section">
            <h4>Invite Team Members</h4>
            <div className="invite-form">
              <div className="input-with-icon">
                <Mail size={16} />
                <input
                  placeholder="name@company.com"
                  value={inviteEmail}
                  onChange={(e) => setInviteEmail(e.target.value)}
                />
              </div>
              <select
                value={inviteRole}
                onChange={(e) => setInviteRole(e.target.value as MemberRole)}
              >
                <option value="Viewer">Viewer</option>
                <option value="Editor">Editor</option>
                <option value="Admin">Admin</option>
              </select>
              <button className="invite-btn" onClick={handleInvite} disabled={loading}>
                {loading ? (
                  '...'
                ) : (
                  <>
                    <UserPlus size={16} /> Invite
                  </>
                )}
              </button>
            </div>
          </div>

          <div className="members-list-section">
            <h4>Workspace Members ({members.length})</h4>
            <div className="members-list">
              {members.map((m) => (
                <div key={m.user_id} className="member-row animate-fade-in">
                  <div className="member-main">
                    <div className="member-avatar">{m.email[0].toUpperCase()}</div>
                    <div className="member-details">
                      <span className="member-email">{m.email}</span>
                      <span className="member-id">ID: {m.user_id.slice(0, 8)}...</span>
                    </div>
                  </div>
                  <div className="member-role-badge" data-role={m.role}>
                    {m.role === 'Admin' ? <Shield size={14} /> : <User size={14} />}
                    {m.role}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="members-list-section">
            <h4>Script Libraries</h4>
            <div className="members-list">
              {libraries.map((lib) => (
                <div key={lib.name} className="member-row animate-fade-in">
                  <div className="member-main">
                    <div className="member-avatar">
                      <Package size={16} />
                    </div>
                    <div className="member-details">
                      <span className="member-email">
                        {lib.name} <small>v{lib.version}</small>
                      </span>
                      <span className="member-id">{lib.description}</span>
                    </div>
                  </div>
                  <button
                    className="invite-btn"
                    onClick={() => toggleLibrary(lib)}
                    title={lib.enabled ? 'Disable for scripts' : 'Enable for scripts'}
                  >
                    {lib.enabled ? (
                      <ToggleRight size={18} color="#22c55e" />
                    ) : (
                      <ToggleLeft size={18} />
                    )}
                  </button>
                </div>
              ))}
            </div>
          </div>

          <div className="sync-status-footer">
            <CheckCircle2 size={16} color="#22c55e" />
            <span>Workspace is connected and syncing in real-time</span>
          </div>
        </div>
      </div>
    </div>
  );
};
