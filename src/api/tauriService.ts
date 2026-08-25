import { invoke } from "@tauri-apps/api/core";

export const TauriService = {
  sendRequest: async (payload: any) => {
    return invoke("send_request", payload);
  },
  
  getCookies: async (url: string) => {
    return invoke<string>("get_cookies", { url });
  },

  generateJsCode: async (request: any, target: 'fetch' | 'node') => {
    return invoke<string>("generate_js_code", { request, target });
  }
};
