#!/usr/bin/env bash
# One-command bootstrap: build the toolbox image if missing, check Ollama is
# reachable, then launch the desktop app.
set -euo pipefail
cd "$(dirname "$0")/.."

IMAGE="cyberbox-toolbox:latest"

if ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
    echo "==> Toolbox image not found, building it (this can take a while)..."
    docker build -t "$IMAGE" -f docker/toolbox.Dockerfile docker/
fi

if command -v ollama >/dev/null 2>&1; then
    if ! curl -sf http://127.0.0.1:11434/api/tags >/dev/null 2>&1; then
        echo "==> Warning: Ollama doesn't seem to be running (http://127.0.0.1:11434 unreachable)."
        echo "    Start it with 'ollama serve', or the AI panel will show errors when used."
    elif [ -z "$(ollama list 2>/dev/null | tail -n +2)" ]; then
        echo "==> Warning: no Ollama models pulled yet. Pull one, e.g.:"
        echo "    ollama pull llama3.1:8b-instruct-q4_K_M"
        echo "    The AI panel lets you pick from whatever models you have pulled."
    fi
else
    echo "==> Warning: 'ollama' not found on PATH. Install it for the AI panel to work."
fi

echo "==> Building frontend..."
npm --prefix frontend install --no-fund --no-audit >/dev/null
npm --prefix frontend run build >/dev/null

echo "==> Launching cyber-box..."
cargo build --release -p cyberbox-desktop
exec ./target/release/cyberbox-desktop
