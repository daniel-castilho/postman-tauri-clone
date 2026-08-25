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

function nestedCollection(): Collection {
  return {
    id: 'col_1',
    name: 'Nested',
    description: null,
    items: [
      {
        Folder: {
          name: 'Group',
          description: '',
          items: [{ Request: sampleRequest({ id: 'req_deep' }) }],
        },
      },
      { Request: sampleRequest() },
    ],
    variables: {},
  };
}

function noWorkspace(): void {
  useWorkspaceStore.setState({ workspacePath: '' });
}

describe('useWorkspaceStore (branch completion)', () => {
  beforeEach(() => {
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
    vi.mocked(invoke).mockReset();
  });

  it('processSyncQueue returns early when the queue is empty', async () => {
    await useWorkspaceStore.getState().processSyncQueue();
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it('setActiveRequestTab with an unknown id clears the selection', async () => {
    const state = useWorkspaceStore.getState();
    state.addTab(sampleRequest());
    state.setActiveRequestTab('ghost');
    expect(useWorkspaceStore.getState().activeRequest).toBeNull();
    expect(useWorkspaceStore.getState().activeTabId).toBe('ghost');
  });

  it('updateEnvironment tolerates unknown environment ids', async () => {
    useWorkspaceStore.setState({
      environments: [{ id: 'env_a', name: 'A', variables: [] }],
    });
    vi.mocked(invoke).mockResolvedValue(undefined);

    await useWorkspaceStore
      .getState()
      .updateEnvironment({ id: 'env_missing', name: 'X', variables: [] });

    // The original list survives the map.
    expect(useWorkspaceStore.getState().environments[0]?.id).toBe('env_a');
  });

  it('updateRequest walks into folders to find its target', async () => {
    const collection = nestedCollection();
    useWorkspaceStore.setState({
      collections: [collection],
      activeRequest: sampleRequest({ id: 'req_deep' }),
    });
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'load_collections') return useWorkspaceStore.getState().collections;
      return undefined;
    });

    await useWorkspaceStore
      .getState()
      .updateRequest(sampleRequest({ id: 'req_deep', name: 'Deep updated' }));

    const folderItem = useWorkspaceStore.getState().collections[0]?.items[0];
    if ('Folder' in folderItem!) {
      const inner = folderItem.Folder.items[0];
      expect('Request' in inner && inner.Request.name).toBe('Deep updated');
    } else {
      throw new Error('expected folder item');
    }
  });

  it('deleteRequest prunes requests nested inside folders', async () => {
    useWorkspaceStore.setState({ collections: [nestedCollection()] });
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'load_collections') return useWorkspaceStore.getState().collections;
      return undefined;
    });

    await useWorkspaceStore.getState().deleteRequest('req_deep');

    const items = useWorkspaceStore.getState().collections[0]?.items ?? [];
    const folder = items.find((item) => 'Folder' in item);
    if ('Folder' in folder!) {
      expect(folder.Folder.items).toHaveLength(0);
    } else {
      throw new Error('expected folder item');
    }
  });

  it('mutations bail out gracefully for unknown collection ids', async () => {
    const state = useWorkspaceStore.getState();
    await state.addRequestToCollection('missing');
    await state.addFolderToCollection('missing', 'X');
    await state.deleteRequest('missing');
    await state.duplicateRequest('missing');
    await state.reorderItems('missing', []);
    expect(vi.mocked(invoke)).not.toHaveBeenCalledWith(
      'save_collection',
      expect.anything(),
    );
  });

  it('persistence actions bail out without a workspace path', async () => {
    noWorkspace();
    const state = useWorkspaceStore.getState();

    await state.loadCollections();
    await state.saveCollection({ id: 'col_1', name: 'X', description: null, items: [], variables: {} });
    await state.deleteCollection('col_1');
    await state.loadEnvironments();
    await state.saveEnvironments();
    await state.loadGlobals();
    await state.saveGlobals({ variables: {} });
    await state.loadDesigns();
    await state.saveDesign({
      id: 's1',
      name: 'S',
      content: '',
      format: 'yaml',
      version: 'OpenAPI 3.0',
      last_modified: 'now',
    });
    await state.createDesign('S', 'yaml');
    await state.deleteDesign('s1');

    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
  });

  it('saveDesign keeps unrelated designs and deleteDesign keeps other selections', async () => {
    const state = useWorkspaceStore.getState();
    const existing = {
      id: 'spec_keep',
      name: 'Keep',
      content: 'a',
      format: 'yaml',
      version: 'OpenAPI 3.0',
      last_modified: 'now',
    };
    const other = { ...existing, id: 'spec_other', name: 'Other' };
    useWorkspaceStore.setState({
      designs: [existing, other],
      activeDesignId: 'spec_other',
    });
    vi.mocked(invoke).mockResolvedValue(undefined);

    await state.saveDesign({ ...existing, content: 'b' });
    let designs = useWorkspaceStore.getState().designs;
    expect(designs.find((d) => d.id === 'spec_keep')?.content).toBe('b');

    await state.deleteDesign('spec_keep');
    designs = useWorkspaceStore.getState().designs;
    expect(designs.map((d) => d.id)).toEqual(['spec_other']);
    // Deleting a non-active design preserves the current selection.
    expect(useWorkspaceStore.getState().activeDesignId).toBe('spec_other');
  });
});
