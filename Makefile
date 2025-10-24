# Makefile for Yew WebAssembly Application

# Default target
.PHONY: all
all: serve

# Install dependencies
.PHONY: install
install:
	@echo "Installing Rust dependencies..."
	cargo check

# Build the project
.PHONY: build
build:
	@echo "Building WebAssembly application..."
	cargo build --target wasm32-unknown-unknown

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf dist/

# Serve the application with Trunk
.PHONY: serve
serve:
	@echo "Starting Trunk development server..."
	@echo "Server will be available at: http://127.0.0.1:8080"
	@echo "Press Ctrl+C to stop the server"
	unset NO_COLOR && trunk serve --port 8080 --address 127.0.0.1 --disable-address-lookup

# Serve on a different port
.PHONY: serve-dev
serve-dev:
	@echo "Starting Trunk development server on port 8081..."
	@echo "Server will be available at: http://127.0.0.1:8081"
	@echo "Press Ctrl+C to stop the server"
	unset NO_COLOR && trunk serve --port 8081 --address 127.0.0.1 --disable-address-lookup

# Build for production
.PHONY: build-prod
build-prod:
	@echo "Building for production..."
	unset NO_COLOR && trunk build --release

# Watch for changes and rebuild
.PHONY: watch
watch:
	@echo "Watching for changes..."
	unset NO_COLOR && trunk watch

# Check code without building
.PHONY: check
check:
	@echo "Checking code..."
	cargo check

# Format code
.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt

# Run clippy linter
.PHONY: clippy
clippy:
	@echo "Running clippy..."
	cargo clippy --target wasm32-unknown-unknown

# Help
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make serve      - Start development server on port 8080"
	@echo "  make serve-dev  - Start development server on port 8081"
	@echo "  make build      - Build WebAssembly application"
	@echo "  make build-prod - Build for production"
	@echo "  make watch      - Watch for changes and rebuild"
	@echo "  make check      - Check code without building"
	@echo "  make fmt        - Format code"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make install    - Install dependencies"
	@echo "  make help       - Show this help message"
