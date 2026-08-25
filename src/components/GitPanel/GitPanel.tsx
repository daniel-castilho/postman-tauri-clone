import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  GitBranch,
  X,
  FileJson,
  Plus,
  CheckCircle2,
  AlertTriangle,
  ChevronDown,
  Undo2,
} from 'lucide-react';
import { toast } from 'sonner';
import type { GitFileChangeDto, GitFileDiffDto, GitStatusSummaryDto } from '../../types/ipc';
import { useWorkspaceStore } from '../../store/workspaceStore';
import { GitSyncHeader } from './GitSyncHeader';
import { GitDiffViewer } from './GitDiffViewer';
import './GitPanel.css';

interface Props {
  onClose: () => void;
}

const STATUS_LABELS: Record<string, string> = {
  Untracked: 'U',
  Modified: 'M',
  Added: 'A',
  Deleted: 'D',
  Renamed: 'R',
  Conflicted: '!',
};

export const GitPanel: React.FC<Props> = ({ onClose }) => {
  const [status, setStatus] = useState<GitStatusSummaryDto | null>(null);
  const [busy, setBusy] = useState(false);
  const [summary, setSummary] = useState('');
  const [description, setDescription] = useState('');
  const [branchMenuOpen, setBranchMenuOpen] = useState(false);
  const [newBranchName, setNewBranchName] = useState('');
  const [diff, setDiff] = useState<GitFileDiffDto | null>(null);
  const workspacePath = useWorkspaceStore(
    (state: { workspacePath: string }) => state.workspacePath,
  );

  const refresh = useCallback(async () => {
    if (!workspacePath) return;
    setBusy(true);
    try {
      setStatus(await invoke<GitStatusSummaryDto>('git_get_status', { workspacePath }));
    } catch {
      toast.error('Failed to read Git status');
    } finally {
      setBusy(false);
    }
  }, [workspacePath]);

  useEffect(() => {
    // Initial load without toggling the global busy flag, which is reserved
    // for user-triggered actions. Re-runs when the workspace path changes.
    let cancelled = false;
    void (async () => {
      if (!workspacePath || cancelled) return;
      try {
        const result = await invoke<GitStatusSummaryDto>('git_get_status', { workspacePath });
        if (!cancelled) setStatus(result);
      } catch {
        toast.error('Failed to read Git status');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [workspacePath]);

  const runAction = async (action: () => Promise<unknown>, successMessage?: string) => {
    setBusy(true);
    try {
      await action();
      if (successMessage) toast.success(successMessage);
      await refresh();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  const openDiff = async (change: GitFileChangeDto) => {
    try {
      setDiff(
        await invoke<GitFileDiffDto>('git_get_file_diff', {
          workspacePath,
          file: change.path,
          staged: change.is_staged,
        }),
      );
    } catch (e) {
      toast.error(String(e));
    }
  };

  const handleCommit = async () => {
    if (!summary.trim()) return;
    const message = description.trim()
      ? `${summary.trim()}\n\n${description.trim()}`
      : summary.trim();
    await runAction(() => invoke('git_commit', { workspacePath, message }), `Committed`);
    setSummary('');
    setDescription('');
  };

  const handleCommitKeyDown = (event: React.KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
      event.preventDefault();
      handleCommit();
    }
  };

  const checkout = (branchName: string, createIfMissing: boolean) => {
    setBranchMenuOpen(false);
    setNewBranchName('');
    return runAction(
      () => invoke('git_checkout_branch', { workspacePath, branchName, createIfMissing }),
      `On branch ${branchName}`,
    );
  };

  const staged = status?.files.filter((f) => f.is_staged) ?? [];
  const unstaged = status?.files.filter((f) => !f.is_staged) ?? [];

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content git-panel-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <div className="title-area">
            <GitBranch size={18} className="accent-icon" />
            <h3>Source Control</h3>
          </div>
          <button className="close-btn" onClick={onClose}>
            <X size={20} />
          </button>
        </div>

        {!status && <div className="git-empty">Loading repository status…</div>}

        {status && !status.is_repository && (
          <div className="git-empty">
            <AlertTriangle size={20} />
            <p>
              This workspace is not a Git repository yet.
              <br />
              Run <code>git init</code> to enable version control.
            </p>
          </div>
        )}

        {status?.is_repository && (
          <div className="modal-body-content git-panel-body">
            <GitSyncHeader
              status={status}
              busy={busy}
              onRefresh={refresh}
              onPush={() => runAction(() => invoke('git_push', { workspacePath }), 'Pushed')}
              onPull={() => runAction(() => invoke('git_pull', { workspacePath }), 'Pulled')}
            />

            <div className="git-branch-row">
              <button
                className="git-branch-switcher"
                onClick={() => setBranchMenuOpen((open) => !open)}
              >
                <GitBranch size={13} />
                {status.current_branch || 'HEAD'}
                <ChevronDown size={13} />
              </button>
              <button
                className="git-stage-all-btn"
                onClick={() => runAction(() => invoke('git_stage_all', { workspacePath }))}
                disabled={busy || unstaged.length === 0}
                title="Stage all changes"
              >
                <Plus size={13} /> Stage all
              </button>
            </div>

            {branchMenuOpen && (
              <div className="git-branch-menu">
                {status.branches.map((branch) => (
                  <button key={branch} onClick={() => checkout(branch, false)}>
                    <GitBranch size={12} /> {branch}
                    {branch === status.current_branch && <CheckCircle2 size={12} />}
                  </button>
                ))}
                <div className="git-branch-create">
                  <input
                    placeholder="new-branch-name"
                    value={newBranchName}
                    onChange={(e) => setNewBranchName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && newBranchName.trim())
                        checkout(newBranchName.trim(), true);
                    }}
                  />
                  <button
                    disabled={!newBranchName.trim()}
                    onClick={() => checkout(newBranchName.trim(), true)}
                  >
                    Create
                  </button>
                </div>
              </div>
            )}

            <div className="git-files-section">
              <h4>Staged Changes ({staged.length})</h4>
              {staged.length === 0 && <div className="git-no-files">Nothing staged</div>}
              {staged.map((change) => (
                <div key={`${change.path}-staged`} className="git-file-row">
                  <button
                    className="git-file-name"
                    onClick={() => openDiff(change)}
                    title="View diff"
                  >
                    <FileJson size={13} /> {change.path}
                  </button>
                  <span className={`git-status-badge git-status-${change.status}`}>
                    {STATUS_LABELS[change.status]}
                  </span>
                  <button
                    className="git-file-action"
                    disabled={busy}
                    onClick={() =>
                      runAction(() =>
                        invoke('git_unstage_file', { workspacePath, file: change.path }),
                      )
                    }
                    title="Unstage"
                  >
                    <Undo2 size={13} />
                  </button>
                </div>
              ))}
            </div>

            <div className="git-files-section">
              <h4>Changes ({unstaged.length})</h4>
              {unstaged.length === 0 && <div className="git-no-files">Working tree clean</div>}
              {unstaged.map((change) => (
                <div key={`${change.path}-unstaged`} className="git-file-row">
                  <button
                    className="git-file-name"
                    onClick={() => openDiff(change)}
                    title="View diff"
                  >
                    <FileJson size={13} /> {change.path}
                  </button>
                  <span className={`git-status-badge git-status-${change.status}`}>
                    {STATUS_LABELS[change.status]}
                  </span>
                  <button
                    className="git-file-action"
                    disabled={busy}
                    onClick={() =>
                      runAction(() =>
                        invoke('git_stage_file', { workspacePath, file: change.path }),
                      )
                    }
                    title="Stage"
                  >
                    <Plus size={13} />
                  </button>
                </div>
              ))}
            </div>

            <div className="git-commit-section" onKeyDown={handleCommitKeyDown}>
              <input
                className="git-commit-summary"
                placeholder="Commit summary (Ctrl+Enter to commit)"
                value={summary}
                onChange={(e) => setSummary(e.target.value)}
              />
              <textarea
                className="git-commit-description"
                placeholder="Extended description (optional)"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                rows={3}
              />
              <button
                className="git-commit-btn"
                onClick={handleCommit}
                disabled={busy || !summary.trim() || staged.length === 0}
              >
                Commit ({staged.length} staged)
              </button>
            </div>
          </div>
        )}

        <GitDiffViewer diff={diff} onClose={() => setDiff(null)} />
      </div>
    </div>
  );
};
