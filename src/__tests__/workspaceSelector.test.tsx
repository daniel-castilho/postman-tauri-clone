import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { WorkspaceSelector } from '../components/WorkspaceSelector';
import { useWorkspaceStore } from '../store/workspaceStore';

// The store registers a module-level sync listener; keep the real Tauri
// API harmless when it leaks into the module graph.
(globalThis as Record<string, unknown>).__TAURI_INTERNALS__ ??= {
  transformCallback: (callback: unknown) => callback,
  invoke: async () => undefined,
  metadata: { currentWindow: { label: 'test' }, currentWebview: { label: 'test' } },
};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

const openWorkspace = vi.fn();

describe('WorkspaceSelector', () => {
  afterEach(cleanup);

  beforeEach(() => {
    vi.clearAllMocks();
    useWorkspaceStore.setState({
      workspacePath: '',
      openWorkspace,
      error: null,
    });
  });

  it('renders the welcome card and triggers openWorkspace on click', () => {
    render(<WorkspaceSelector />);

    expect(screen.getByText('Welcome to Tyny Pulse')).toBeTruthy();
    expect(screen.getByText(/open a folder to use as your workspace/)).toBeTruthy();
    expect(screen.getByText('Open Workspace')).toBeTruthy();

    fireEvent.click(screen.getByRole('button'));
    expect(openWorkspace).toHaveBeenCalledTimes(1);
  });

  it('surfaces workspace errors when present', () => {
    useWorkspaceStore.setState({ error: 'disk exploded' });
    render(<WorkspaceSelector />);
    expect(screen.getByText(/Error: disk exploded/)).toBeTruthy();
  });

  it('hides the error box when there is no error', () => {
    render(<WorkspaceSelector />);
    expect(screen.queryByText(/Error:/)).toBeNull();
  });
});
