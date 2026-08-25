import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Collection, HttpRequest } from '../types/ipc';

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
const { open } = await import('@tauri-apps/plugin-dialog');
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

function sampleCollection(): Collection {
  return {
    id: 'col_1',
    name: 'Main',
    description: null,
    items: [{ Request: sampleRequest() }],
    variables: {},
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

/**
 * The store chains IPC calls internally (e.g. every mutation reloads the
 * collection list afterwards), so tests register per-command handlers
 * instead of ordered one-shot responses.
 */
function mockBackend(handlers: Record<string, (payload: unknown) => unknown> = {}): void {
  vi.mocked(invoke).mockImplementation(async (command: string, payload?: unknown) => {
    const handler = handlers[command];
    if (handler) return handler(payload);
    // Sensible default: internal reloads keep the current list.
    if (command === 'load_collections') return useWorkspaceStore.getState().collections;
    return undefined;
  });
}

/** Simulates disk persistence: mutations feed subsequent internal reloads. */
function mockPersistence(initial: Collection[]): void {
  let stored: Collection[] = initial;
  mockBackend({
    save_collection: (payload) => {
      stored = [(payload as { collection: Collection }).collection];
      return undefined;
    },
    delete_collection: (payload) => {
      const id = (payload as { collectionId: string }).collectionId;
      stored = stored.filter((collection) => collection.id !== id);
      return undefined;
    },
    load_collections: () => stored,
  });
}

describe('useWorkspaceStore (tabs and requests)', () => {
  beforeEach(async () => {
    await freshStore();
  });

  it('setActiveRequest opens a tab and focuses it', async () => {
    const state = await freshStore();
    const request = sampleRequest();
    state.setActiveRequest(request);
    const after = useWorkspaceStore.getState();
    expect(after.openTabs).toHaveLength(1);
    expect(after.activeTabId).toBe('req_1');
    expect(after.activeRequest?.id).toBe('req_1');
  });

  it('addTab deduplicates by id', async () => {
    const state = await freshStore();
    const request = sampleRequest();
    state.addTab(request);
    state.addTab(request);
    expect(useWorkspaceStore.getState().openTabs).toHaveLength(1);
  });

  it('closeTab falls back to the last remaining tab', async () => {
    const state = await freshStore();
    state.addTab(sampleRequest({ id: 'req_1' }));
    state.addTab(sampleRequest({ id: 'req_2' }));
    state.closeTab('req_2');
    const after = useWorkspaceStore.getState();
    expect(after.activeTabId).toBe('req_1');
    expect(after.activeRequest?.id).toBe('req_1');
  });

  it('closeTab clears the selection when the last tab closes', async () => {
    const state = await freshStore();
    state.addTab(sampleRequest());
    state.closeTab('req_1');
    const after = useWorkspaceStore.getState();
    expect(after.openTabs).toHaveLength(0);
    expect(after.activeTabId).toBeNull();
    expect(after.activeRequest).toBeNull();
  });

  it('setActiveRequestTab restores the request for an existing tab', async () => {
    const state = await freshStore();
    state.addTab(sampleRequest({ id: 'req_1' }));
    state.setActiveRequest(null); // force clear through public API
    state.setActiveRequestTab('req_1');
    expect(useWorkspaceStore.getState().activeRequest?.id).toBe('req_1');
  });
});

describe('useWorkspaceStore (collections persistence)', () => {
  beforeEach(async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(open).mockReset();
  });

  it('loadCollections fetches collections for the current workspace', async () => {
    const state = await freshStore();
    const collections = [sampleCollection()];
    vi.mocked(invoke).mockResolvedValue(collections);

    await state.loadCollections();

    expect(vi.mocked(invoke)).toHaveBeenCalledWith('load_collections', {
      workspacePath: '/tmp/ws',
    });
    expect(useWorkspaceStore.getState().collections).toEqual(collections);
    expect(useWorkspaceStore.getState().isLoading).toBe(false);
  });

  it('loadCollections is a no-op without a workspace', async () => {
    await freshStore();
    useWorkspaceStore.setState({ workspacePath: '' });
    await useWorkspaceStore.getState().loadCollections();
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it('loadCollections surfaces errors into state', async () => {
    await freshStore();
    vi.mocked(invoke).mockRejectedValue('disk full');
    await useWorkspaceStore.getState().loadCollections();
    expect(useWorkspaceStore.getState().error).toContain('disk full');
    expect(useWorkspaceStore.getState().isLoading).toBe(false);
  });

  it('saveCollection persists then reloads from disk', async () => {
    const state = await freshStore();
    const reloaded = [sampleCollection()];
    vi.mocked(invoke)
      .mockResolvedValueOnce(undefined) // save_collection
      .mockResolvedValueOnce(reloaded); // load_collections

    await state.saveCollection(sampleCollection());

    const calls = vi.mocked(invoke).mock.calls.map(([command]) => command);
    expect(calls).toEqual(['save_collection', 'load_collections']);
    expect(useWorkspaceStore.getState().collections).toEqual(reloaded);
  });

  it('deleteCollection calls the IPC contract and reloads', async () => {
    const state = await freshStore();
    vi.mocked(invoke)
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce([]);

    await state.deleteCollection('col_1');

    expect(vi.mocked(invoke)).toHaveBeenCalledWith('delete_collection', {
      workspacePath: '/tmp/ws',
      collectionId: 'col_1',
    });
    expect(useWorkspaceStore.getState().collections).toEqual([]);
  });

  it('addCollection creates, persists and reloads', async () => {
    const state = await freshStore();
    const reloaded = [
      { id: 'col_x', name: 'Created', description: null, items: [], variables: {} },
    ];
    mockBackend({ load_collections: () => reloaded });

    await state.addCollection('Created');

    expect(useWorkspaceStore.getState().collections[0]?.name).toBe('Created');
  });

  it('addRequestToCollection appends a default request and selects it', async () => {
    const state = await freshStore();
    useWorkspaceStore.setState({ collections: [sampleCollection()] });
    mockPersistence([sampleCollection()]);

    await state.addRequestToCollection('col_1');

    const after = useWorkspaceStore.getState();
    expect(after.collections[0]?.items).toHaveLength(2);
    expect(after.activeRequest?.name).toBe('New Request');
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      'save_collection',
      expect.objectContaining({ workspacePath: '/tmp/ws' }),
    );
  });

  it('addFolderToCollection appends a folder node', async () => {
    const state = await freshStore();
    useWorkspaceStore.setState({ collections: [sampleCollection()] });
    mockPersistence([sampleCollection()]);

    await state.addFolderToCollection('col_1', 'Auth flows');

    const items = useWorkspaceStore.getState().collections[0]?.items ?? [];
    expect(items).toHaveLength(2);
    expect('Folder' in items[1]! && items[1].Folder.name).toBe('Auth flows');
  });

  it('deleteRequest removes nested requests and clears the active selection', async () => {
    const state = await freshStore();
    const collection = sampleCollection();
    useWorkspaceStore.setState({
      collections: [collection],
      activeRequest: sampleRequest(),
    });
    mockPersistence([collection]);

    await state.deleteRequest('req_1');

    const after = useWorkspaceStore.getState();
    expect(after.collections[0]?.items).toHaveLength(0);
    expect(after.activeRequest).toBeNull();
  });

  it('duplicateRequest clones with a fresh id and copy suffix', async () => {
    const state = await freshStore();
    useWorkspaceStore.setState({ collections: [sampleCollection()] });
    mockPersistence([sampleCollection()]);

    await state.duplicateRequest('req_1');

    const items = useWorkspaceStore.getState().collections[0]?.items ?? [];
    expect(items).toHaveLength(2);
    const copy = items[1];
    expect('Request' in copy && copy.Request.name).toBe('Get users (Copy)');
    expect('Request' in copy && copy.Request.id).not.toBe('req_1');
  });

  it('reorderItems persists the new item order', async () => {
    const state = await freshStore();
    const reordered = [
      { Folder: { name: 'F', description: '', items: [] } },
      { Request: sampleRequest() },
    ];
    useWorkspaceStore.setState({ collections: [sampleCollection()] });
    mockPersistence([sampleCollection()]);

    await state.reorderItems('col_1', reordered);

    expect(useWorkspaceStore.getState().collections[0]?.items[0]).toEqual(reordered[0]);
  });
});

describe('useWorkspaceStore (environments, globals, designs)', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(open).mockReset();
  });

  it('addEnvironment appends and persists through save_environments', async () => {
    const state = await freshStore();
    useWorkspaceStore.setState({ environments: [] });
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.addEnvironment({
      id: 'env_prod',
      name: 'Prod',
      variables: [],
    });

    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_environments', {
      workspacePath: '/tmp/ws',
      environments: [expect.objectContaining({ id: 'env_prod' })],
    });
    expect(useWorkspaceStore.getState().activeEnvironmentId).toBe('env_prod');
  });

  it('updateEnvironment queues a sync change after saving', async () => {
    const state = await freshStore();
    const env = { id: 'env_local', name: 'Local', variables: [] };
    useWorkspaceStore.setState({ environments: [env], activeEnvironmentId: 'env_local' });
    mockBackend(); // sync_resource_change succeeds -> queue drains, stays online

    await state.updateEnvironment({ ...env, name: 'Local v2' });

    // Successful relay drains the queue; failure paths would keep it pending.
    expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      'sync_resource_change',
      expect.objectContaining({ resource_type: 'Environment' }),
    );
    expect(useWorkspaceStore.getState().isOnline).toBe(true);
  });

  it('processSyncQueue marks offline when the relay rejects changes', async () => {
    vi.useFakeTimers();
    try {
      const state = await freshStore();
      vi.mocked(invoke).mockRejectedValue(new Error('offline'));
      state.addToSyncQueue({
        id: 'chg_1',
        resource_type: 'Environment',
        resource_id: 'env_local',
        operation: 'Update',
        data: '{}',
        timestamp: new Date().toISOString(),
      });
      await vi.advanceTimersByTimeAsync(0);
      expect(useWorkspaceStore.getState().isOnline).toBe(false);
      expect(useWorkspaceStore.getState().syncQueue).toHaveLength(1);
      vi.advanceTimersByTime(5000);
    } finally {
      vi.useRealTimers();
    }
  });

  it('loadGlobals stores the fetched bundle', async () => {
    const state = await freshStore();
    vi.mocked(invoke).mockResolvedValue({ variables: { TOKEN: 'abc' } });

    await state.loadGlobals();

    expect(useWorkspaceStore.getState().globals).toEqual({ variables: { TOKEN: 'abc' } });
  });

  it('saveGlobals updates local state before persisting', async () => {
    const state = await freshStore();
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.saveGlobals({ variables: { A: '1' } });

    expect(useWorkspaceStore.getState().globals).toEqual({ variables: { A: '1' } });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith('save_globals', {
      workspacePath: '/tmp/ws',
      globals: { variables: { A: '1' } },
    });
  });

  it('design CRUD flows hit their IPC contracts', async () => {
    const state = await freshStore();
    const created = {
      id: 'spec_1',
      name: 'Billing API',
      content: '',
      format: 'yaml',
      version: 'OpenAPI 3.0',
      last_modified: 'now',
    };

    vi.mocked(invoke).mockResolvedValueOnce(created); // create_design
    await state.createDesign('Billing API', 'yaml');
    expect(useWorkspaceStore.getState().designs).toHaveLength(1);
    expect(useWorkspaceStore.getState().activeDesignId).toBe('spec_1');

    vi.mocked(invoke).mockResolvedValueOnce(undefined); // save_design
    await state.saveDesign({ ...created, content: 'openapi: 3.0.0' });
    expect(useWorkspaceStore.getState().designs[0]?.content).toBe('openapi: 3.0.0');

    vi.mocked(invoke).mockResolvedValueOnce([created]); // list_designs
    await state.loadDesigns();
    expect(useWorkspaceStore.getState().designs).toHaveLength(1);

    vi.mocked(invoke).mockResolvedValueOnce(undefined); // delete_design
    await state.deleteDesign('spec_1');
    expect(useWorkspaceStore.getState().designs).toHaveLength(0);
    expect(useWorkspaceStore.getState().activeDesignId).toBeNull();
  });
});

describe('useWorkspaceStore (workspace lifecycle)', () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
    vi.mocked(open).mockReset();
  });

  it('openWorkspace wires the folder and loads every workspace surface', async () => {
    const state = await freshStore();
    vi.mocked(open).mockResolvedValue('/chosen/path');
    vi.mocked(invoke)
      .mockResolvedValueOnce([]) // load_collections
      .mockResolvedValueOnce([]) // load_environments
      .mockResolvedValueOnce({ variables: {} }); // load_globals

    await state.openWorkspace();

    const after = useWorkspaceStore.getState();
    expect(after.workspacePath).toBe('/chosen/path');
    expect(after.error).toBeNull();
    expect(vi.mocked(invoke).mock.calls.map(([command]) => command)).toEqual([
      'load_collections',
      'load_environments',
      'load_globals',
    ]);
  });

  it('openWorkspace records failures into state instead of throwing', async () => {
    const state = await freshStore();
    vi.mocked(open).mockRejectedValue('dialog crashed');

    await state.openWorkspace();

    expect(useWorkspaceStore.getState().error).toContain('dialog crashed');
  });

  it('importCollection delegates to the backend and reloads', async () => {
    const state = await freshStore();
    vi.mocked(open).mockResolvedValue('/imports/api.json');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.importCollection();

    expect(vi.mocked(invoke)).toHaveBeenCalledWith('import_collection_by_path', {
      collectionPath: '/imports/api.json',
      workspacePath: '/tmp/ws',
    });
  });

  it('exportWorkspace saves the bundle through the backend', async () => {
    const state = await freshStore();
    const { save } = await import('@tauri-apps/plugin-dialog');
    vi.mocked(save).mockResolvedValue('/backups/bundle.json');
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.exportWorkspace();

    expect(vi.mocked(invoke)).toHaveBeenCalledWith('export_workspace', {
      workspacePath: '/tmp/ws',
      exportPath: '/backups/bundle.json',
    });
  });
});
