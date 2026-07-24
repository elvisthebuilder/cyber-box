import { api } from "./api";

export interface FileNode {
  path: string;
  name: string;
  isDir: boolean;
  expanded: boolean;
  children?: FileNode[];
}

/** Lists a directory's entries and turns them into child FileNodes. */
export async function listChildren(node: FileNode): Promise<FileNode[]> {
  const entries = await api.listDir(node.path).catch(() => []);
  return entries
    .filter((e) => e.length > 0)
    .map((e) => {
      const isDir = e.endsWith("/");
      const name = isDir ? e.slice(0, -1) : e;
      return {
        path: node.path === "/" ? `/${name}` : `${node.path}/${name}`,
        name,
        isDir,
        expanded: false,
      };
    });
}
