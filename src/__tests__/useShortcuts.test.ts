import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { renderHook } from '@testing-library/react';
import type { Environment, HttpRequest } from '../types/ipc';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn(), info: vi.fn() },
}));

const { invoke } = await import('@tauri-apps/api/core');
const { toast } = await import('sonner');
const { useWorkspaceStore } = await import('../store/workspaceStore');
const { useShortcuts } = await import('../hooks/useShortcuts');

function sampleRequest(): HttpRequest {
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
  };
}

function pressKey(key: string, modifiers: { ctrl?: boolean; meta?: boolean } = {}): void {
  window.dispatchEvent(
    new KeyboardEvent('keydown', {
      key,
      ctrlKey: modifiers.ctrl ?? false,
      metaKey: modifiers.meta ?? false,
      cancelable: true,
    }),
  );
}

describe('useShortcuts (global keyboard shortcuts)', () => {
  const setToggle = vi.fn();
  let result: ReturnType<typeof renderHook> | undefined;
  let updateRequestSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    setToggle.mockClear();
    vi.mocked(invoke).mockReset();
    useWorkspaceStore.setState({
      workspacePath: '/tmp/ws',
      activeRequest: null,
      openTabs: [],
      activeTabId: null,
      history: [],
      sessionVariables: {},
      sidebarMode: 'Collections',
      collections: [],
      environments: [],
      activeEnvironmentId: null,
      globals: { variables: {} },
      designs: [],
      activeDesignId: null,
      syncQueue: [],
      isOnline: true,
      isLoading: false,
      error: null,
    });
    document.body.innerHTML = '';
  });

  afterEach(() => {
    // Unmount the hook so listeners are removed between tests.
    result?.unmount();
    result = undefined;
  });

  function mountHook(): void {
    result = renderHook(() => useShortcuts(setToggle));
  }

  it('toggles the command palette on Ctrl+P and Ctrl+K', () => {
    mountHook();
    pressKey('p', { ctrl: true });
    expect(setToggle).toHaveBeenCalledTimes(1);
    pressKey('k', { meta: true });
    expect(setToggle).toHaveBeenCalledTimes(2);
  });

  it('dispatches trigger-send on Ctrl+Enter, clicking the send button once', () => {
    mountHook();
    const button = document.createElement('button');
    button.className = 'send-btn';
    const clickSpy = vi.fn();
    button.addEventListener('click', clickSpy);
    document.body.appendChild(button);

    pressKey('Enter', { ctrl: true });

    expect(clickSpy).toHaveBeenCalledTimes(1);
  });

  it('saves the active request on Ctrl+S', () => {
    useWorkspaceStore.setState({ activeRequest: sampleRequest() });
    updateRequestSpy = vi
      .spyOn(useWorkspaceStore.getState(), 'updateRequest')
      .mockResolvedValue(undefined);
    mountHook();

    pressKey('s', { ctrl: true });

    expect(updateRequestSpy).toHaveBeenCalledWith(expect.objectContaining({ id: 'req_1' }));
    expect(toast.success).toHaveBeenCalledWith('Request saved!');
  });

  it('ignores Ctrl+S when no request is active', () => {
    mountHook();

    pressKey('s', { ctrl: true });

    expect(toast.success).not.toHaveBeenCalled();
  });

  it('switches to the environment matching the pressed number key', () => {
    const environments: Environment[] = [
      { id: 'env_a', name: 'Alpha', variables: [] },
      { id: 'env_b', name: 'Beta', variables: [] },
    ];
    useWorkspaceStore.setState({
      environments,
      activeEnvironmentId: 'env_a',
      activeRequest: sampleRequest(),
    });
    updateRequestSpy = vi
      .spyOn(useWorkspaceStore.getState(), 'updateRequest')
      .mockResolvedValue(undefined);
    mountHook();

    pressKey('2', { ctrl: true });

    expect(useWorkspaceStore.getState().activeEnvironmentId).toBe('env_b');
    expect(toast.success).toHaveBeenCalledWith('Environment: Beta');
  });

  it('does nothing for number keys beyond the environment list or the zero key', () => {
    const environments: Environment[] = [{ id: 'env_a', name: 'Alpha', variables: [] }];
    useWorkspaceStore.setState({ environments, activeEnvironmentId: 'env_a' });
    mountHook();

    pressKey('5', { ctrl: true });
    pressKey('0', { ctrl: true });

    expect(useWorkspaceStore.getState().activeEnvironmentId).toBe('env_a');
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });
});
