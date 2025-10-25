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
	@echo "API URL: $${SNAPRAG_API_URL:-http://127.0.0.1:3000}"
	unset NO_COLOR && trunk build --release

# Build for production with custom API URL
.PHONY: build-prod-custom
build-prod-custom:
	@echo "Building for production with custom API URL..."
	@if [ -z "$$SNAPRAG_API_URL" ]; then \
		echo "Error: SNAPRAG_API_URL environment variable is required"; \
		echo "Usage: SNAPRAG_API_URL=https://your-api.com make build-prod-custom"; \
		exit 1; \
	fi
	@echo "Using API URL: $$SNAPRAG_API_URL"
	SNAPRAG_API_URL=$$SNAPRAG_API_URL unset NO_COLOR && trunk build --release

# Build for production deployment (uses snaprag.0xbase.ai)
.PHONY: build-deploy
build-deploy:
	@echo "Building for production deployment..."
	@echo "Using API URL: https://snaprag.0xbase.ai/"
	SNAPRAG_API_URL=https://snaprag.0xbase.ai/ unset NO_COLOR && trunk build --release

# Deploy to GitHub Pages (local build + git push)
.PHONY: deploy
deploy: build-deploy
	@echo "Deploying to GitHub Pages..."
	@echo "Make sure you have committed all changes before deploying"
	git add .
	git commit -m "Deploy Polyjuice to GitHub Pages" || true
	git push origin master
	@echo "Deployment triggered! Check GitHub Actions for progress."

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
	@echo "  make build-prod - Build for production (uses SNAPRAG_API_URL if set)"
	@echo "  make build-prod-custom - Build for production with required SNAPRAG_API_URL"
	@echo "  make build-deploy - Build for production deployment (uses snaprag.0xbase.ai)"
	@echo "  make deploy     - Deploy to GitHub Pages (build + git push)"
	@echo "  make watch      - Watch for changes and rebuild"
	@echo "  make check      - Check code without building"
	@echo "  make fmt        - Format code"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make install    - Install dependencies"
	@echo "  make help       - Show this help message"
	@echo ""
	@echo "Environment variables:"
	@echo "  SNAPRAG_API_URL - API server URL (default: http://127.0.0.1:3000)"
	@echo ""
	@echo "Deployment:"
	@echo "  make deploy     - Deploy to polyjuice.0xbase.ai via GitHub Pages"
	@echo ""
	@echo "Examples:"
	@echo "  SNAPRAG_API_URL=https://api.example.com make build-prod-custom"
	@echo "  SNAPRAG_API_URL=http://192.168.1.100:3000 make serve"
	@echo "  make deploy     # Deploy to production"
