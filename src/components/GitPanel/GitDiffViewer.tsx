import React from 'react';
import type { GitFileDiffDto } from '../../types/ipc';
import { X } from 'lucide-react';

interface Props {
  diff: GitFileDiffDto | null;
  onClose: () => void;
}

export const GitDiffViewer: React.FC<Props> = ({ diff, onClose }) => {
  if (!diff) return null;

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content git-diff-modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <div className="title-area">
            <h3>{diff.path}</h3>
          </div>
          <button className="close-btn" onClick={onClose}><X size={20} /></button>
        </div>
        <div className="git-diff-body">
          {diff.chunks.length === 0 && (
            <div className="git-diff-empty">No textual changes (file may be binary or identical).</div>
          )}
          {diff.chunks.map((chunk, index) => (
            <div key={index} className={`git-diff-line git-diff-${chunk.change_type}`}>
              <span className="git-diff-lineno">{chunk.old_line_number ?? ''}</span>
              <span className="git-diff-lineno">{chunk.new_line_number ?? ''}</span>
              <span className="git-diff-marker">
                {chunk.change_type === 'add' ? '+' : chunk.change_type === 'delete' ? '-' : ' '}
              </span>
              <span className="git-diff-content">{chunk.content || ' '}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
