# Contributing to cyber-box

Thanks for taking a look. This project is young and there's plenty to do —
bug fixes, new tools in the registry, UI polish, and platform support are
all welcome.

## Project layout

```
crates/cyberbox-core/   shared Rust library: Docker client, tool registry,
                         Tor control, Ollama client, session handling
src-tauri/               Tauri desktop app backend (Rust), built on cyberbox-core
frontend/                Svelte 5 + xterm.js + CodeMirror 6 desktop UI
docker/                  the "toolbox" container image (Kali + tools + Tor)
registry/tools.toml      the tool manifest — see "Adding a tool" below
```

See the [Architecture](README.md#architecture) section of the README for how
these pieces talk to each other.

## Getting set up

You'll need:

- Docker (running daemon)
- Rust/Cargo (stable) with `rustfmt` and `clippy` components
- Node.js + npm
- [Ollama](https://ollama.com), for the AI panel

```sh
git clone https://github.com/elvisthebuilder/cyber-box.git
cd cyber-box
./scripts/run.sh
```

For frontend-only iteration (hot reload against a running toolbox
container), you can also use `cargo tauri dev` if you have the
[Tauri CLI](https://tauri.app/reference/cli/) installed, or just re-run
`./scripts/run.sh` after changes — it rebuilds the frontend and the app.

## Before opening a PR

Run the same checks CI runs:

```sh
# Rust
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace --release

# Frontend
cd frontend
npm run format:check   # npm run format to auto-fix
npm run lint
npm run check           # svelte-check (types)
npm run build
```

A PR with a red CI check will get sent back before it gets a real review, so
it's worth running these locally first.

## Adding a tool

This is the easiest and most valuable kind of contribution. Every tool is a
`[[tool]]` table in `registry/tools.toml` — no Rust or frontend code needed.

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

Two ways to ship it:

1. **Pre-baked** — install the binary in `docker/toolbox.Dockerfile` and it's
   available immediately.
2. **Install-on-demand** — add an `install_cmd` (a shell command run inside
   the toolbox container) instead of touching the Dockerfile. See
   `subfinder` in `registry/tools.toml` for an example that bootstraps a Go
   toolchain and builds from source. No image rebuild needed.

Full field reference is in the README's
[Extending the registry](README.md#extending-the-registry) section.

## Code style

- Rust: default `rustfmt` formatting, `clippy`-clean. Prefer `anyhow::Result`
  at the edges and keep `cyberbox-core` free of Tauri/UI-specific types —
  it's meant to be usable by any front-end.
- Frontend: Prettier-formatted, ESLint-clean, Svelte 5 runes (`$state`,
  `$derived`, `$props`) rather than the old stores-based API.
- Comments should explain *why*, not *what* — see existing files (e.g. the
  `pty_open` cwd-tracking comment in `src-tauri/src/commands.rs`) for the
  bar.

## Reporting security issues

cyber-box bundles real offensive-security tooling and is meant to be run
against systems you're authorized to test. If you find a vulnerability in
cyber-box itself (e.g. something that breaks the container sandbox, leaks
credentials, or lets injected input escape a shell command), please open a
private report rather than a public issue — use GitHub's "Report a
vulnerability" flow under the Security tab.

## Commit / PR conventions

- Keep commits focused; a good commit message explains *why*, not just what
  changed.
- Rebase on `main` rather than merging it in, where practical.
- Squash fixup commits before requesting review.
