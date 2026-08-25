import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Users, UserPlus, Shield, User, Mail, X, CheckCircle2 } from 'lucide-react';
import { toast } from 'sonner';
import type { MemberRole, WorkspaceMember } from '../types/ipc';
import './WorkspaceSettings.css';

interface Props {
  onClose: () => void;
}

export const WorkspaceSettings: React.FC<Props> = ({ onClose }) => {
  const [members, setMembers] = useState<WorkspaceMember[]>([]);
  const [inviteEmail, setInviteEmail] = useState('');
  const [inviteRole, setInviteRole] = useState<MemberRole>('Viewer');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadMembers();
  }, []);

  const loadMembers = async () => {
    try {
      const list = await invoke<WorkspaceMember[]>('get_members');
      setMembers(list);
    } catch (e) {
      toast.error("Erro ao carregar membros");
    }
  };

  const handleInvite = async () => {
    if (!inviteEmail) return;
    setLoading(true);
    try {
      await invoke('invite_user', { email: inviteEmail, role: inviteRole });
      toast.success("Convite enviado com sucesso!");
      setInviteEmail('');
      loadMembers();
    } catch (e) {
      toast.error("Falha ao enviar convite");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content workspace-settings-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div className="title-area">
            <Users size={20} className="accent-icon" />
            <h3>Workspace Collaboration</h3>
          </div>
          <button className="close-btn" onClick={onClose}><X size={20} /></button>
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
                  onChange={e => setInviteEmail(e.target.value)}
                />
              </div>
              <select value={inviteRole} onChange={e => setInviteRole(e.target.value as any)}>
                <option value="Viewer">Viewer</option>
                <option value="Editor">Editor</option>
                <option value="Admin">Admin</option>
              </select>
              <button className="invite-btn" onClick={handleInvite} disabled={loading}>
                {loading ? '...' : <><UserPlus size={16} /> Invite</>}
              </button>
            </div>
          </div>

          <div className="members-list-section">
            <h4>Workspace Members ({members.length})</h4>
            <div className="members-list">
              {members.map(m => (
                <div key={m.user_id} className="member-row animate-fade-in">
                  <div className="member-main">
                    <div className="member-avatar">
                      {m.email[0].toUpperCase()}
                    </div>
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

          <div className="sync-status-footer">
            <CheckCircle2 size={16} color="#22c55e" />
            <span>Workspace is connected and syncing in real-time</span>
          </div>
        </div>
      </div>
    </div>
  );
};
