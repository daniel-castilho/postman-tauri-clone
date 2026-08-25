import React from 'react';
import { ArrowUp, ArrowDown, RefreshCw, Upload, Download } from 'lucide-react';
import type { GitStatusSummaryDto } from '../../types/ipc';

interface Props {
  status: GitStatusSummaryDto;
  busy: boolean;
  onRefresh: () => void;
  onPush: () => void;
  onPull: () => void;
}

export const GitSyncHeader: React.FC<Props> = ({ status, busy, onRefresh, onPush, onPull }) => (
  <div className="git-sync-header">
    <div className="git-branch-badge" title="Active branch">
      {status.current_branch || '—'}
    </div>
    <div className="git-counters">
      <span className="git-counter git-ahead" title="Commits ahead of remote">
        <ArrowUp size={13} /> {status.ahead_count}
      </span>
      <span className="git-counter git-behind" title="Commits behind remote">
        <ArrowDown size={13} /> {status.behind_count}
      </span>
    </div>
    <div className="git-sync-actions">
      <button className="git-action-btn" onClick={onRefresh} disabled={busy} title="Refresh status">
        <RefreshCw size={14} />
      </button>
      <button className="git-action-btn" onClick={onPush} disabled={busy} title="Push to remote">
        <Upload size={14} /> Push
      </button>
      <button className="git-action-btn" onClick={onPull} disabled={busy} title="Pull from remote">
        <Download size={14} /> Pull
      </button>
    </div>
  </div>
);
