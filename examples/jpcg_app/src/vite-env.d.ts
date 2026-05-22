/// <reference types="vite/client" />

declare module "*.module.css" {
  const classes: { readonly [key: string]: string };
  export default classes;
}

interface Window {
  __TAURI__?: {
    core?: {
      invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    };
    event?: {
      listen: (event: string, cb: (e: { payload: unknown }) => void) => Promise<() => void>;
    };
  };
}
