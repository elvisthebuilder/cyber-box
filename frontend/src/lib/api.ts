import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Tab {
  id: string;
  title: string;
  kind: "terminal" | "editor";
  path?: string;
}

export interface ToolInfo {
  name: string;
  category: string;
  description: string;
  binary: string;
  invocation: string;
  default_flags: string;
  output: string;
  tor_wrappable: boolean;
  image: string | null;
  install_cmd: string | null;
  install_status: {
    kind: "Unknown" | "Installed" | "NotInstalled" | "Installing" | "Failed";
    detail?: string;
  };
}

export const api = {
  listTools: () => invoke<ToolInfo[]>("list_tools"),
  containerStatus: () => invoke<boolean>("container_status"),
  torStatus: () => invoke<boolean>("tor_status"),
  toggleTor: (enabled: boolean) => invoke<boolean>("toggle_tor", { enabled }),
  refreshInstallStatus: () => invoke<void>("refresh_install_status"),
  installTool: (name: string) => invoke<void>("install_tool", { name }),
  listDir: (path: string) => invoke<string[]>("list_dir", { path }),
  readFile: (path: string) => invoke<string>("read_file", { path }),
  writeFile: (path: string, content: string) => invoke<void>("write_file", { path, content }),
  ptyOpen: (id: string) => invoke<void>("pty_open", { id }),
  ptyWrite: (id: string, data: string) => invoke<void>("pty_write", { id, data }),
  ptyResize: (id: string, cols: number, rows: number) => invoke<void>("pty_resize", { id, cols, rows }),
  ptyClose: (id: string) => invoke<void>("pty_close", { id }),
  ptyCwd: (id: string) => invoke<string>("pty_cwd", { id }),
  askAi: (requestId: string, question: string, context: string) =>
    invoke<void>("ask_ai", { requestId, question, context }),
};

export function onEvent<T>(name: string, handler: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(name, (e) => handler(e.payload));
}

export function b64encode(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export function b64decode(s: string): Uint8Array {
  const binary = atob(s);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
