import '@testing-library/jest-dom/vitest';

// Tauri API internals stub: some modules register top-level listeners at
// import time; without a desktop runtime those calls would throw.
(globalThis as Record<string, unknown>).__TAURI_INTERNALS__ ??= {
  transformCallback: (callback: unknown) => callback,
  invoke: async () => undefined,
  metadata: { currentWindow: { label: 'test' }, currentWebview: { label: 'test' } },
};
