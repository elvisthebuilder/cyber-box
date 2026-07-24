IMAGE := cyberbox-toolbox:latest
MODEL := llama3.1:8b-instruct-q4_K_M

.PHONY: image build run ollama-pull clean

image:
	docker build -t $(IMAGE) -f docker/toolbox.Dockerfile docker/

build:
	cargo build --release

run: image
	cargo run --release --bin cyberbox

ollama-pull:
	ollama pull $(MODEL)

clean:
	docker rm -f cyberbox-toolbox 2>/dev/null || true
	docker rmi $(IMAGE) 2>/dev/null || true
	cargo clean
