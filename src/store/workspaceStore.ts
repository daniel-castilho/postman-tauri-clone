// src/store/workspaceStore.ts
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

// IPC contract types are auto-generated from the Rust domain models.
// Never redeclare them by hand here (Zero Type-Drift epic).
import type {
  Collection,
  CollectionItem,
  DesignSpec,
  Environment,
  HttpRequest,
  SyncChange,
  GlobalVariables,
} from '../types/ipc';
import type { WorkspaceState } from '../types/workspaceState';

export const useWorkspaceStore = create<WorkspaceState>()((set, get) => ({
  workspacePath: '', // The user will set this via UI or the native desktop FilePicker
  activeRequest: null,
  sidebarMode: 'Collections',
  setSidebarMode: (mode) => set({ sidebarMode: mode }),
  collections: [],
  environments: [
    {
      id: 'env_local',
      name: 'Local',
      variables: [
        {
          key: 'BASE_URL',
          initial_value: 'http://localhost:3000',
          current_value: 'http://localhost:3000',
          var_type: 'Public',
          enabled: true,
        },
      ],
    },
  ],
  activeEnvironmentId: 'env_local',
  history: [],
  isLoading: false,
  error: null,
  openTabs: [],
  activeTabId: null,
  syncQueue: [],
  isOnline: true,

  addToSyncQueue: (change: SyncChange) => {
    set((state) => ({ syncQueue: [...state.syncQueue, change] }));
    get().processSyncQueue();
  },

  processSyncQueue: async () => {
    const { syncQueue } = get();
    if (syncQueue.length === 0) return;

    try {
      // Try to process the whole queue
      for (const change of [...syncQueue]) {
        await invoke('sync_resource_change', change);
        // Remove from the queue on success
        set((state) => ({ syncQueue: state.syncQueue.filter((c) => c !== change) }));
      }
      set({ isOnline: true });
    } catch {
      console.warn('Sync temporarily offline, retrying in 5s...');
      set({ isOnline: false });
      setTimeout(() => {
        set({ isOnline: true });
        get().processSyncQueue();
      }, 5000);
    }
  },

  setWorkspacePath: (path: string) => set({ workspacePath: path }),

  setActiveRequest: (request: HttpRequest | null) => {
    if (request) {
      get().addTab(request);
    } else {
      set({ activeRequest: null, activeTabId: null });
    }
  },

  setActiveRequestTab: (tabId: string | null) => {
    const tab = get().openTabs.find((t) => t.id === tabId);
    set({ activeTabId: tabId, activeRequest: tab || null });
  },

  addTab: (request: HttpRequest) => {
    set((state) => {
      const exists = state.openTabs.some((t) => t.id === request.id);
      if (exists) {
        return { activeTabId: request.id, activeRequest: request };
      }
      return {
        openTabs: [...state.openTabs, request],
        activeTabId: request.id,
        activeRequest: request,
      };
    });
  },

  closeTab: (tabId: string) => {
    set((state) => {
      const newTabs = state.openTabs.filter((t) => t.id !== tabId);
      let newActiveId = state.activeTabId;
      let newActiveRequest = state.activeRequest;

      if (state.activeTabId === tabId) {
        newActiveId = newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null;
        newActiveRequest = newTabs.length > 0 ? newTabs[newTabs.length - 1] : null;
      }

      return {
        openTabs: newTabs,
        activeTabId: newActiveId,
        activeRequest: newActiveRequest,
      };
    });
  },

  setActiveEnvironment: (id: string | null) => set({ activeEnvironmentId: id }),

  addEnvironment: async (env: Environment) => {
    set((state) => ({
      environments: [...state.environments, env],
      activeEnvironmentId: state.activeEnvironmentId || env.id,
    }));
    await get().saveEnvironments();
  },

  updateEnvironment: async (env: Environment) => {
    set((state) => ({
      environments: state.environments.map((e) => (e.id === env.id ? env : e)),
    }));
    await get().saveEnvironments();

    // NEW: use the robust sync queue
    get().addToSyncQueue({
      id: crypto.randomUUID(),
      resource_type: 'Environment',
      resource_id: env.id,
      operation: 'Update',
      data: JSON.stringify(env),
      timestamp: new Date().toISOString(),
    });
  },

  addToHistory: (request: HttpRequest) =>
    set((state) => ({
      history: [request, ...state.history].slice(0, 50), // Cap history at 50 items
    })),

  clearHistory: () => set({ history: [] }),

  updateRequest: async (updatedRequest: HttpRequest) => {
    const { collections, saveCollection } = get();
    let targetCollection: Collection | null = null;
    const newCollections = [...collections];

    // Recursive helper to find and update the request
    const updateInItems = (items: CollectionItem[]): boolean => {
      for (let i = 0; i < items.length; i++) {
        const item = items[i];
        if ('Request' in item && item.Request.id === updatedRequest.id) {
          items[i] = { Request: updatedRequest };
          return true;
        } else if ('Folder' in item) {
          if (updateInItems(item.Folder.items)) return true;
        }
      }
      return false;
    };

    for (const c of newCollections) {
      if (updateInItems(c.items)) {
        targetCollection = c;
        break;
      }
    }

    if (targetCollection) {
      // Update local state first to be optimistic
      set({ collections: newCollections, activeRequest: updatedRequest });
      // Persist to disk
      await saveCollection(targetCollection);

      // NEW: use the robust sync queue
      get().addToSyncQueue({
        id: crypto.randomUUID(),
        resource_type: 'Request',
        resource_id: updatedRequest.id,
        operation: 'Update',
        data: JSON.stringify(updatedRequest),
        timestamp: new Date().toISOString(),
      });
    }
  },

  addCollection: async (name: string) => {
    const newCollection: Collection = {
      id: `col_${Date.now()}`,
      name: name,
      description: null,
      items: [],
      variables: {},
    };
    await get().saveCollection(newCollection);
    await get().loadCollections();
  },

  importCollection: async () => {
    const { workspacePath, loadCollections } = get();
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    });

    if (selected && typeof selected === 'string') {
      try {
        await invoke('import_collection_by_path', {
          collectionPath: selected,
          workspacePath,
        });
        await loadCollections();
      } catch (err) {
        console.error('Import failed:', err);
      }
    }
  },

  duplicateRequest: async (requestId: string) => {
    const { collections, saveCollection } = get();
    let originalReq: HttpRequest | null = null;
    let targetCollection: Collection | null = null;

    // Helper to find and clone
    const findAndClone = (items: CollectionItem[]): CollectionItem[] => {
      const newItems: CollectionItem[] = [];
      for (const item of items) {
        newItems.push(item);
        if ('Request' in item && item.Request.id === requestId) {
          originalReq = {
            ...item.Request,
            id: `req_${Date.now()}`,
            name: `${item.Request.name} (Copy)`,
          };
          newItems.push({ Request: originalReq });
        } else if ('Folder' in item) {
          item.Folder.items = findAndClone(item.Folder.items);
        }
      }
      return newItems;
    };

    const newCollections = collections.map((col) => {
      const updatedItems = findAndClone(col.items);
      if (originalReq) {
        targetCollection = { ...col, items: updatedItems };
        return targetCollection;
      }
      return col;
    });

    if (targetCollection) {
      set({ collections: newCollections });
      await saveCollection(targetCollection);
    }
  },

  reorderItems: async (collectionId: string, newItems: CollectionItem[]) => {
    const { collections, saveCollection } = get();
    const targetColl = collections.find((c) => c.id === collectionId);
    if (!targetColl) return;

    const updatedCollection = { ...targetColl, items: newItems };
    const newCollections = collections.map((c) => (c.id === collectionId ? updatedCollection : c));

    set({ collections: newCollections });
    await saveCollection(updatedCollection);
  },

  exportWorkspace: async () => {
    const { workspacePath } = get();
    const filePath = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'workspace-bundle.json',
    });

    if (filePath) {
      try {
        await invoke('export_workspace', { workspacePath, exportPath: filePath });
        console.log('Workspace exported successfully to', filePath);
      } catch (err) {
        console.error('Export failed:', err);
      }
    }
  },

  addRequestToCollection: async (collectionId: string) => {
    const { collections, saveCollection, setActiveRequest } = get();
    const targetColl = collections.find((c) => c.id === collectionId);
    if (!targetColl) return;

    const defaultRequest: HttpRequest = {
      id: `req_${Date.now()}`,
      name: 'New Request',
      description: null,
      method: 'GET',
      url: 'https://api.example.com',
      headers: [],
      body: null,
      auth: { type: 'NoAuth' },
      variables: {},
      scripts: {
        preRequest: '',
        tests: '',
      },
      grpc_config: null,
    };

    const updatedCollection = {
      ...targetColl,
      items: [...targetColl.items, { Request: defaultRequest }],
    };

    await saveCollection(updatedCollection);
    setActiveRequest(defaultRequest);
  },

  addFolderToCollection: async (collectionId: string, name: string) => {
    const { collections, saveCollection } = get();
    const targetColl = collections.find((c) => c.id === collectionId);
    if (!targetColl) return;

    const updatedCollection = {
      ...targetColl,
      items: [...targetColl.items, { Folder: { name, description: '', items: [] } }],
    };

    await saveCollection(updatedCollection);
  },

  deleteRequest: async (requestId: string) => {
    const { collections, saveCollection, activeRequest, setActiveRequest } = get();
    let targetCollection: Collection | undefined;

    for (const col of collections) {
      let found = false;
      const searchRecursive = (items: CollectionItem[]) => {
        for (const item of items) {
          if ('Folder' in item) searchRecursive(item.Folder.items);
          else if (item.Request.id === requestId) found = true;
        }
      };
      searchRecursive(col.items);
      if (found) {
        targetCollection = col;
        break;
      }
    }

    if (!targetCollection) return;

    const deleteRecursive = (items: CollectionItem[]): CollectionItem[] => {
      return items
        .filter((item) => {
          if ('Request' in item && item.Request.id === requestId) return false;
          return true;
        })
        .map((item) => {
          if ('Folder' in item) {
            return {
              ...item,
              Folder: { ...item.Folder, items: deleteRecursive(item.Folder.items) },
            };
          }
          return item;
        });
    };

    const updatedCollection = {
      ...targetCollection,
      items: deleteRecursive(targetCollection.items),
    };

    const newCollections = collections.map((c) =>
      c.id === updatedCollection.id ? updatedCollection : c,
    );
    set({ collections: newCollections });
    await saveCollection(updatedCollection);

    if (activeRequest?.id === requestId) {
      setActiveRequest(null);
    }
  },

  openWorkspace: async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: 'Select workspace folder',
      });

      if (selectedPath && typeof selectedPath === 'string') {
        set({ workspacePath: selectedPath });
        await get().loadCollections();
        await get().loadEnvironments();
        await get().loadGlobals();

        // Arm the real-time watcher for this folder (debt #3).
        void invoke('start_workspace_watch', { workspacePath: selectedPath }).catch(
          () => undefined,
        );
      }
    } catch (error) {
      set({ error: JSON.stringify(error) });
    }
  },

  loadCollections: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;

    set({ isLoading: true, error: null });
    try {
      const collections: Collection[] = await invoke('load_collections', { workspacePath });
      set({ collections, isLoading: false });
    } catch (error) {
      set({ error: JSON.stringify(error), isLoading: false });
    }
  },

  saveCollection: async (collection: Collection) => {
    const { workspacePath, loadCollections } = get();
    if (!workspacePath) return;

    set({ isLoading: true, error: null });
    try {
      await invoke('save_collection', { workspacePath, collection });
      await loadCollections(); // Reload to reflect disk changes
    } catch (error) {
      set({ error: JSON.stringify(error), isLoading: false });
    }
  },

  deleteCollection: async (id: string) => {
    const { workspacePath, loadCollections } = get();
    if (!workspacePath) return;

    set({ isLoading: true, error: null });
    try {
      await invoke('delete_collection', { workspacePath, collectionId: id });
      await loadCollections();
    } catch (error) {
      set({ error: JSON.stringify(error), isLoading: false });
    }
  },

  loadEnvironments: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;

    try {
      const environments: Environment[] = await invoke('load_environments', { workspacePath });
      set({ environments });
      if (environments.length > 0 && !get().activeEnvironmentId) {
        set({ activeEnvironmentId: environments[0].id });
      }
    } catch (error) {
      console.error('Failed to load environments:', error);
    }
  },

  saveEnvironments: async () => {
    const { workspacePath, environments } = get();
    if (!workspacePath) return;

    try {
      await invoke('save_environments', { workspacePath, environments });
    } catch (error) {
      console.error('Failed to save environments:', error);
    }
  },

  globals: { variables: {} },
  sessionVariables: {},

  loadGlobals: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      const globals = await invoke<GlobalVariables>('load_globals', { workspacePath });
      set({ globals: globals || { variables: {} } });
    } catch (err) {
      console.error('Failed to load globals:', err);
    }
  },

  saveGlobals: async (globals: GlobalVariables) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    set({ globals });
    try {
      await invoke('save_globals', { workspacePath, globals });
    } catch (err) {
      console.error('Failed to save globals:', err);
    }
  },

  setSessionVariable: (key: string, value: string) => {
    set((state) => ({
      sessionVariables: { ...state.sessionVariables, [key]: value },
    }));
  },

  designs: [],
  activeDesignId: null,

  setActiveDesign: (id: string | null) => set({ activeDesignId: id }),

  loadDesigns: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      const designs: DesignSpec[] = await invoke('list_designs', { workspacePath });
      set({ designs });
    } catch (err) {
      console.error('Failed to load designs:', err);
    }
  },

  saveDesign: async (design: DesignSpec) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      await invoke('save_design', { workspacePath, design });
      set((state) => ({
        designs: state.designs.map((d) => (d.id === design.id ? design : d)),
      }));
    } catch (err) {
      console.error('Failed to save design:', err);
    }
  },

  createDesign: async (name: string, format: string) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      const newDesign: DesignSpec = await invoke('create_design', { workspacePath, name, format });
      set((state) => ({
        designs: [...state.designs, newDesign],
        activeDesignId: newDesign.id,
      }));
    } catch (err) {
      console.error('Failed to create design:', err);
    }
  },

  deleteDesign: async (id: string) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      await invoke('delete_design', { workspacePath, designId: id });
      set((state) => ({
        designs: state.designs.filter((d) => d.id !== id),
        activeDesignId: state.activeDesignId === id ? null : state.activeDesignId,
      }));
    } catch (err) {
      console.error('Failed to delete design:', err);
    }
  },
}));

// Global listener for real-time synchronization
listen<SyncChange>('sync-change', (event) => {
  const change = event.payload;
  const store = useWorkspaceStore.getState();

  if (change.resource_type === 'Request' && change.operation === 'Update') {
    const updatedReq = JSON.parse(change.data);

    // Avoid a loop if this is the same change we sent (future improvement: client IDs)
    if (store.activeRequest?.id === updatedReq.id) {
      // If it is the one being edited, update the UI carefully
      // useWorkspaceStore.setState({ activeRequest: updatedReq });
    }

    // Update the collection tree
    const newCollections = [...store.collections];
    let found = false;

    const updateRecursive = (items: CollectionItem[]) => {
      for (let i = 0; i < items.length; i++) {
        const entry = items[i];
        if ('Request' in entry && entry.Request.id === updatedReq.id) {
          items[i] = { Request: updatedReq };
          found = true;
          return;
        }
        if ('Folder' in entry) updateRecursive(entry.Folder.items);
      }
    };

    for (const col of newCollections) {
      updateRecursive(col.items);
      if (found) break;
    }

    if (found) {
      useWorkspaceStore.setState({ collections: newCollections });
    }
  }

  if (change.resource_type === 'Environment' && change.operation === 'Update') {
    const updatedEnv = JSON.parse(change.data) as Environment;
    const existingEnv = store.environments.find((e) => e.id === updatedEnv.id);

    if (existingEnv) {
      // Merge while keeping local values (current_value)
      updatedEnv.variables = updatedEnv.variables.map((v) => {
        const localVar = existingEnv.variables.find((lv) => lv.key === v.key);
        return {
          ...v,
          current_value: localVar ? localVar.current_value : v.initial_value,
        };
      });
    }

    const newEnvs = store.environments.map((e) => (e.id === updatedEnv.id ? updatedEnv : e));
    useWorkspaceStore.setState({ environments: newEnvs });
  }
});

// Real-time workspace watching (debt #3): the backend debounces filesystem
// events and notifies us here; affected surfaces are reloaded quietly while
// local UI state (open tabs, drafts) stays untouched.
listen<{ paths: string[] }>('workspace-changed', () => {
  const state = useWorkspaceStore.getState();
  if (!state.workspacePath || state.isLoading) return;
  void state.loadCollections();
  void state.loadEnvironments();
  void state.loadGlobals();
  void state.loadDesigns();
});
