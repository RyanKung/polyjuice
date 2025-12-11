# Simplified Makefile for Polyjuice WebAssembly Application

# Default target
.PHONY: all
all: serve

# Build WalletConnect bundle
.PHONY: build-walletconnect
build-walletconnect:
	@echo "Building WalletConnect bundle..."
	npm run build:walletconnect

# Build the project
.PHONY: build
build: build-walletconnect
	@echo "Building WebAssembly application..."
	cargo build --target wasm32-unknown-unknown

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf dist/

# Serve the application with Trunk (with watch mode)
.PHONY: serve
serve: build-walletconnect
	@echo "Starting Trunk development server with watch mode..."
	@echo "Server will be available at: http://127.0.0.1:8080"
	@echo "Changes to files will automatically trigger rebuild"
	@echo "Press Ctrl+C to stop the server"
	@echo ""
	@if [ -f .env ]; then \
		echo "🌐 Loading configuration from .env file..."; \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && echo "📡 API Server: $$SNAPRAG_API_URL"; \
		echo ""; \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && unset NO_COLOR && trunk serve --port 8080 --address 127.0.0.1 --disable-address-lookup --watch .; \
	else \
		echo "⚠️  Warning: .env file not found. Using default API URL: https://snaprag.0xbase.ai"; \
		echo "💡 Tip: Copy .env.example to .env and configure your API URL"; \
		echo ""; \
		unset NO_COLOR && trunk serve --port 8080 --address 127.0.0.1 --disable-address-lookup --watch .; \
	fi

# Build for production
.PHONY: build-prod
build-prod: build-walletconnect
	@echo "Building for production..."
	@if [ -f .env ]; then \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && echo "API URL: $$SNAPRAG_API_URL"; \
	else \
		echo "API URL: using default or environment variable"; \
	fi
	@if [ -f .env ]; then \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && unset NO_COLOR && trunk build --release; \
	else \
		unset NO_COLOR && trunk build --release; \
	fi

# Build for production deployment (uses snaprag.0xbase.ai)
.PHONY: build-deploy
build-deploy: build-walletconnect
	@echo "Building for production deployment..."
	@echo "Using API URL: https://snaprag.0xbase.ai/"
	SNAPRAG_API_URL=https://snaprag.0xbase.ai/ unset NO_COLOR && trunk build --release

# Deploy to GitHub Pages
.PHONY: deploy
deploy: build-deploy
	@echo "Deploying to GitHub Pages..."
	@echo "Make sure you have committed all changes before deploying"
	git add .
	git commit -m "Deploy Polyjuice to GitHub Pages" || true
	git push origin master
	@echo "Deployment triggered! Check GitHub Actions for progress."

# Check code without building
.PHONY: check
check:
	@echo "Checking code..."
	cargo check

# Help
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make serve      - Start dev server with watch mode (http://127.0.0.1:8080)"
	@echo "  make build      - Build WebAssembly application"
	@echo "  make build-prod - Build for production"
	@echo "  make build-deploy - Build for production deployment"
	@echo "  make deploy     - Deploy to GitHub Pages"
	@echo "  make check      - Check code without building"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make help       - Show this help message"
	@echo ""
	@echo "Environment variables:"
	@echo "  SNAPRAG_API_URL - API server URL (can be set in .env file)"
	@echo ""
	@echo "Configuration:"
	@echo "  1. Copy .env.example to .env"
	@echo "  2. Edit .env to set SNAPRAG_API_URL"
	@echo "  3. Run 'make serve' to load configuration"
	@echo ""
	@echo "Examples:"
	@echo "  make serve      # Start dev server with auto-reload (.env will be loaded)"
	@echo "  make deploy     # Deploy to production"