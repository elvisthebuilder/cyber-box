// Lightweight file/folder icon set, similar in spirit to Terax's per-extension
// icon resolver (iconify/catppuccin there) but hand-rolled to avoid pulling in
// a large icon package for a handful of glyphs.

const ICONS = {
  folder: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M1.5 3.5A1 1 0 0 1 2.5 2.5h3.379a1 1 0 0 1 .707.293l1.121 1.121a1 1 0 0 0 .707.293H13.5a1 1 0 0 1 1 1V12.5a1 1 0 0 1-1 1h-11a1 1 0 0 1-1-1v-9Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>`,
  folderOpen: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M1.5 5.3V3.5a1 1 0 0 1 1-1h3.379a1 1 0 0 1 .707.293l1.121 1.121a1 1 0 0 0 .707.293H13.5a1 1 0 0 1 1 1v.09" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/><path d="M1.86 6.4a1 1 0 0 1 .98-.8h10.32a1 1 0 0 1 .98 1.2l-.72 4.6a1 1 0 0 1-.98.8H3.56a1 1 0 0 1-.98-.8l-.72-4.6Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>`,
  file: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M4 1.5h5.379a1 1 0 0 1 .707.293l2.621 2.621a1 1 0 0 1 .293.707V13.5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/><path d="M9.5 1.5V4a1 1 0 0 0 1 1h2.5" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>`,
  code: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M5.5 5 2.5 8l3 3M10.5 5l3 3-3 3" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  braces: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M5.5 2c-1.2 0-1.8.6-1.8 1.8V6c0 .8-.4 1.2-1.2 1.2v1.6c.8 0 1.2.4 1.2 1.2v2.2c0 1.2.6 1.8 1.8 1.8M10.5 2c1.2 0 1.8.6 1.8 1.8V6c0 .8.4 1.2 1.2 1.2v1.6c-.8 0-1.2.4-1.2 1.2v2.2c0 1.2-.6 1.8-1.8 1.8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  image: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="1.5" y="2.5" width="13" height="11" rx="1.2" stroke="currentColor" stroke-width="1.2"/><circle cx="5.2" cy="6" r="1" fill="currentColor"/><path d="M2 12l3.8-4 2.5 2.7L11 8l3 4" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/></svg>`,
  text: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M4 1.5h5.379a1 1 0 0 1 .707.293l2.621 2.621a1 1 0 0 1 .293.707V13.5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-11a1 1 0 0 1 1-1Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round"/><path d="M5 8h6M5 10.3h6M5 5.7h2.5" stroke="currentColor" stroke-width="1.1" stroke-linecap="round"/></svg>`,
  archive: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="2" width="12" height="12" rx="1.2" stroke="currentColor" stroke-width="1.2"/><path d="M6.3 2v2.2M9.7 2v2.2M6.3 6.6h3.4v2.3H6.3z" stroke="currentColor" stroke-width="1.1" stroke-linejoin="round"/></svg>`,
  terminal: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><rect x="1.5" y="2.5" width="13" height="11" rx="1.2" stroke="currentColor" stroke-width="1.2"/><path d="M4 6.2 6.4 8 4 9.8M8 9.8h3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round"/></svg>`,
  key: `<svg viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg"><circle cx="5" cy="8" r="2.7" stroke="currentColor" stroke-width="1.2"/><path d="M7.4 8h6.1M11.5 8v2M13.3 8v1.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/></svg>`,
} as const;

export interface FileIconInfo {
  svg: string;
  color: string;
}

interface Rule {
  exts: string[];
  icon: keyof typeof ICONS;
  color: string;
}

const RULES: Rule[] = [
  {
    exts: [
      "ts",
      "tsx",
      "js",
      "jsx",
      "mjs",
      "cjs",
      "py",
      "rs",
      "go",
      "c",
      "cpp",
      "h",
      "java",
      "rb",
      "php",
      "lua",
    ],
    icon: "code",
    color: "#7aa2f7",
  },
  { exts: ["json", "jsonc", "toml", "yaml", "yml", "ini", "cfg", "conf"], icon: "braces", color: "#e0af68" },
  { exts: ["png", "jpg", "jpeg", "gif", "svg", "ico", "webp", "bmp"], icon: "image", color: "#bb9af7" },
  { exts: ["md", "markdown", "txt", "log", "rst"], icon: "text", color: "#9aa5ce" },
  { exts: ["zip", "tar", "gz", "tgz", "bz2", "xz", "7z", "rar"], icon: "archive", color: "#e0af68" },
  { exts: ["sh", "bash", "zsh", "fish"], icon: "terminal", color: "#9ece6a" },
  { exts: ["pem", "key", "crt", "pub", "asc"], icon: "key", color: "#f7768e" },
];

export function fileIcon(name: string): FileIconInfo {
  const dot = name.lastIndexOf(".");
  const ext = dot > 0 ? name.slice(dot + 1).toLowerCase() : "";
  for (const rule of RULES) {
    if (rule.exts.includes(ext)) return { svg: ICONS[rule.icon], color: rule.color };
  }
  return { svg: ICONS.file, color: "var(--text-faint)" };
}

export function folderIcon(expanded: boolean): FileIconInfo {
  return { svg: expanded ? ICONS.folderOpen : ICONS.folder, color: "#7aa2f7" };
}
