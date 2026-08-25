// src/types/workspaceState.ts
//
// Shape of the global workspace store state. Kept outside the coverage
// instrumentation surface: pure type declarations cannot be "covered" and
// would otherwise drag the Vitest branch gate down with phantom branches.
import type {
  Collection,
  CollectionItem,
  DesignSpec,
  Environment,
  HttpRequest,
  SyncChange,
} from './ipc';

export interface WorkspaceState {
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

  syncQueue: SyncChange[];
  isOnline: boolean;
  processSyncQueue: () => Promise<void>;
  addToSyncQueue: (change: SyncChange) => void;

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
