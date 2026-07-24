# cyber-box

A terminal UI pentest workstation. The TUI runs on your host terminal but
every actual tool — nmap, sqlmap, Metasploit, hydra, tshark, Tor, everything
— executes inside a single sandboxed Docker container ("toolbox"), driven
live via `docker exec`. Real output, real targets you're authorized to test,
streamed back into the TUI as it happens.

Only ever run tools here against systems you own or have explicit written
authorization to test.

## Features

- **Tool browser** — categorized registry of pentest tools (recon, web,
  exploitation, credentials, traffic), launch with a target/flags prompt.
- **Install on demand** — tools that aren't baked into the toolbox image yet
  (e.g. `subfinder`, `amass`, `wpscan` in v1) show up greyed out as
  "not installed". Press `i` to install one; it installs in the background
  inside the container (streamed into the Output Pane like a tool run) —
  the TUI stays responsive the whole time.
- **Live output pane** — streaming stdout/stderr from the running tool.
- **AI assistant panel** — ask questions about the last tool's output; runs
  entirely on a local Ollama model, toggle on/off with `Ctrl+A`. When off, no
  network call is made at all.
- **Tor toggle** — pre-installed and pre-configured inside the container, off
  by default. `Ctrl+T` starts/stops it; tools marked `tor_wrappable` in the
  registry get transparently wrapped with `torsocks` while Tor is on.
- **Registry-driven extensibility** — tools are defined in
  `registry/tools.toml`, not hardcoded. Add a tool by appending a table entry;
  no Rust changes required. See [Extending the registry](#extending-the-registry).

## Requirements

- Docker (running daemon)
- Rust/Cargo (stable)
- [Ollama](https://ollama.com) running locally, with a model pulled

## Quick start

```sh
./scripts/run.sh
```

This builds the toolbox Docker image if it doesn't exist yet, warns if Ollama
isn't reachable or the default model isn't pulled, then launches the TUI.

Or step by step:

```sh
make image         # build the toolbox Docker image
make ollama-pull    # pull the default AI model
make run             # build image if needed + cargo run --release
```

## Keybindings

| Key       | Action                              |
|-----------|--------------------------------------|
| `Tab`     | cycle focus: Browser → Output → AI  |
| `j` / `k` | move selection / scroll             |
| `Enter`   | launch selected tool / submit input |
| `i`       | install the selected tool (if not already installed) |
| `Esc`     | return focus to Browser             |
| `Ctrl+T`  | toggle Tor on/off                   |
| `Ctrl+A`  | toggle AI assistant on/off          |
| `Ctrl+C` / `q` | quit                            |

## Architecture

```
Host                                    Docker container "cyberbox-toolbox"
 cyber-box (Rust TUI)                    supervisord (pid 1)
  ├─ bollard  ──── docker exec ───────►    ├─ tor (off by default)
  │                                        ├─ keepalive
  └─ reqwest ─────► Ollama (host, :11434)  └─ nmap, sqlmap, msf, hydra, ...
```

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
  the browser, and pressing `i` runs this shell command inside the toolbox
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
