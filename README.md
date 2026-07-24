# cyber-box

[![CI](https://github.com/elvisthebuilder/cyber-box/actions/workflows/ci.yml/badge.svg)](https://github.com/elvisthebuilder/cyber-box/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A desktop pentest workstation. The app itself is a normal Tauri + Svelte
desktop app, but every actual tool — nmap, sqlmap, Metasploit, hydra, tshark,
Tor, everything — executes inside a single sandboxed Docker container (the
"toolbox"), driven live via `docker exec`. Real output, real targets you're
authorized to test, streamed back into an embedded terminal as it happens.

> [!WARNING]
> Only run tools from this project against systems you own or have explicit
> written authorization to test. You are responsible for how you use it.

## Contents

- [Features](#features)
- [Requirements](#requirements)
- [Quick start](#quick-start)
- [Architecture](#architecture)
- [Extending the registry](#extending-the-registry)
- [Known limitations](#known-limitations-v1)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Embedded terminal(s)** — multiple real shell tabs backed by an
  interactive `docker exec` PTY session into the toolbox container, using
  xterm.js on the frontend.
- **Built-in code editor** — browse the container's filesystem in the Files
  sidebar and open any file in a syntax-highlighted, multi-tab CodeMirror 6
  editor (language auto-detected from the file extension). `Ctrl+S` / `Cmd+S`
  saves back into the container; unsaved tabs show a `●` marker. The file
  tree polls the container and picks up files/folders created from the
  terminal (or by other tools) automatically.
- **Tool sidebar** — categorized registry of pentest tools (recon, web,
  exploitation, credentials, traffic); clicking one inserts its invocation
  into the active terminal tab.
- **Install on demand** — tools that aren't baked into the toolbox image yet
  (e.g. `subfinder`, `amass`, `wpscan` in v1) show up with an "install"
  badge. Clicking installs in the background inside the container, streamed
  live into the sidebar — the UI stays responsive the whole time.
- **AI assistant panel** — ask questions about the active terminal's recent
  output; runs entirely on a local Ollama model, toggle with `Ctrl+I`. When
  off, no network call is made at all.
- **Tor toggle** — pre-installed and pre-configured inside the container, off
  by default. Toggle from the status bar; tools marked `tor_wrappable` in the
  registry get transparently wrapped with `torsocks` while Tor is on.
- **Registry-driven extensibility** — tools are defined in
  `registry/tools.toml`, not hardcoded. Add a tool by appending a table entry;
  no Rust changes required. See [Extending the registry](#extending-the-registry).

## Requirements

- Docker (running daemon)
- Rust/Cargo (stable)
- Node.js + npm (to build the frontend)
- [Ollama](https://ollama.com) running locally, with a model pulled

## Quick start

```sh
git clone https://github.com/elvisthebuilder/cyber-box.git
cd cyber-box
./scripts/run.sh
```

This builds the toolbox Docker image if it doesn't exist yet, warns if Ollama
isn't reachable or the default model isn't pulled, builds the frontend, then
launches the desktop app.

Or step by step:

```sh
make image         # build the toolbox Docker image
make ollama-pull    # pull the default AI model
make run             # build image + frontend + app, then launch it
```

## Architecture

```
Host                                        Docker container "cyberbox-toolbox"
 cyber-box (Tauri + Svelte desktop app)      supervisord (pid 1)
  ├─ cyberbox-core (bollard) ── docker exec ──►  ├─ tor (off by default)
  │    │                                          ├─ keepalive
  │    └─ interactive PTY exec ─► xterm.js         └─ nmap, sqlmap, msf, hydra, ...
  └─ reqwest ─────► Ollama (host, :11434)
```

```
crates/cyberbox-core/   shared Rust library: Docker client, tool registry,
                         Tor control, Ollama client, session handling
src-tauri/               Tauri desktop app backend (Rust), built on cyberbox-core
frontend/                Svelte 5 + xterm.js + CodeMirror 6 desktop UI
docker/                  the "toolbox" container image (Kali + tools + Tor)
registry/tools.toml      the tool manifest — see "Extending the registry" below
```

`src-tauri` exposes Tauri commands (`pty_open`, `read_file`/`write_file`,
`install_tool`, `toggle_tor`, `ask_ai`, ...) that the Svelte frontend calls
via `@tauri-apps/api`; each one is backed by `cyberbox-core`, which owns the
actual Docker/Tor/Ollama logic so it isn't tied to any particular UI.

Ollama runs on the host (not in the container) — it's already installed
there, keeps the toolbox image focused on security tools, and avoids GPU
passthrough complexity. This is the one deliberate exception to "everything
runs in the container."

## Extending the registry

Every tool is a `[[tool]]` table in `registry/tools.toml`:

```toml
[[tool]]
name = "sqlmap"
category = "web"
description = "Automated SQL injection and database takeover tool"
binary = "sqlmap"
invocation = "sqlmap -u {target} {flags} --batch"
default_flags = "--level=1 --risk=1"
output = "stream"
tor_wrappable = true
```

- `invocation` — the shell command template; `{target}` and `{flags}` are
  substituted from the launch prompt.
- `tor_wrappable` — whether it's safe to prefix the command with `torsocks`
  when Tor is enabled. Tools needing raw sockets (nmap SYN scans, masscan,
  hydra, msf) should stay `false`.
- `image` — reserved, currently unused. In v1 all tools run as binaries
  already installed in the shared toolbox container. A future tool catalog
  repo can set this field to mean "run this tool in its own sidecar
  container" instead, without changing the schema.
- `install_cmd` — optional. If present, the tool doesn't need to be baked
  into `docker/toolbox.Dockerfile` at all: it shows up as "not installed" in
  the sidebar, and clicking it runs this shell command inside the toolbox
  container (in the background) to install it — anything from a plain
  `apt-get install -y ...` to bootstrapping a Go toolchain and building from
  source (see `subfinder` in `registry/tools.toml` for an example). Omit it
  for tools that are already part of the base image.

To add a pre-baked tool: install its binary into `docker/toolbox.Dockerfile`,
then append a `[[tool]]` entry here. To add an on-demand tool: just append a
`[[tool]]` entry with an `install_cmd` — no image rebuild needed. Either way,
no Rust code changes required.

## Known limitations (v1)

- **Wireshark GUI is not included** — the container is headless (no X11), so
  only `tshark` (the CLI packet analyzer) is available. Capture with tshark
  inside the box, export the `.pcap`, and open it in a host-side Wireshark
  if you want the GUI.
- The AI panel calls a **host-local** Ollama instance, not the toolbox
  container — see [Architecture](#architecture).
- The editor and file tree operate on the toolbox container's filesystem,
  not the host's.

## Contributing

Bug fixes, new tools in the registry, UI polish, and platform support are
all welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for how the project is
laid out, how to run the same checks CI does, and how to add a tool to the
registry (usually just a TOML entry, no code).

## License

[MIT](LICENSE)
