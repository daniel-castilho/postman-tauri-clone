// src/store/workspaceStore.ts
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

export interface Collection {
  id: string;
  name: string;
  description?: string;
  items: CollectionItem[];
  variables: Record<string, string>;
}

export type CollectionItem =
  | { Request: HttpRequest }
  | { Folder: { name: string; description?: string; items: CollectionItem[] } };

export interface TestResult {
  name: string;
  passed: boolean;
  error?: string;
}

export interface EnvironmentVariable {
  key: String;
  initial_value: string;
  current_value: string;
  var_type: 'Public' | 'Secret';
  enabled: boolean;
}

export interface Environment {
  id: string;
  name: string;
  variables: EnvironmentVariable[];
}

export interface HttpRequest {
  id: string;
  name: string;
  description?: string;
  method: string;
  url: string;
  headers: any[];
  body: any | null;
  auth: any | null;
  variables: Record<string, string>;
  scripts: {
    preRequest: string;
    tests: string;
  };
  grpc_config?: any;
}

interface WorkspaceState {
  workspacePath: string;
  activeRequest: HttpRequest | null;
  collections: Collection[];
  environments: Environment[];
   activeEnvironmentId: string | null;
  history: HttpRequest[];
  isLoading: boolean;
  error: string | null;
  openTabs: HttpRequest[];
  activeTabId: string | null;
  
  setWorkspacePath: (path: string) => void;
  setActiveRequest: (request: HttpRequest | null) => void;
  setActiveRequestTab: (tabId: string | null) => void;
  addTab: (request: HttpRequest) => void;
  closeTab: (tabId: string) => void;
  setActiveEnvironment: (id: string | null) => void;
  addEnvironment: (env: Environment) => void;
  updateEnvironment: (env: Environment) => void;
  updateRequest: (updatedRequest: HttpRequest) => Promise<void>;
  addCollection: (name: string) => Promise<void>;
  addRequestToCollection: (collectionId: string) => Promise<void>;
  addFolderToCollection: (collectionId: string, name: string) => Promise<void>;
  deleteRequest: (requestId: string) => Promise<void>;
  openWorkspace: () => Promise<void>;
  loadCollections: () => Promise<void>;
  saveCollection: (collection: Collection) => Promise<void>;
  deleteCollection: (id: string) => Promise<void>;
  
  loadEnvironments: () => Promise<void>;
  saveEnvironments: () => Promise<void>;

  globals: { variables: Record<string, string> };
  sessionVariables: Record<string, string>;
  loadGlobals: () => Promise<void>;
  saveGlobals: (globals: { variables: Record<string, string> }) => Promise<void>;
  setSessionVariable: (key: string, value: string) => void;

  addToHistory: (request: HttpRequest) => void;
  clearHistory: () => void;
  duplicateRequest: (requestId: string) => Promise<void>;
  importCollection: () => Promise<void>;
  reorderItems: (collectionId: string, newItems: CollectionItem[]) => Promise<void>;
  exportWorkspace: () => Promise<void>;
  
  syncQueue: any[];
  isOnline: boolean;
  processSyncQueue: () => Promise<void>;
  addToSyncQueue: (change: any) => void;

  designs: DesignSpec[];
  activeDesignId: string | null;
  setActiveDesign: (id: string | null) => void;
  loadDesigns: () => Promise<void>;
  saveDesign: (design: DesignSpec) => Promise<void>;
  createDesign: (name: string, format: string) => Promise<void>;
  deleteDesign: (id: string) => Promise<void>;
  
  sidebarMode: 'Collections' | 'Designs' | 'History';
  setSidebarMode: (mode: 'Collections' | 'Designs' | 'History') => void;
}

export interface DesignSpec {
  id: string;
  name: string;
  content: string;
  format: string;
  version: string;
  last_modified: string;
}

export interface LintIssue {
  line: number;
  message: string;
  severity: 'Error' | 'Warning' | 'Info';
  path: string;
}

export const useWorkspaceStore = create<WorkspaceState>((set: any, get: any) => ({
  workspacePath: '', // O usuário definirá via UI ou FilePicker nativo do desktop
  activeRequest: null,
  sidebarMode: 'Collections',
  setSidebarMode: (mode: any) => set({ sidebarMode: mode }),
  collections: [],
  environments: [{ 
    id: 'env_local', 
    name: 'Local', 
    variables: [{ 
      key: 'BASE_URL', 
      initial_value: 'http://localhost:3000', 
      current_value: 'http://localhost:3000', 
      var_type: 'Public', 
      enabled: true 
    }] 
  }],
  activeEnvironmentId: 'env_local',
  history: [],
  isLoading: false,
  error: null,
  openTabs: [],
  activeTabId: null,
  syncQueue: [],
  isOnline: true,

  addToSyncQueue: (change: any) => {
    set((state: any) => ({ syncQueue: [...state.syncQueue, change] }));
    get().processSyncQueue();
  },

  processSyncQueue: async () => {
    const { syncQueue, isOnline } = get();
    if (syncQueue.length === 0) return;

    try {
      // Tenta processar toda a fila
      for (const change of [...syncQueue]) {
         await invoke('sync_resource_change', change);
         // Remove da fila se teve sucesso
         set((state: any) => ({ syncQueue: state.syncQueue.filter((c: any) => c !== change) }));
      }
      set({ isOnline: true });
    } catch (err) {
      console.warn("Sync temporarily offline, retrying in 5s...");
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
    const tab = get().openTabs.find((t: any) => t.id === tabId);
    set({ activeTabId: tabId, activeRequest: tab || null });
  },

  addTab: (request: HttpRequest) => {
    set((state: any) => {
      const exists = state.openTabs.some((t: any) => t.id === request.id);
      if (exists) {
        return { activeTabId: request.id, activeRequest: request };
      }
      return { 
        openTabs: [...state.openTabs, request],
        activeTabId: request.id,
        activeRequest: request
      };
    });
  },

  closeTab: (tabId: string) => {
    set((state: any) => {
      const newTabs = state.openTabs.filter((t: any) => t.id !== tabId);
      let newActiveId = state.activeTabId;
      let newActiveRequest = state.activeRequest;

      if (state.activeTabId === tabId) {
        newActiveId = newTabs.length > 0 ? newTabs[newTabs.length - 1].id : null;
        newActiveRequest = newTabs.length > 0 ? newTabs[newTabs.length - 1] : null;
      }

      return { 
        openTabs: newTabs, 
        activeTabId: newActiveId,
        activeRequest: newActiveRequest
      };
    });
  },

  setActiveEnvironment: (id: string | null) => set({ activeEnvironmentId: id }),

  addEnvironment: async (env: Environment) => {
    set((state: any) => ({ 
      environments: [...state.environments, env],
      activeEnvironmentId: state.activeEnvironmentId || env.id
    }));
    await get().saveEnvironments();
  },

  updateEnvironment: async (env: Environment) => {
    set((state: any) => ({
      environments: state.environments.map((e: any) => e.id === env.id ? env : e)
    }));
    await get().saveEnvironments();
    
    // NOVO: Usa a fila robusta p/ Sync
    get().addToSyncQueue({
      resourceType: 'Environment',
      resourceId: env.id,
      operation: 'Update',
      data: JSON.stringify(env)
    });
  },

  addToHistory: (request: HttpRequest) => set((state: any) => ({
    history: [request, ...state.history].slice(0, 50) // Limite de 50 itens
  })),

  clearHistory: () => set({ history: [] }),

  updateRequest: async (updatedRequest: HttpRequest) => {
    const { collections, saveCollection } = get();
    let targetCollection: Collection | null = null;
    let newCollections = [...collections];

    // Helper recursivo para buscar e atualizar o request
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

    for (let c of newCollections) {
      if (updateInItems(c.items)) {
        targetCollection = c;
        break;
      }
    }

    if (targetCollection) {
      // Atualiza estado local primeiro pra ser otimista
      set({ collections: newCollections, activeRequest: updatedRequest });
      // Grava no disco
      await saveCollection(targetCollection);

      // NOVO: Usa a fila robusta p/ Sync
      get().addToSyncQueue({
        resourceType: 'Request',
        resourceId: updatedRequest.id,
        operation: 'Update',
        data: JSON.stringify(updatedRequest)
      });
    }
  },

  addCollection: async (name: string) => {
    const newCollection: Collection = {
      id: `col_${Date.now()}`,
      name: name,
      items: [],
      variables: {}
    };
    await get().saveCollection(newCollection);
    await get().loadCollections();
  },

  importCollection: async () => {
    const { workspacePath, loadCollections } = get();
    const selected = await open({
      multiple: false,
      filters: [{ name: 'JSON', extensions: ['json'] }]
    });

    if (selected && typeof selected === 'string') {
      try {
        await invoke('import_collection_by_path', { 
          collectionPath: selected, 
          workspacePath 
        });
        await loadCollections();
      } catch (err) {
        console.error("Import failed:", err);
      }
    }
  },

  duplicateRequest: async (requestId: string) => {
    const { collections, saveCollection } = get();
    let originalReq: HttpRequest | null = null;
    let targetCollection: Collection | null = null;

    // Helper para achar e clonar
    const findAndClone = (items: CollectionItem[]): CollectionItem[] => {
      const newItems: CollectionItem[] = [];
      for (const item of items) {
        newItems.push(item);
        if ('Request' in item && item.Request.id === requestId) {
          originalReq = { ...item.Request, id: `req_${Date.now()}`, name: `${item.Request.name} (Copy)` };
          newItems.push({ Request: originalReq });
        } else if ('Folder' in item) {
          item.Folder.items = findAndClone(item.Folder.items);
        }
      }
      return newItems;
    };

    const newCollections = collections.map((col: any) => {
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
    const targetColl = collections.find((c: any) => c.id === collectionId);
    if (!targetColl) return;

    const updatedCollection = { ...targetColl, items: newItems };
    const newCollections = collections.map((c: any) => c.id === collectionId ? updatedCollection : c);
    
    set({ collections: newCollections });
    await saveCollection(updatedCollection);
  },

  exportWorkspace: async () => {
    const { workspacePath } = get();
    const filePath = await save({
      filters: [{ name: 'JSON', extensions: ['json'] }],
      defaultPath: 'workspace-bundle.json'
    });

    if (filePath) {
      try {
        await invoke('export_workspace', { workspacePath, exportPath: filePath });
        console.log("Workspace exported successfully to", filePath);
      } catch (err) {
        console.error("Export failed:", err);
      }
    }
  },

  addRequestToCollection: async (collectionId: string) => {
    const { collections, saveCollection, setActiveRequest } = get();
    const targetColl = collections.find((c: any) => c.id === collectionId);
    if (!targetColl) return;

    const defaultRequest: HttpRequest = {
      id: `req_${Date.now()}`,
      name: "New Request",
      method: "GET",
      url: "https://api.example.com",
      headers: [],
      body: null,
      auth: { type: 'NoAuth' },
      variables: {},
      scripts: {
        preRequest: "",
        tests: ""
      }
    };

    const updatedCollection = {
      ...targetColl,
      items: [...targetColl.items, { Request: defaultRequest }]
    };

    await saveCollection(updatedCollection);
    setActiveRequest(defaultRequest);
  },

  addFolderToCollection: async (collectionId: string, name: string) => {
    const { collections, saveCollection } = get();
    const targetColl = collections.find((c: any) => c.id === collectionId);
    if (!targetColl) return;

    const updatedCollection = {
      ...targetColl,
      items: [...targetColl.items, { Folder: { name, description: "", items: [] } }]
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
      return items.filter(item => {
        if ('Request' in item && item.Request.id === requestId) return false;
        return true;
      }).map(item => {
        if ('Folder' in item) {
          return { ...item, Folder: { ...item.Folder, items: deleteRecursive(item.Folder.items) } };
        }
        return item;
      });
    };

    const updatedCollection = {
      ...targetCollection,
      items: deleteRecursive(targetCollection.items)
    };

    const newCollections = collections.map((c: any) => c.id === updatedCollection.id ? updatedCollection : c);
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
        title: 'Selecione a pasta do Workspace',
      });
      
      if (selectedPath && typeof selectedPath === 'string') {
        set({ workspacePath: selectedPath });
        await get().loadCollections();
        await get().loadEnvironments();
        await get().loadGlobals();
      }
    } catch (error: any) {
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
    } catch (error: any) {
      set({ error: JSON.stringify(error), isLoading: false });
    }
  },

  saveCollection: async (collection: Collection) => {
    const { workspacePath, loadCollections } = get();
    if (!workspacePath) return;

    set({ isLoading: true, error: null });
    try {
      await invoke('save_collection', { workspacePath, collection });
      await loadCollections(); // Reload para refletir as mudanças do disco
    } catch (error: any) {
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
    } catch (error: any) {
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
    } catch (error: any) {
      console.error("Failed to load environments:", error);
    }
  },

  saveEnvironments: async () => {
    const { workspacePath, environments } = get();
    if (!workspacePath) return;

    try {
      await invoke('save_environments', { workspacePath, environments });
    } catch (error: any) {
      console.error("Failed to save environments:", error);
    }
  },

  globals: { variables: {} },
  sessionVariables: {},

  loadGlobals: async () => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      const globals: any = await invoke('load_globals', { workspacePath });
      set({ globals: globals || { variables: {} } });
    } catch (err) {
      console.error("Failed to load globals:", err);
    }
  },

  saveGlobals: async (globals: any) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    set({ globals });
    try {
      await invoke('save_globals', { workspacePath, globals });
    } catch (err) {
      console.error("Failed to save globals:", err);
    }
  },

  setSessionVariable: (key: string, value: string) => {
    set((state: any) => ({
      sessionVariables: { ...state.sessionVariables, [key]: value }
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
      console.error("Failed to load designs:", err);
    }
  },

  saveDesign: async (design: DesignSpec) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      await invoke('save_design', { workspacePath, design });
      set((state: any) => ({
        designs: state.designs.map((d: any) => d.id === design.id ? design : d)
      }));
    } catch (err) {
      console.error("Failed to save design:", err);
    }
  },

  createDesign: async (name: string, format: string) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      const newDesign: DesignSpec = await invoke('create_design', { workspacePath, name, format });
      set((state: any) => ({
        designs: [...state.designs, newDesign],
        activeDesignId: newDesign.id
      }));
    } catch (err) {
      console.error("Failed to create design:", err);
    }
  },

  deleteDesign: async (id: string) => {
    const { workspacePath } = get();
    if (!workspacePath) return;
    try {
      await invoke('delete_design', { workspacePath, designId: id });
      set((state: any) => ({
        designs: state.designs.filter((d: any) => d.id !== id),
        activeDesignId: state.activeDesignId === id ? null : state.activeDesignId
      }));
    } catch (err) {
      console.error("Failed to delete design:", err);
    }
  }
}));

// Listener Global para Sincronização em Tempo Real
listen('sync-change', (event: any) => {
  const change = event.payload;
  const store = useWorkspaceStore.getState();

  console.log("🔄 Sync Change Received:", change);

  if (change.resource_type === 'Request' && change.operation === 'Update') {
    const updatedReq = JSON.parse(change.data);
    
    // Evita loop se for a mesma mudança que nós enviamos (melhoria futura: IDs de cliente)
    if (store.activeRequest?.id === updatedReq.id) {
       // Se for o mesmo que estamos editando, atualizamos a UI mas com cuidado
       // useWorkspaceStore.setState({ activeRequest: updatedReq });
    }

    // Atualiza na árvore de coleções
    const newCollections = [...store.collections];
    let found = false;

    const updateRecursive = (items: any[]) => {
      for (let i = 0; i < items.length; i++) {
        if (items[i].Request?.id === updatedReq.id) {
          items[i] = { Request: updatedReq };
          found = true;
          return;
        }
        if (items[i].Folder) updateRecursive(items[i].Folder.items);
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
    const updatedEnv = JSON.parse(change.data);
    const existingEnv = store.environments.find((e: any) => e.id === updatedEnv.id);
    
    if (existingEnv) {
       // Mescla mantendo os valores locais (current_value)
       updatedEnv.variables = updatedEnv.variables.map((v: any) => {
          const localVar = existingEnv.variables.find((lv: any) => lv.key === v.key);
          return { 
            ...v, 
            current_value: localVar ? localVar.current_value : v.initial_value 
          };
       });
    }

    const newEnvs = store.environments.map((e: any) => e.id === updatedEnv.id ? updatedEnv : e);
    useWorkspaceStore.setState({ environments: newEnvs });
  }
});

