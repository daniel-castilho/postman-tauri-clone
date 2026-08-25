import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useWorkspaceStore } from './store/workspaceStore';
import type {
  Auth,
  CollectionItem,
  Environment,
  HttpMethod,
  HttpResponse,
  FormField,
  Body,
  SendRequestOutput,
  TestResult,
} from './types/ipc';
import { open as openFile } from '@tauri-apps/plugin-dialog';
import { WorkspaceSelector } from './components/WorkspaceSelector';
import { Sidebar } from './components/Sidebar';
import { CollectionRunner } from './components/CollectionRunner';
import { WorkspaceSettings } from './components/WorkspaceSettings';
import { CommandPalette } from './components/CommandPalette';
import { TabBar } from './components/TabBar';
import { WebSocketDebugger } from './components/WebSocketDebugger';
import { GlobalsModal } from './components/GlobalsModal';
import { LoadTestingPanel } from './components/LoadTestingPanel';
import { DesignPanel } from './components/DesignPanel';
import {
  Save,
  Settings,
  Terminal,
  Copy,
  Check,
  Sun,
  Moon,
  Sparkles,
  BrainCircuit,
  Code2,
  Globe,
  Users,
} from 'lucide-react';
import { Toaster, toast } from 'sonner';
import { motion, AnimatePresence } from 'framer-motion';
import { useShortcuts } from './hooks/useShortcuts';
import './App.css';

function App() {
  const {
    workspacePath,
    activeRequest,
    updateRequest,
    environments,
    activeEnvironmentId,
    setActiveEnvironment,
    updateEnvironment,
    addToHistory,
    activeTabId,
    globals,
    sessionVariables,
    saveGlobals,
    sidebarMode,
  } = useWorkspaceStore();
  const [runnerItems, setRunnerItems] = useState<CollectionItem[] | null>(null);
  const [theme, setTheme] = useState<'light' | 'dark'>(
    () => (localStorage.getItem('app-theme') as 'light' | 'dark') || 'dark',
  );
  const [showWorkspaceSettings, setShowWorkspaceSettings] = useState(false);
  const [showCommandPalette, setShowCommandPalette] = useState(false);

  useShortcuts(setShowCommandPalette);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('app-theme', theme);
  }, [theme]);

  useEffect(() => {
    const handler = (event: Event) =>
      setRunnerItems((event as CustomEvent<CollectionItem[]>).detail);
    window.addEventListener('open-runner', handler);
    return () => window.removeEventListener('open-runner', handler);
  }, []);
  const activeEnvironment = environments?.find((e) => e.id === activeEnvironmentId) || null;
  const [showEnvModal, setShowEnvModal] = useState(false);
  const [showGlobalsModal, setShowGlobalsModal] = useState(false);
  const [showCurlModal, setShowCurlModal] = useState(false);
  const [copiedCurl, setCopiedCurl] = useState(false);
  const [responseMode, setResponseMode] = useState<'Pretty' | 'Raw'>('Pretty');

  const [url, setUrl] = useState('https://jsonplaceholder.typicode.com/users/1');
  const [method, setMethod] = useState<HttpMethod>('GET');
  const [headers, setHeaders] = useState<{ key: string; value: string; enabled: boolean }[]>([]);
  const [params, setParams] = useState<{ key: string; value: string; enabled: boolean }[]>([]);
  const [body, setBody] = useState<string>('');
  const [bodyMode, setBodyMode] = useState<'Raw' | 'FormData' | 'GraphQL' | 'None'>('Raw');
  const [formData, setFormData] = useState<
    { key: string; value: string; file: string | null; enabled: boolean }[]
  >([]);
  const [gqlQuery, setGqlQuery] = useState('');
  const [gqlVariables, setGqlVariables] = useState('');
  const [auth, setAuth] = useState<Auth | null>(null);
  const [activeConfigTab, setActiveConfigTab] = useState<
    'Params' | 'Headers' | 'Body' | 'Auth' | 'Scripts' | 'Docs' | 'LoadTest'
  >('Body');
  const [activeResponseTab, setActiveResponseTab] = useState<
    'Body' | 'Headers' | 'Preview' | 'Test Results' | 'Console'
  >('Body');
  const [activeCookies, setActiveCookies] = useState<string>('');
  const [showCookies, setShowCookies] = useState(false);
  const [protoPath, setProtoPath] = useState('');
  const [grpcService, setGrpcService] = useState('');
  const [grpcMethod, setGrpcMethod] = useState('');
  const [preRequestScript, setPreRequestScript] = useState('');
  const [testScript, setTestScript] = useState('');
  const [aiLoading, setAiLoading] = useState(false);
  const [aiExplanation, setAiExplanation] = useState('');
  const [showCodeModal, setShowCodeModal] = useState(false);
  const [generatedCode, setGeneratedCode] = useState('');
  const [codeTarget, setCodeTarget] = useState<'fetch' | 'node'>('fetch');

  const [response, setResponse] = useState<HttpResponse | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (activeRequest) {
      setUrl(activeRequest.url);
      setMethod(activeRequest.method);
      setHeaders(activeRequest.headers?.length ? activeRequest.headers : []);

      try {
        const urlObj = new URL(activeRequest.url);
        const parsedParams: { key: string; value: string; enabled: boolean }[] = [];
        urlObj.searchParams.forEach((val, key) => {
          parsedParams.push({ key, value: val, enabled: true });
        });
        setParams(parsedParams);
      } catch {
        // Best effort: failures here must not break the surrounding flow.
      }

      let initialBody = '';
      let initialMode: 'Raw' | 'FormData' | 'GraphQL' | 'None' = 'None';
      let initialFormData: FormField[] = [];

      if (activeRequest.body) {
        if ('Raw' in activeRequest.body) {
          initialBody = activeRequest.body.Raw[0] || '';
          initialMode = 'Raw';
        } else if ('FormData' in activeRequest.body) {
          initialFormData = activeRequest.body.FormData;
          initialMode = 'FormData';
        } else if ('GraphQL' in activeRequest.body) {
          setGqlQuery(activeRequest.body.GraphQL.query || '');
          setGqlVariables(activeRequest.body.GraphQL.variables || '');
          initialMode = 'GraphQL';
        }
      } else if (typeof activeRequest.body === 'string') {
        initialBody = activeRequest.body;
        initialMode = 'Raw';
      }

      setBody(initialBody);
      setBodyMode(initialMode);
      setFormData(initialFormData || []);
      setAuth(activeRequest.auth);
      setPreRequestScript(activeRequest.scripts?.preRequest || '');
      setTestScript(activeRequest.scripts?.tests || '');

      if (activeRequest.grpc_config) {
        setProtoPath(activeRequest.grpc_config.proto_path || '');
        setGrpcService(activeRequest.grpc_config.service || '');
        setGrpcMethod(activeRequest.grpc_config.method || '');
      } else {
        setProtoPath('');
        setGrpcService('');
        setGrpcMethod('');
      }

      setResponse(null);
    }
  }, [activeRequest]);

  async function handleSend() {
    setLoading(true);
    // No response panel content yet; previous code stored an unused string here.
    setResponse(null);

    try {
      let backendBody: Body | null = null;
      if (bodyMode === 'Raw' && body) {
        backendBody = { Raw: [body, 'Json'] } as Body;
      } else if (bodyMode === 'FormData') {
        backendBody = { FormData: formData.filter((f) => f.key.trim() !== '') };
      } else if (bodyMode === 'GraphQL') {
        backendBody = { GraphQL: { query: gqlQuery, variables: gqlVariables } };
      }

      const activeEnv = environments?.find((e) => e.id === activeEnvironmentId) || {
        id: 'env_default',
        name: 'No Environment',
        variables: {},
      };

      const output = await invoke<SendRequestOutput>('send_request', {
        request: {
          id: activeRequest?.id || `hist_${Date.now()}`,
          name: activeRequest?.name || url,
          description: activeRequest?.description || null,
          method: method,
          url: url,
          headers: headers.filter((h) => h.key.trim() !== ''),
          body: backendBody,
          auth: auth,
          variables: {},
          scripts: {
            preRequest: preRequestScript,
            tests: testScript,
          },
          grpc_config:
            method === 'GRPC'
              ? {
                  proto_path: protoPath,
                  service: grpcService,
                  method: grpcMethod,
                  metadata: [], // For future expansion
                }
              : null,
        },
        environment: activeEnv,
        globals: globals,
        sessionVars: sessionVariables,
      });
      const {
        response: res,
        environment: updatedEnv,
        globals: updatedGlobals,
        sessionVars: updatedSession,
      } = output;

      if (updatedEnv && activeEnvironmentId && updatedEnv.id === activeEnvironmentId) {
        updateEnvironment(updatedEnv);
      }

      if (updatedGlobals) {
        saveGlobals(updatedGlobals);
      }

      if (updatedSession) {
        // Update the store with session variables modified by scripts
        useWorkspaceStore.setState({ sessionVariables: updatedSession });
      }

      addToHistory({
        id: activeRequest?.id || `hist_${Date.now()}`,
        name: activeRequest?.name || url,
        description: activeRequest?.description || null,
        method,
        url,
        headers,
        body:
          backendBody ||
          (bodyMode === 'Raw'
            ? { Raw: [body, 'Json'] }
            : bodyMode === 'GraphQL'
              ? { GraphQL: { query: gqlQuery, variables: gqlVariables } }
              : { FormData: formData.filter((f) => f.key.trim() !== '') }),
        auth,
        variables: {},
        scripts: {
          preRequest: preRequestScript,
          tests: testScript,
        },
        grpc_config: null,
      });

      setResponse(res);
      toast.success('Requisição concluída!', {
        description: `${method} ${url} - ${res.status} OK`,
      });
    } catch (e) {
      toast.error('Falha ao enviar requisição');
      setResponse({
        status: 500,
        statusText: 'Internal Error',
        headers: [],
        body: typeof e === 'string' ? e : JSON.stringify(e, null, 2),
        timeMs: 0,
        sizeBytes: 0,
        testsResults: [],
        logs: [],
      });
    } finally {
      setLoading(false);
    }
  }

  const syntaxHighlight = (jsonStr: string) => {
    if (!jsonStr) return '';
    const str = jsonStr.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
    return str.replace(
      /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE]+-?\d+)?)/g,
      function (match) {
        let cls = 'json-number';
        if (/^"/.test(match)) {
          if (/:$/.test(match)) {
            cls = 'json-key';
          } else {
            cls = 'json-string';
          }
        } else if (/true|false/.test(match)) {
          cls = 'json-boolean';
        } else if (/null/.test(match)) {
          cls = 'json-null';
        }
        return '<span class="' + cls + '">' + match + '</span>';
      },
    );
  };

  const generateCurl = () => {
    let curl = `curl --location '${url}' \\\n--request ${method}`;

    headers.forEach((h) => {
      if (h.enabled && h.key) {
        curl += ` \\\n--header '${h.key}: ${h.value}'`;
      }
    });

    if (auth?.type === 'Bearer') {
      curl += ` \\\n--header 'Authorization: Bearer ${auth.data.token}'`;
    }

    if (['POST', 'PUT', 'PATCH'].includes(method as string) && body) {
      curl += ` \\\n--data '${body.replace(/'/g, "'\\''")}'`;
    }

    return curl;
  };

  if (!workspacePath) {
    return <WorkspaceSelector />;
  }

  return (
    <div className="app-layout">
      <Sidebar />
      <div className="main-content-area">
        <header className="header">
          <div className="header-left">
            <h1 className="logo">⚡ Tyny Pulse</h1>
            <span className="workspace-path-badge">{workspacePath}</span>
          </div>
          <div className="header-right">
            <div className="env-selector">
              <select
                value={activeEnvironmentId || ''}
                onChange={(e: React.ChangeEvent<HTMLSelectElement>) =>
                  setActiveEnvironment(e.target.value)
                }
                className="env-select"
              >
                {environments.map((e: Environment) => (
                  <option key={e.id} value={e.id}>
                    {e.name}
                  </option>
                ))}
              </select>
              <button
                className="env-settings-btn share-btn"
                onClick={() => setShowWorkspaceSettings(true)}
                title="Share Workspace"
              >
                <Users size={16} />
                <span>Share</span>
              </button>
              <button
                className="env-settings-btn"
                onClick={() => setShowGlobalsModal(true)}
                title="Manage Globals"
              >
                <Globe size={16} />
              </button>
              <button
                className="env-settings-btn"
                onClick={() => setShowEnvModal(true)}
                title="Manage Environments"
              >
                <Settings size={16} />
              </button>
              <button
                className="env-settings-btn"
                onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
                title="Toggle Theme"
              >
                {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
              </button>
            </div>
          </div>
        </header>

        <TabBar />

        {sidebarMode === 'Designs' ? (
          <DesignPanel />
        ) : (
          <AnimatePresence mode="wait">
            <motion.main
              key={activeTabId || 'empty'}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              transition={{ duration: 0.2 }}
              className="request-pane"
            >
              <Toaster
                theme={theme === 'dark' ? 'dark' : 'light'}
                position="bottom-right"
                richColors
              />
              <div className="request-bar">
                <select
                  value={method as string}
                  onChange={(e) => setMethod(e.target.value as HttpMethod)}
                  className="method-select"
                >
                  <option>GET</option>
                  <option>POST</option>
                  <option>PUT</option>
                  <option>DELETE</option>
                  <option>WS</option>
                  <option>GRPC</option>
                </select>
                <input
                  type="text"
                  value={url}
                  onChange={(e) => {
                    const newUrl = e.target.value;
                    setUrl(newUrl);
                    try {
                      const urlObj = new URL(newUrl);
                      const parsedParams: { key: string; value: string; enabled: boolean }[] = [];
                      urlObj.searchParams.forEach((val, key) => {
                        parsedParams.push({ key, value: val, enabled: true });
                      });
                      setParams(parsedParams);
                    } catch {
                      // Best effort: failures here must not break the surrounding flow.
                    }
                  }}
                  className="url-input"
                  placeholder="https://api.example.com/data"
                />
                <button
                  className="cookies-btn-top"
                  onClick={async () => {
                    const cookies = await invoke<string>('get_cookies', { url });
                    setActiveCookies(cookies);
                    setShowCookies(!showCookies);
                  }}
                  title="View Cookies"
                >
                  <div className="cookie-dot" />
                  Cookies
                </button>
                <button onClick={handleSend} disabled={loading} className="send-btn">
                  {loading ? '...' : 'Send'}
                </button>
                {activeRequest && (
                  <button
                    onClick={() =>
                      updateRequest({
                        ...activeRequest,
                        method,
                        url,
                        headers: headers.filter((h) => h.key.trim() !== ''),
                        body:
                          bodyMode === 'Raw'
                            ? body
                              ? { Raw: [body, 'Json'] }
                              : null
                            : bodyMode === 'GraphQL'
                              ? { GraphQL: { query: gqlQuery, variables: gqlVariables } }
                              : { FormData: formData.filter((f) => f.key.trim() !== '') },
                        auth,
                        scripts: {
                          preRequest: preRequestScript,
                          tests: testScript,
                        },
                        grpc_config:
                          method === 'GRPC'
                            ? {
                                proto_path: protoPath,
                                service: grpcService,
                                method: grpcMethod,
                                metadata: [],
                              }
                            : null,
                      })
                    }
                    className="save-btn"
                    title="Salvar alterações"
                  >
                    <Save size={16} />
                  </button>
                )}
                <button
                  onClick={() => setShowCurlModal(true)}
                  className="save-btn"
                  title="Gerar comando cURL"
                >
                  <Terminal size={16} />
                </button>
                <button
                  className="save-btn"
                  onClick={async () => {
                    const request = {
                      id: activeRequest?.id || 'temp',
                      method,
                      url,
                      headers: headers.filter((h) => h.key.trim() !== ''),
                      body: bodyMode === 'Raw' ? { Raw: [body, 'Json'] } : null,
                      auth,
                      variables: {},
                      scripts: { preRequest: preRequestScript, tests: testScript },
                    };
                    const code = await invoke<string>('generate_js_code', {
                      request,
                      target: codeTarget,
                    });
                    setGeneratedCode(code);
                    setShowCodeModal(true);
                  }}
                  title="Gerar código JavaScript"
                >
                  <Code2 size={16} />
                </button>
              </div>

              {method === 'GRPC' && (
                <div className="grpc-config-bar animate-fade-in">
                  <div className="grpc-input-group">
                    <label>Proto Path</label>
                    <input
                      type="text"
                      value={protoPath}
                      onChange={(e) => setProtoPath(e.target.value)}
                      placeholder="/path/to/service.proto"
                    />
                  </div>
                  <div className="grpc-input-group">
                    <label>Service</label>
                    <input
                      type="text"
                      value={grpcService}
                      onChange={(e) => setGrpcService(e.target.value)}
                      placeholder="UserService"
                    />
                  </div>
                  <div className="grpc-input-group">
                    <label>Method</label>
                    <input
                      type="text"
                      value={grpcMethod}
                      onChange={(e) => setGrpcMethod(e.target.value)}
                      placeholder="GetUser"
                    />
                  </div>
                </div>
              )}

              {showCookies && (
                <div className="cookies-overlay" onClick={() => setShowCookies(false)}>
                  <div className="cookies-modal" onClick={(e) => e.stopPropagation()}>
                    <div className="cookies-modal-header">
                      <h3>Cookies for this domain</h3>
                      <button className="close-cookies-btn" onClick={() => setShowCookies(false)}>
                        ×
                      </button>
                    </div>
                    <div className="cookies-modal-body">
                      {activeCookies ? (
                        <div className="cookie-list">
                          {activeCookies.split(';').map((c, i) => (
                            <div key={i} className="cookie-item">
                              <code className="cookie-code">{c.trim()}</code>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div className="empty-tab">No cookies found for this URL.</div>
                      )}
                    </div>
                  </div>
                </div>
              )}

              {showCodeModal && (
                <div className="cookies-overlay" onClick={() => setShowCodeModal(false)}>
                  <div
                    className="cookies-modal code-snippet-modal"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="cookies-modal-header">
                      <h3>Gerar Código JavaScript</h3>
                      <div className="code-targets">
                        <button
                          className={`target-btn ${codeTarget === 'fetch' ? 'active' : ''}`}
                          onClick={async () => {
                            const target = 'fetch';
                            setCodeTarget(target);
                            const code = await invoke<string>('generate_js_code', {
                              request: {
                                id: activeRequest?.id || 'temp',
                                name: activeRequest?.name || url,
                                description: activeRequest?.description || null,
                                method,
                                url,
                                headers: headers.filter((h) => h.key.trim() !== ''),
                                body: bodyMode === 'Raw' ? { Raw: [body, 'Json'] } : null,
                                auth,
                                variables: {},
                                scripts: { preRequest: preRequestScript, tests: testScript },
                                grpc_config: null,
                              },
                              target,
                            });
                            setGeneratedCode(code);
                          }}
                        >
                          Fetch
                        </button>
                        <button
                          className={`target-btn ${codeTarget === 'node' ? 'active' : ''}`}
                          onClick={async () => {
                            const target = 'node';
                            setCodeTarget(target);
                            const code = await invoke<string>('generate_js_code', {
                              request: {
                                id: activeRequest?.id || 'temp',
                                name: activeRequest?.name || url,
                                description: activeRequest?.description || null,
                                method,
                                url,
                                headers: headers.filter((h) => h.key.trim() !== ''),
                                body: bodyMode === 'Raw' ? { Raw: [body, 'Json'] } : null,
                                auth,
                                variables: {},
                                scripts: { preRequest: preRequestScript, tests: testScript },
                                grpc_config: null,
                              },
                              target,
                            });
                            setGeneratedCode(code);
                          }}
                        >
                          Node.js
                        </button>
                      </div>
                    </div>
                    <div className="cookies-modal-body">
                      <div className="curl-container">
                        <pre className="curl-code">{generatedCode}</pre>
                        <button
                          className="copy-curl-btn"
                          onClick={() => {
                            navigator.clipboard.writeText(generatedCode);
                            toast.success('Código copiado!');
                          }}
                        >
                          <Copy size={14} />
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Variable resolution hint bar */}
              {url.includes('{{') &&
                (() => {
                  const matches = url.match(/\{\{([^}]+)\}\}/g) || [];
                  const envVars = activeEnvironment?.variables || [];
                  const varMap: Record<string, string> = {};
                  envVars.forEach((v) => {
                    if (v.enabled) varMap[v.key as string] = v.current_value;
                  });

                  return (
                    <div className="var-hint-bar">
                      {matches.map((m, i) => {
                        const key = m.slice(2, -2);
                        const resolved = varMap[key];
                        return (
                          <span
                            key={i}
                            className={`var-token ${resolved ? 'resolved' : 'unresolved'}`}
                          >
                            {m} {resolved ? `→ ${resolved}` : '⚠ undefined'}
                          </span>
                        );
                      })}
                    </div>
                  );
                })()}

              <div className="request-config-panel">
                <div className="tabs">
                  <button
                    className={`tab-btn ${activeConfigTab === 'Params' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Params')}
                  >
                    Params ({params.length})
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'Auth' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Auth')}
                  >
                    Auth
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'Headers' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Headers')}
                  >
                    Headers ({headers.length})
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'Body' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Body')}
                  >
                    Body
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'Scripts' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Scripts')}
                  >
                    Scripts
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'Docs' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('Docs')}
                  >
                    Documentation
                  </button>
                  <button
                    className={`tab-btn ${activeConfigTab === 'LoadTest' ? 'active' : ''}`}
                    onClick={() => setActiveConfigTab('LoadTest')}
                  >
                    Load Test
                  </button>
                </div>

                <div className="tab-content">
                  {activeConfigTab === 'Params' && (
                    <div className="headers-pane">
                      {params.map((p, idx) => (
                        <div key={idx} className="header-row">
                          <input
                            type="checkbox"
                            checked={p.enabled}
                            onChange={(e) => {
                              const newParams = [...params];
                              newParams[idx].enabled = e.target.checked;
                              setParams(newParams);

                              try {
                                const urlObj = new URL(url);
                                urlObj.search = '';
                                newParams.forEach((np) => {
                                  if (np.enabled && np.key)
                                    urlObj.searchParams.append(np.key, np.value);
                                });
                                setUrl(urlObj.toString());
                              } catch {
                                // Best effort: failures here must not break the surrounding flow.
                              }
                            }}
                          />
                          <input
                            type="text"
                            placeholder="Key"
                            value={p.key}
                            onChange={(e) => {
                              const newParams = [...params];
                              newParams[idx].key = e.target.value;
                              setParams(newParams);

                              try {
                                const urlObj = new URL(url);
                                urlObj.search = '';
                                newParams.forEach((np) => {
                                  if (np.enabled && np.key)
                                    urlObj.searchParams.append(np.key, np.value);
                                });
                                setUrl(urlObj.toString());
                              } catch {
                                // Best effort: failures here must not break the surrounding flow.
                              }
                            }}
                          />
                          <input
                            type="text"
                            placeholder="Value"
                            value={p.value}
                            onChange={(e) => {
                              const newParams = [...params];
                              newParams[idx].value = e.target.value;
                              setParams(newParams);

                              try {
                                const urlObj = new URL(url);
                                urlObj.search = '';
                                newParams.forEach((np) => {
                                  if (np.enabled && np.key)
                                    urlObj.searchParams.append(np.key, np.value);
                                });
                                setUrl(urlObj.toString());
                              } catch {
                                // Best effort: failures here must not break the surrounding flow.
                              }
                            }}
                          />
                          <button
                            onClick={() => {
                              const newParams = params.filter((_, i) => i !== idx);
                              setParams(newParams);
                              try {
                                const urlObj = new URL(url);
                                urlObj.search = '';
                                newParams.forEach((np) => {
                                  if (np.enabled && np.key)
                                    urlObj.searchParams.append(np.key, np.value);
                                });
                                setUrl(urlObj.toString());
                              } catch {
                                // Best effort: failures here must not break the surrounding flow.
                              }
                            }}
                            className="remove-row-btn"
                          >
                            ×
                          </button>
                        </div>
                      ))}
                      <button
                        onClick={() =>
                          setParams([...params, { key: '', value: '', enabled: true }])
                        }
                        className="add-row-btn"
                      >
                        + Add Parameter
                      </button>
                    </div>
                  )}

                  {activeConfigTab === 'Auth' && (
                    <div className="auth-pane">
                      <select
                        value={auth?.type || 'NoAuth'}
                        onChange={(e) => {
                          const type = e.target.value;
                          if (type === 'NoAuth') setAuth({ type: 'NoAuth' });
                          else if (type === 'Bearer')
                            setAuth({ type: 'Bearer', data: { token: '' } });
                          else if (type === 'Basic')
                            setAuth({ type: 'Basic', data: { username: '', password: '' } });
                          else if (type === 'ApiKey')
                            setAuth({
                              type: 'ApiKey',
                              data: { key: 'x-api-key', value: '', in_header: true },
                            });
                          else if (type === 'OAuth2')
                            setAuth({
                              type: 'OAuth2',
                              data: { access_token: '', header_prefix: 'Bearer' },
                            });
                          else if (type === 'AWSSig4')
                            setAuth({
                              type: 'AWSSig4',
                              data: {
                                access_key: '',
                                secret_key: '',
                                region: 'us-east-1',
                                service: 'execute-api',
                                session_token: null,
                              },
                            });
                        }}
                        className="auth-select"
                      >
                        <option value="NoAuth">No Auth</option>
                        <option value="Bearer">Bearer Token</option>
                        <option value="Basic">Basic Auth</option>
                        <option value="ApiKey">API Key</option>
                        <option value="OAuth2">OAuth 2.0</option>
                        <option value="AWSSig4">AWS Signature v4</option>
                      </select>

                      <div className="auth-inputs">
                        {auth?.type === 'Bearer' && (
                          <input
                            type="text"
                            placeholder="Token"
                            value={auth.data?.token || ''}
                            onChange={(e) =>
                              setAuth({ ...auth, data: { ...auth.data, token: e.target.value } })
                            }
                          />
                        )}
                        {auth?.type === 'Basic' && (
                          <>
                            <input
                              type="text"
                              placeholder="Username"
                              value={auth.data?.username || ''}
                              onChange={(e) =>
                                setAuth({
                                  ...auth,
                                  data: { ...auth.data, username: e.target.value },
                                })
                              }
                            />
                            <input
                              type="password"
                              placeholder="Password"
                              value={auth.data?.password || ''}
                              onChange={(e) =>
                                setAuth({
                                  ...auth,
                                  data: { ...auth.data, password: e.target.value },
                                })
                              }
                            />
                          </>
                        )}
                        {auth?.type === 'ApiKey' && (
                          <>
                            <input
                              type="text"
                              placeholder="Key"
                              value={auth.data?.key || ''}
                              onChange={(e) =>
                                setAuth({ ...auth, data: { ...auth.data, key: e.target.value } })
                              }
                            />
                            <input
                              type="text"
                              placeholder="Value"
                              value={auth.data?.value || ''}
                              onChange={(e) =>
                                setAuth({ ...auth, data: { ...auth.data, value: e.target.value } })
                              }
                            />
                            <select
                              value={auth.data?.in_header ? 'header' : 'query'}
                              onChange={(e) =>
                                setAuth({
                                  ...auth,
                                  data: { ...auth.data, in_header: e.target.value === 'header' },
                                })
                              }
                              className="auth-sub-select"
                            >
                              <option value="header">In Header</option>
                              <option value="query">In Query Params</option>
                            </select>
                          </>
                        )}
                        {auth?.type === 'OAuth2' && (
                          <input
                            type="text"
                            placeholder="Access Token"
                            value={auth.data?.access_token || ''}
                            onChange={(e) =>
                              setAuth({
                                ...auth,
                                data: { ...auth.data, access_token: e.target.value },
                              })
                            }
                          />
                        )}
                        {auth?.type === 'AWSSig4' && (
                          <div className="auth-note">
                            AWS Signature v4 structure defined. Full signing logic pending.
                          </div>
                        )}
                      </div>
                    </div>
                  )}

                  {activeConfigTab === 'Headers' && (
                    <div className="headers-pane">
                      {headers.map((h, idx) => (
                        <div key={idx} className="header-row">
                          <input
                            type="checkbox"
                            checked={h.enabled}
                            onChange={(e) => {
                              const newHeaders = [...headers];
                              newHeaders[idx].enabled = e.target.checked;
                              setHeaders(newHeaders);
                            }}
                          />
                          <input
                            type="text"
                            placeholder="Key"
                            value={h.key}
                            onChange={(e) => {
                              const newHeaders = [...headers];
                              newHeaders[idx].key = e.target.value;
                              setHeaders(newHeaders);
                            }}
                          />
                          <input
                            type="text"
                            placeholder="Value"
                            value={h.value}
                            onChange={(e) => {
                              const newHeaders = [...headers];
                              newHeaders[idx].value = e.target.value;
                              setHeaders(newHeaders);
                            }}
                          />
                          <button
                            onClick={() => setHeaders(headers.filter((_, i) => i !== idx))}
                            className="remove-row-btn"
                          >
                            ×
                          </button>
                        </div>
                      ))}
                      <button
                        onClick={() =>
                          setHeaders([...headers, { key: '', value: '', enabled: true }])
                        }
                        className="add-row-btn"
                      >
                        + Add Header
                      </button>
                    </div>
                  )}

                  {activeConfigTab === 'Scripts' && (
                    <div className="scripts-pane scrollable">
                      <div className="script-section">
                        <label>Pre-request Script</label>
                        <textarea
                          value={preRequestScript}
                          onChange={(e) => setPreRequestScript(e.target.value)}
                          placeholder="// This script runs before the request is sent"
                          className="script-editor"
                        />
                      </div>
                      <div className="script-section">
                        <div className="script-header">
                          <label>Tests</label>
                          <button
                            className="ai-action-btn"
                            onClick={async () => {
                              setAiLoading(true);
                              try {
                                const tests = await invoke<string>('ai_generate_tests', {
                                  url,
                                  responseBody: response?.body || '{}',
                                });
                                setTestScript((prev) => (prev ? prev + '\n' : '') + tests);
                                toast.success('Testes gerados pela IA!');
                              } catch (e) {
                                toast.error('IA Falhou', { description: String(e) });
                              } finally {
                                setAiLoading(false);
                              }
                            }}
                            disabled={aiLoading}
                          >
                            {aiLoading ? (
                              'Gerando...'
                            ) : (
                              <>
                                <Sparkles size={14} /> Gerar Testes com IA
                              </>
                            )}
                          </button>
                        </div>
                        <textarea
                          value={testScript}
                          onChange={(e) => setTestScript(e.target.value)}
                          placeholder="// This script runs after the response is received"
                          className="script-editor"
                        />
                      </div>
                    </div>
                  )}

                  {activeConfigTab === 'Body' && (
                    <div className="body-pane">
                      {method === 'WS' ? (
                        <WebSocketDebugger id={activeRequest?.id || 'temp'} url={url} />
                      ) : (
                        <>
                          <div className="body-toolbar">
                            <div className="body-mode-selector">
                              <button
                                className={`mode-btn ${bodyMode === 'None' ? 'active' : ''}`}
                                onClick={() => setBodyMode('None')}
                              >
                                none
                              </button>
                              <button
                                className={`mode-btn ${bodyMode === 'Raw' ? 'active' : ''}`}
                                onClick={() => setBodyMode('Raw')}
                              >
                                raw
                              </button>
                              <button
                                className={`mode-btn ${bodyMode === 'FormData' ? 'active' : ''}`}
                                onClick={() => setBodyMode('FormData')}
                              >
                                form-data
                              </button>
                              <button
                                className={`mode-btn ${bodyMode === 'GraphQL' ? 'active' : ''}`}
                                onClick={() => setBodyMode('GraphQL')}
                              >
                                graphql
                              </button>
                            </div>
                            {bodyMode === 'Raw' && (
                              <button
                                onClick={() => {
                                  try {
                                    if (body) setBody(JSON.stringify(JSON.parse(body), null, 2));
                                  } catch {
                                    // Best effort: failures here must not break the surrounding flow.
                                  }
                                }}
                                className="beautify-btn"
                              >
                                Beautify
                              </button>
                            )}
                          </div>

                          {bodyMode === 'None' && (
                            <div className="empty-tab">This request does not have a body</div>
                          )}

                          {bodyMode === 'Raw' && (
                            <textarea
                              className="body-editor"
                              placeholder="Enter request body here..."
                              value={body}
                              onChange={(e) => setBody(e.target.value)}
                              spellCheck={false}
                            />
                          )}

                          {bodyMode === 'FormData' && (
                            <div className="headers-pane">
                              {formData.map((f, idx) => (
                                <div key={idx} className="header-row">
                                  <input
                                    type="checkbox"
                                    checked={f.enabled}
                                    onChange={(e) => {
                                      const newFd = [...formData];
                                      newFd[idx].enabled = e.target.checked;
                                      setFormData(newFd);
                                    }}
                                  />
                                  <input
                                    type="text"
                                    placeholder="Key"
                                    value={f.key}
                                    onChange={(e) => {
                                      const newFd = [...formData];
                                      newFd[idx].key = e.target.value;
                                      setFormData(newFd);
                                    }}
                                  />
                                  {f.file !== null ? (
                                    <div className="file-input-wrapper">
                                      <input
                                        type="text"
                                        readOnly
                                        value={f.file || 'No file selected'}
                                        className="file-path-input"
                                      />
                                      <button
                                        className="select-file-btn"
                                        onClick={async () => {
                                          const selected = await openFile({ multiple: false });
                                          if (selected && typeof selected === 'string') {
                                            const newFd = [...formData];
                                            newFd[idx].file = selected;
                                            setFormData(newFd);
                                          }
                                        }}
                                      >
                                        Select
                                      </button>
                                    </div>
                                  ) : (
                                    <input
                                      type="text"
                                      placeholder="Value"
                                      value={f.value}
                                      onChange={(e) => {
                                        const newFd = [...formData];
                                        newFd[idx].value = e.target.value;
                                        setFormData(newFd);
                                      }}
                                    />
                                  )}
                                  <select
                                    value={f.file === null ? 'text' : 'file'}
                                    onChange={(e) => {
                                      const newFd = [...formData];
                                      newFd[idx].file = e.target.value === 'file' ? '' : null;
                                      setFormData(newFd);
                                    }}
                                    className="auth-sub-select"
                                  >
                                    <option value="text">Text</option>
                                    <option value="file">File</option>
                                  </select>
                                  <button
                                    onClick={() =>
                                      setFormData(formData.filter((_, i) => i !== idx))
                                    }
                                    className="remove-row-btn"
                                  >
                                    ×
                                  </button>
                                </div>
                              ))}
                              <button
                                onClick={() =>
                                  setFormData([
                                    ...formData,
                                    { key: '', value: '', file: null, enabled: true },
                                  ])
                                }
                                className="add-row-btn"
                              >
                                + Add Form Field
                              </button>
                            </div>
                          )}

                          {bodyMode === 'GraphQL' && (
                            <div className="graphql-pane">
                              <div className="gql-section">
                                <label>Query</label>
                                <textarea
                                  className="gql-editor query"
                                  placeholder="query { ... }"
                                  value={gqlQuery}
                                  onChange={(e) => setGqlQuery(e.target.value)}
                                  spellCheck={false}
                                />
                              </div>
                              <div className="gql-section">
                                <label>Variables (JSON)</label>
                                <textarea
                                  className="gql-editor vars"
                                  placeholder="{}"
                                  value={gqlVariables}
                                  onChange={(e) => setGqlVariables(e.target.value)}
                                  spellCheck={false}
                                />
                              </div>
                            </div>
                          )}
                        </>
                      )}
                    </div>
                  )}

                  {activeConfigTab === 'Docs' && (
                    <div className="tab-content body-tab">
                      <div className="scripts-header">
                        <div className="scripts-title">Documentação (Markdown)</div>
                      </div>
                      <div className="docs-editor-container" style={{ padding: '16px' }}>
                        <textarea
                          className="body-editor docs-editor"
                          placeholder="Descreva esta requisição usando Markdown..."
                          value={activeRequest?.description || ''}
                          onChange={(e) => {
                            if (activeRequest) {
                              updateRequest({ ...activeRequest, description: e.target.value });
                            }
                          }}
                          style={{
                            height: '300px',
                            width: '100%',
                            resize: 'vertical',
                            fontSize: '14px',
                            lineHeight: '1.6',
                          }}
                        />
                      </div>
                    </div>
                  )}

                  {activeConfigTab === 'LoadTest' && <LoadTestingPanel />}
                </div>
              </div>

              <section className="response-section">
                <div className="response-header">
                  <div className="response-header-left">
                    <h3 className="response-title">Response</h3>
                    <div className="response-tabs">
                      <button
                        className={`resp-tab-btn ${activeResponseTab === 'Body' ? 'active' : ''}`}
                        onClick={() => setActiveResponseTab('Body')}
                      >
                        Body
                      </button>
                      <button
                        className={`resp-tab-btn ${activeResponseTab === 'Headers' ? 'active' : ''}`}
                        onClick={() => setActiveResponseTab('Headers')}
                      >
                        Headers{' '}
                        {response !== null &&
                          response.headers.length > 0 &&
                          `(${response.headers.length})`}
                      </button>
                      <button
                        className={`resp-tab-btn ${activeResponseTab === 'Preview' ? 'active' : ''}`}
                        onClick={() => setActiveResponseTab('Preview')}
                      >
                        Preview
                      </button>
                      <button
                        className={`resp-tab-btn ${activeResponseTab === 'Test Results' ? 'active' : ''}`}
                        onClick={() => setActiveResponseTab('Test Results')}
                      >
                        Test Results
                        {response !== null && response.testsResults.length > 0 && (
                          <span className="test-count">
                            ({response.testsResults.filter((r) => r.passed).length}/
                            {response.testsResults.length})
                          </span>
                        )}
                      </button>
                      <button
                        className={`resp-tab-btn ${activeResponseTab === 'Console' ? 'active' : ''}`}
                        onClick={() => setActiveResponseTab('Console')}
                      >
                        Console
                        {response !== null &&
                          response.logs.length > 0 &&
                          ` (${response.logs.length})`}
                      </button>
                    </div>
                  </div>
                  <div className="response-header-right">
                    {activeResponseTab === 'Body' && (
                      <div className="response-toggles">
                        <button
                          className={`resp-toggle-btn ${responseMode === 'Pretty' ? 'active' : ''}`}
                          onClick={() => setResponseMode('Pretty')}
                        >
                          Pretty
                        </button>
                        <button
                          className={`resp-toggle-btn ${responseMode === 'Raw' ? 'active' : ''}`}
                          onClick={() => setResponseMode('Raw')}
                        >
                          Raw
                        </button>
                      </div>
                    )}
                    {response && response.status && (
                      <div className="response-meta-new">
                        <span
                          className={`status-badge ${response.status >= 200 && response.status < 300 ? 'status-ok' : 'status-err'}`}
                        >
                          {response.status} {response.statusText}
                        </span>
                        <span className="meta-item">{response.timeMs} ms</span>
                        <span className="meta-item">
                          {(response.sizeBytes / 1024).toFixed(2)} KB
                        </span>
                      </div>
                    )}
                  </div>
                </div>
                <div className="response-body-wrapper">
                  {!response && !loading && (
                    <div className="empty-response-placeholder">
                      // Enter a URL above and click Send to see the result.
                    </div>
                  )}
                  {loading && <div className="loading-spinner">Processando requisição...</div>}

                  {response && (
                    <>
                      {activeResponseTab === 'Body' && (
                        <div className="response-body-container">
                          {response?.body && (
                            <div className="ai-explanation-bar">
                              <button
                                className="ai-explain-btn"
                                onClick={async () => {
                                  setAiLoading(true);
                                  try {
                                    const explanation = await invoke<string>(
                                      'ai_explain_response',
                                      { responseBody: response.body },
                                    );
                                    setAiExplanation(explanation);
                                  } catch {
                                    toast.error('IA Falhou');
                                  } finally {
                                    setAiLoading(false);
                                  }
                                }}
                              >
                                <BrainCircuit size={14} />{' '}
                                {aiLoading ? 'Analisando...' : 'Explicar com IA'}
                              </button>
                              {aiExplanation && (
                                <div className="ai-explanation-content">
                                  {aiExplanation}
                                  <button
                                    onClick={() => setAiExplanation('')}
                                    className="close-ai-btn"
                                  >
                                    ×
                                  </button>
                                </div>
                              )}
                            </div>
                          )}
                          <pre
                            className="response-body"
                            dangerouslySetInnerHTML={{
                              __html:
                                responseMode === 'Pretty'
                                  ? syntaxHighlight(response.body || '')
                                  : response.body || '',
                            }}
                          />
                        </div>
                      )}

                      {activeResponseTab === 'Headers' && (
                        <div className="response-headers-list">
                          <table className="headers-table">
                            <thead>
                              <tr>
                                <th>Header</th>
                                <th>Value</th>
                              </tr>
                            </thead>
                            <tbody>
                              {response.headers &&
                                response.headers.map((h, i) => (
                                  <tr key={i}>
                                    <td className="header-key">{h.key}</td>
                                    <td className="header-value">{h.value}</td>
                                  </tr>
                                ))}
                            </tbody>
                          </table>
                        </div>
                      )}

                      {activeResponseTab === 'Preview' && (
                        <div className="response-preview">
                          <iframe
                            title="Response Preview"
                            srcDoc={response.body || ''}
                            sandbox=""
                            className="preview-iframe"
                          />
                        </div>
                      )}

                      {activeResponseTab === 'Test Results' && (
                        <div className="test-results-view">
                          {!response.testsResults || response.testsResults.length === 0 ? (
                            <div className="empty-tests-msg">
                              Nenhum teste configurado ou executado para esta requisição.
                            </div>
                          ) : (
                            <div className="test-results-list">
                              {response.testsResults.map((r: TestResult, i: number) => (
                                <div
                                  key={i}
                                  className={`test-result-item ${r.passed ? 'passed' : 'failed'}`}
                                >
                                  <span className="test-status-badge">
                                    {r.passed ? 'PASS' : 'FAIL'}
                                  </span>
                                  <span className="test-name">{r.name}</span>
                                  {!r.passed && r.error && (
                                    <div className="test-error-detail">{r.error}</div>
                                  )}
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      )}

                      {activeResponseTab === 'Console' && (
                        <div className="test-results-view">
                          {!response.logs || response.logs.length === 0 ? (
                            <div className="empty-tests-msg">
                              Nenhum log gerado pelos scripts. Use console.log() para depurar.
                            </div>
                          ) : (
                            <div className="console-log-list">
                              {response.logs.map((log, i) => (
                                <div key={i} className={`console-log-item log_level_${log.level}`}>
                                  <span className="log-level-badge">{log.level.toUpperCase()}</span>
                                  <pre className="log-content">{log.content}</pre>
                                </div>
                              ))}
                            </div>
                          )}
                        </div>
                      )}
                    </>
                  )}
                </div>
              </section>
            </motion.main>
          </AnimatePresence>
        )}
      </div>

      {showEnvModal && activeEnvironment && (
        <div className="modal-overlay">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Environment: {activeEnvironment.name}</h2>
              <button onClick={() => setShowEnvModal(false)} className="close-modal-btn">
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="env-vars-list">
                {activeEnvironment.variables.map((variable, idx) => (
                  <div key={idx} className="header-row">
                    <input
                      type="text"
                      value={variable.key}
                      placeholder="Variable Name"
                      onChange={(e) => {
                        updateEnvironment({
                          ...activeEnvironment,
                          variables: activeEnvironment.variables.map((v, i) =>
                            i === idx ? { ...v, key: e.target.value } : v,
                          ),
                        });
                      }}
                    />
                    <input
                      type="text"
                      value={variable.current_value || variable.initial_value}
                      placeholder="Value"
                      onChange={(e) => {
                        updateEnvironment({
                          ...activeEnvironment,
                          variables: activeEnvironment.variables.map((v, i) =>
                            i === idx ? { ...v, current_value: e.target.value } : v,
                          ),
                        });
                      }}
                    />
                    <button
                      className="remove-row-btn"
                      onClick={() => {
                        updateEnvironment({
                          ...activeEnvironment,
                          variables: activeEnvironment.variables.filter((_, i) => i !== idx),
                        });
                      }}
                    >
                      ×
                    </button>
                  </div>
                ))}
                <button
                  className="add-row-btn"
                  onClick={() => {
                    const newVars = {
                      ...activeEnvironment.variables,
                      [`NewVar_${Date.now()}`]: '',
                    };
                    updateEnvironment({ ...activeEnvironment, variables: newVars });
                  }}
                >
                  + Add Variable
                </button>
              </div>
              <p className="hint-text">Use vars as {'{{VAR_NAME}}'} in URL, Headers and Body.</p>
            </div>
          </div>
        </div>
      )}

      {showCurlModal && (
        <div className="modal-overlay">
          <div className="modal-content curl-modal">
            <div className="modal-header">
              <h2>Generate Code (cURL)</h2>
              <button onClick={() => setShowCurlModal(false)} className="close-modal-btn">
                ×
              </button>
            </div>
            <div className="modal-body">
              <div className="curl-container">
                <pre className="curl-code">{generateCurl()}</pre>
                <button
                  className="copy-curl-btn"
                  onClick={() => {
                    navigator.clipboard.writeText(generateCurl());
                    setCopiedCurl(true);
                    setTimeout(() => setCopiedCurl(false), 2000);
                  }}
                >
                  {copiedCurl ? <Check size={16} /> : <Copy size={16} />}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {showGlobalsModal && <GlobalsModal onClose={() => setShowGlobalsModal(false)} />}

      {runnerItems && (
        <CollectionRunner
          items={runnerItems}
          environment={activeEnvironment}
          onClose={() => setRunnerItems(null)}
        />
      )}

      {showWorkspaceSettings && (
        <WorkspaceSettings onClose={() => setShowWorkspaceSettings(false)} />
      )}

      {showCommandPalette && <CommandPalette onClose={() => setShowCommandPalette(false)} />}
    </div>
  );
}

export default App;
