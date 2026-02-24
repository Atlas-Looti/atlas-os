.PHONY: all build check test ci install fmt clippy clean run

# Default target shows help
all: help

# Show available commands
help:
	@echo "Atlas OS Makefile"
	@echo ""
	@echo "Available commands:"
	@echo "  make help       - Show this help message"
	@echo "  make build      - Build the entire workspace"
	@echo "  make check      - Check compilation without producing binaries"
	@echo "  make test       - Run all tests in the workspace"
	@echo "  make fmt        - Run code formatter"
	@echo "  make clippy     - Run linter on all targets and features"
	@echo "  make ci         - Run CI tasks (fmt, clippy, test)"
	@echo "  make install    - Install the 'atlas' CLI binary to your system (~/.cargo/bin)"
	@echo "  make run        - Run the CLI locally for development"
	@echo "  make clean      - Clean build artifacts"
	@echo ""

# Build the entire workspace
build:
	cargo build --workspace

# Check compilation without producing binaries (faster than build)
check:
	cargo check --workspace

# Run all tests in the workspace
test:
	cargo test --workspace

# Run code formatter
fmt:
	cargo fmt --all

# Run linter (Clippy) on all targets and features
clippy:
	cargo clippy --workspace --all-targets --all-features

# Run continuous integration tasks (formatting, linting, testing)
ci: fmt clippy test

# Install the `atlas` CLI binary to your system (~/.cargo/bin)
install:
	cargo install --path crates/cli --locked --force

# Convenient command to run the CLI locally for development
run:
	cargo run -p atlas-cli --

# Clean build artifacts
clean:
	cargo clean
