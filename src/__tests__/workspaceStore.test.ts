import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { HttpRequest } from '../types/ipc';

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

function sampleRequest(overrides: Partial<HttpRequest> = {}): HttpRequest {
  return {
    id: 'req_1',
    name: 'Get users',
    description: null,
    method: 'GET',
    url: 'https://api.example.com/users',
    headers: [],
    body: null,
    auth: { type: 'NoAuth' },
    variables: {},
    scripts: { preRequest: '', tests: '' },
    grpc_config: null,
    ...overrides,
  };
}

describe('useWorkspaceStore (synchronous UI state)', () => {
  beforeEach(async () => {
    const { useWorkspaceStore } = await import('../store/workspaceStore');
    useWorkspaceStore.setState({
      workspacePath: '',
      activeRequest: null,
      openTabs: [],
      activeTabId: null,
      history: [],
      sessionVariables: {},
      sidebarMode: 'Collections',
    });
  });

  it('sets the workspace path and sidebar mode', async () => {
    const { useWorkspaceStore } = await import('../store/workspaceStore');
    useWorkspaceStore.getState().setWorkspacePath('/tmp/ws');
    useWorkspaceStore.getState().setSidebarMode('History');
    expect(useWorkspaceStore.getState().workspacePath).toBe('/tmp/ws');
    expect(useWorkspaceStore.getState().sidebarMode).toBe('History');
  });

  it('opens a tab, focuses an existing tab, and closes the active tab', async () => {
    const { useWorkspaceStore } = await import('../store/workspaceStore');
    const first = sampleRequest({ id: 'req_1' });
    const second = sampleRequest({ id: 'req_2', name: 'Get posts' });

    useWorkspaceStore.getState().addTab(first);
    useWorkspaceStore.getState().addTab(second);
    expect(useWorkspaceStore.getState().openTabs).toHaveLength(2);
    expect(useWorkspaceStore.getState().activeTabId).toBe('req_2');

    useWorkspaceStore.getState().addTab(first);
    expect(useWorkspaceStore.getState().openTabs).toHaveLength(2);
    expect(useWorkspaceStore.getState().activeTabId).toBe('req_1');

    useWorkspaceStore.getState().closeTab('req_1');
    expect(useWorkspaceStore.getState().openTabs).toHaveLength(1);
    expect(useWorkspaceStore.getState().activeTabId).toBe('req_2');
  });

  it('records history with a 50-item cap and can clear it', async () => {
    const { useWorkspaceStore } = await import('../store/workspaceStore');
    for (let i = 0; i < 52; i += 1) {
      useWorkspaceStore.getState().addToHistory(sampleRequest({ id: `req_${i}` }));
    }
    expect(useWorkspaceStore.getState().history).toHaveLength(50);
    useWorkspaceStore.getState().clearHistory();
    expect(useWorkspaceStore.getState().history).toHaveLength(0);
  });

  it('stores session variables', async () => {
    const { useWorkspaceStore } = await import('../store/workspaceStore');
    useWorkspaceStore.getState().setSessionVariable('token', 'abc');
    expect(useWorkspaceStore.getState().sessionVariables.token).toBe('abc');
  });
});
