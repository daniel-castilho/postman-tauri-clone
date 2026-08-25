import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { CollectionRunner } from '../components/CollectionRunner';
import { useWorkspaceStore } from '../store/workspaceStore';
import type { CollectionItem, CollectionRunReport, Environment } from '../types/ipc';

// Depending on module graph ordering the real Tauri API may leak into this
// file; stub the internals so its top-level listeners stay harmless.
(globalThis as Record<string, unknown>).__TAURI_INTERNALS__ ??= {
  transformCallback: (callback: unknown) => callback,
  invoke: async () => undefined,
  metadata: { currentWindow: { label: 'test' }, currentWebview: { label: 'test' } },
};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import('@tauri-apps/api/core');

const environment: Environment = { id: 'env_local', name: 'Local', variables: [] };

const items: CollectionItem[] = [
  {
    Request: {
      id: 'req_1',
      name: 'Ping',
      description: null,
      method: 'GET',
      url: 'https://api.example.com/ping',
      headers: [],
      body: null,
      auth: { type: 'NoAuth' },
      variables: {},
      scripts: null,
      grpc_config: null,
    },
  },
];

const runReport: CollectionRunReport = {
  totalRequests: 1,
  totalTests: 2,
  passedTests: 1,
  results: [
    {
      requestName: 'Ping',
      status: 500,
      timeMs: 8,
      tests: [
        { name: 'reachable', passed: true, error: null },
        { name: 'fast', passed: false, error: 'too slow' },
      ],
    },
  ],
};

describe('CollectionRunner', () => {
  let createObjectURLSpy: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    vi.clearAllMocks();
    createObjectURLSpy = vi.fn(() => 'blob:mock');
    vi.stubGlobal(
      'URL',
      Object.assign(URL, { createObjectURL: createObjectURLSpy, revokeObjectURL: vi.fn() }),
    );
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined);

    const store = useWorkspaceStore;
    store.setState({
      workspacePath: '/tmp/ws',
      globals: { variables: {} },
      sessionVariables: {},
    });
    vi.mocked(invoke).mockImplementation(async (command: string) => {
      if (command === 'run_collection') return runReport;
      if (command === 'render_run_report') return '# rendered report';
      return undefined;
    });
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('runs the collection and exposes HTML/Markdown report exports', async () => {
    render(<CollectionRunner items={items} environment={environment} onClose={() => undefined} />);

    fireEvent.click(screen.getByText('Start Run'));

    await waitFor(() => expect(screen.getByText('Close Report')).toBeTruthy());
    // Run results are visible before exporting.
    expect(screen.getByText('Ping')).toBeTruthy();

    const commands = () => vi.mocked(invoke).mock.calls.map(([command]) => command);
    expect(commands()).toContain('run_collection');

    fireEvent.click(screen.getByText('Export HTML'));
    await waitFor(() =>
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        'render_run_report',
        expect.objectContaining({ format: 'Html', collectionName: 'Collection Run' }),
      ),
    );

    fireEvent.click(screen.getByText('Export Markdown'));
    await waitFor(() =>
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        'render_run_report',
        expect.objectContaining({ format: 'Markdown' }),
      ),
    );

    expect(createObjectURLSpy).toHaveBeenCalledTimes(2);
    const [htmlBlob, mdBlob] = createObjectURLSpy.mock.calls.map((call) => call[0] as Blob);
    expect(htmlBlob.type).toBe('text/html');
    expect(mdBlob.type).toBe('text/md');
  });

  it('surfaces backend failures instead of opening the report modal', async () => {
    vi.mocked(invoke).mockReset();
    vi.mocked(invoke).mockRejectedValue('boom');

    render(<CollectionRunner items={items} environment={environment} onClose={() => undefined} />);
    fireEvent.click(screen.getByText('Start Run'));

    await waitFor(() => expect(screen.getByText(/boom/)).toBeTruthy());
    expect(screen.queryByText('Close Report')).toBeNull();
  });
});
