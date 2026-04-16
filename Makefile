.PHONY: help dev start portless build check fmt clean

# Override via `make portless PORTLESS_NAME=admin.dbnyan`
PORTLESS_NAME ?= dbnyan

help:
	@echo "dbnyan tasks:"
	@echo "  make dev       Run development mode (Rust :3939 + SvelteKit HMR :5173)"
	@echo "  make start     Production-style: web build + Rust release on a single port"
	@echo "  make portless  Same as start, but spawned under portless (https://$(PORTLESS_NAME).localhost)"
	@echo "  make build     Build web bundle + Rust release binary"
	@echo "  make check     cargo check + svelte-check"
	@echo "  make fmt       cargo fmt + prettier (web)"
	@echo "  make clean     Remove build artifacts (cargo target, web build)"

dev:
	./bin/dev

start:
	./bin/start

portless:
	portless $(PORTLESS_NAME) ./bin/start

build:
	cd web && bun run build
	cargo build -p dbnyan-server --release

check:
	cargo check --workspace
	cd web && bun run check

fmt:
	cargo fmt --all
	cd web && bun run format

clean:
	cargo clean
	rm -rf web/build web/.svelte-kit
