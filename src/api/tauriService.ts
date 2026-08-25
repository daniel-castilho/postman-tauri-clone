import { invoke } from '@tauri-apps/api/core';
import type { Environment, GlobalVariables, HttpRequest } from '../types/ipc';

export const TauriService = {
  sendRequest: async (payload: {
    request: HttpRequest;
    environment: Environment;
    globals: GlobalVariables;
    sessionVars: Record<string, string>;
  }) => {
    return invoke('send_request', payload);
  },

  getCookies: async (url: string) => {
    return invoke<string>('get_cookies', { url });
  },

  generateJsCode: async (request: HttpRequest, target: 'fetch' | 'node') => {
    return invoke<string>('generate_js_code', { request, target });
  },
};
