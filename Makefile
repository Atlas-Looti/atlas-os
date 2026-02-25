.PHONY: all build check test ci install fmt clippy clean run

# Default target
all: help

help:
	@echo "Atlas OS"
	@echo ""
	@echo "  make build      Build workspace (debug)"
	@echo "  make release    Build workspace (release)"
	@echo "  make check      Type-check without building"
	@echo "  make test       Run all tests"
	@echo "  make fmt        Format code"
	@echo "  make clippy     Lint with clippy"
	@echo "  make ci         Full CI pipeline (fmt check + clippy + test)"
	@echo "  make install    Install atlas binary to ~/.cargo/bin"
	@echo "  make uninstall  Remove atlas binary"
	@echo "  make clean      Clean build artifacts"
	@echo ""

# ── Build ──────────────────────────────────────────────────────────

build:
	cargo build --workspace

release:
	cargo build --workspace --release

check:
	cargo check --workspace

# ── Quality ────────────────────────────────────────────────────────

test:
	cargo test --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# ── CI (matches GitHub Actions) ────────────────────────────────────

ci: fmt-check clippy test

# ── Install / Run ──────────────────────────────────────────────────

install:
	cargo install --path crates/cli --locked --force

uninstall:
	-cargo uninstall atlas-cli

run:
	cargo run -p atlas-cli --

# ── Cleanup ────────────────────────────────────────────────────────

clean:
	cargo clean
