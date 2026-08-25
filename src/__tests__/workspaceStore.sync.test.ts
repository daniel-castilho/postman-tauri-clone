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

const { invoke } = await import('@tauri-apps/api/core');
const { listen } = await import('@tauri-apps/api/event');
const { useWorkspaceStore } = await import('../store/workspaceStore');

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

async function freshStore() {
  const store = useWorkspaceStore;
  store.setState({
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
  vi.mocked(invoke).mockReset();
  return store.getState();
}

describe('useWorkspaceStore (request updates)', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('updateRequest persists through the parent collection and queues sync', async () => {
    const state = await freshStore();
    const original = sampleRequest();
    useWorkspaceStore.setState({
      collections: [
        {
          id: 'col_1',
          name: 'Main',
          description: null,
          items: [{ Request: original }],
          variables: {},
        },
      ],
    });
    // Internal reload keeps the current tree; mutations resolve as no-ops.
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'load_collections') return useWorkspaceStore.getState().collections;
      return undefined;
    });

    await state.updateRequest(sampleRequest({ name: 'Get users v2' }));

    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      'save_collection',
      expect.objectContaining({ workspacePath: '/tmp/ws' }),
    );
    expect(useWorkspaceStore.getState().activeRequest?.name).toBe('Get users v2');
    // Drains the queue immediately on successful relay.
    expect(useWorkspaceStore.getState().syncQueue).toHaveLength(0);
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      'sync_resource_change',
      expect.objectContaining({ resource_id: 'req_1' }),
    );
  });

  it('updateRequest is a no-op when the request lives in no collection', async () => {
    const state = await freshStore();
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.updateRequest(sampleRequest({ id: 'ghost' }));

    expect(vi.mocked(invoke)).not.toHaveBeenCalledWith(
      'save_collection',
      expect.anything(),
    );
  });
});

describe('useWorkspaceStore (error and edge branches)', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it('loadEnvironments selects the first environment when none is active', async () => {
    const state = await freshStore();
    useWorkspaceStore.setState({ environments: [], activeEnvironmentId: null });
    vi.mocked(invoke).mockResolvedValue([
      { id: 'env_a', name: 'A', variables: [] },
      { id: 'env_b', name: 'B', variables: [] },
    ]);

    await state.loadEnvironments();

    expect(useWorkspaceStore.getState().activeEnvironmentId).toBe('env_a');
  });

  it('environment/globals/design failures degrade without throwing', async () => {
    const state = await freshStore();
    vi.mocked(invoke).mockRejectedValue(new Error('io'));

    await expect(state.saveEnvironments()).resolves.toBeUndefined();
    await expect(state.loadGlobals()).resolves.toBeUndefined();
    await expect(state.loadDesigns()).resolves.toBeUndefined();
    await state.setActiveDesign('spec_x');

    vi.mocked(invoke).mockRejectedValue(new Error('io'));
    await expect(state.saveDesign(sampleDesign())).resolves.toBeUndefined();
    await expect(state.createDesign('X', 'yaml')).resolves.toBeUndefined();
    await expect(state.deleteDesign('spec_x')).resolves.toBeUndefined();

    // State remains usable afterwards.
    expect(useWorkspaceStore.getState().workspacePath).toBe('/tmp/ws');
  });

  it('exportWorkspace skips the backend when the dialog is cancelled', async () => {
    const state = await freshStore();
    const { save } = await import('@tauri-apps/plugin-dialog');
    vi.mocked(save).mockResolvedValue(null);

    await state.exportWorkspace();

    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it('importCollection swallows backend failures', async () => {
    const state = await freshStore();
    const { open } = await import('@tauri-apps/plugin-dialog');
    vi.mocked(open).mockResolvedValue('/imports/broken.json');
    vi.mocked(invoke).mockRejectedValue(new Error('bad spec'));

    await expect(state.importCollection()).resolves.toBeUndefined();
  });
});

function sampleDesign() {
  return {
    id: 'spec_1',
    name: 'Billing API',
    content: '',
    format: 'yaml',
    version: 'OpenAPI 3.0',
    last_modified: 'now',
  };
}

describe('useWorkspaceStore (real-time sync listener)', () => {
  // The module registers its listener at import time; grab the latest
  // registered handler instead of resetting modules (which would fork state).
  function capturedSyncHandler(): ((event: { payload: unknown }) => void) | undefined {
    const calls = vi.mocked(listen).mock.calls;
    return calls.length > 0 ? (calls[calls.length - 1][1] as typeof syncHandler) : undefined;
  }
  let syncHandler: ((event: { payload: unknown }) => void) | undefined;

  beforeEach(async () => {
    await freshStore();
    syncHandler = capturedSyncHandler();
  });

  it('applies remote Request updates into the local collection tree', async () => {
    await freshStore();
    const remote = sampleRequest({ name: 'Renamed remotely' });
    useWorkspaceStore.setState({
      collections: [
        {
          id: 'col_1',
          name: 'Main',
          description: null,
          items: [{ Request: sampleRequest() }],
          variables: {},
        },
      ],
    });

    console.error('DBG handler?', !!syncHandler, 'calls:', vi.mocked(listen).mock.calls.length);
syncHandler?.({
      payload: {
        id: 'chg',
        resource_type: 'Request',
        resource_id: 'req_1',
        operation: 'Update',
        data: JSON.stringify(remote),
        timestamp: new Date().toISOString(),
      },
    });

    const items = useWorkspaceStore.getState().collections[0]?.items ?? [];
    expect('Request' in items[0]! && items[0].Request.name).toBe('Renamed remotely');
  });

  it('merges remote Environment updates while preserving local current values', async () => {
    await freshStore();
    useWorkspaceStore.setState({
      environments: [
        {
          id: 'env_local',
          name: 'Local',
          variables: [
            {
              key: 'TOKEN',
              initial_value: 'remote-secret',
              current_value: 'local-edit',
              var_type: 'Public',
              enabled: true,
            },
          ],
        },
      ],
    });

    syncHandler?.({
      payload: {
        id: 'chg',
        resource_type: 'Environment',
        resource_id: 'env_local',
        operation: 'Update',
        data: JSON.stringify({
          id: 'env_local',
          name: 'Local',
          variables: [
            {
              key: 'TOKEN',
              initial_value: 'remote-secret',
              current_value: '',
              var_type: 'Public',
              enabled: true,
            },
          ],
        }),
        timestamp: new Date().toISOString(),
      },
    });

    const variables = useWorkspaceStore.getState().environments[0]?.variables ?? [];
    // Remote initial value arrives, but the local edit survives the merge.
    expect(variables[0]?.current_value).toBe('local-edit');
  });
});
