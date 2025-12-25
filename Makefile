# Simplified Makefile for Polyjuice WebAssembly Application

# Default target
.PHONY: all
all: serve

# Build Farcaster SDK bundle
.PHONY: build-farcaster
build-farcaster:
	@echo "Building Farcaster SDK bundle..."
	npm run build:farcaster

# Build the project
.PHONY: build
build: build-farcaster
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
serve: build-farcaster
	@echo "Starting Trunk development server with watch mode..."
	@echo "Server will be available at: http://127.0.0.1:8080"
	@echo "Changes to files will automatically trigger rebuild"
	@echo "Press Ctrl+C to stop the server"
	@echo ""
	@if [ -f .env ]; then \
		echo "🌐 Loading configuration from .env file..."; \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && echo "📡 API Server: $$SNAPRAG_API_URL"; \
		echo ""; \
		export $$(grep -v '^#' .env | grep -v '^$$' | xargs) && unset NO_COLOR && TRUNK_BUILD_NO_SRI=true trunk serve --port 8080 --address 127.0.0.1 --disable-address-lookup --watch .; \
	else \
		echo "⚠️  Warning: .env file not found. Using default API URL: https://snaprag.0xbase.ai"; \
		echo "💡 Tip: Copy .env.example to .env and configure your API URL"; \
		echo ""; \
		unset NO_COLOR && TRUNK_BUILD_NO_SRI=true trunk serve --port 8080 --address 127.0.0.1 --disable-address-lookup --watch .; \
	fi

# Build for production
.PHONY: build-prod
build-prod: build-farcaster
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
	@echo "✅ Production build complete. Integrity checks are enabled for security."

# Build for production deployment (uses snaprag.0xbase.ai)
.PHONY: build-deploy
build-deploy: build-farcaster
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

# Worker commands
.PHONY: worker-build
worker-build:
	@echo "Building Cloudflare Worker..."
	cd worker && cargo build --target wasm32-unknown-unknown --release
	@echo "✅ Worker build complete"

.PHONY: worker-deploy
worker-deploy: worker-build
	@echo "Deploying Cloudflare Worker..."
	@echo "Make sure you have configured wrangler.toml and logged in with 'wrangler login'"
	cd worker && wrangler deploy
	@echo "✅ Worker deployed successfully"

.PHONY: worker-dev
worker-dev:
	@echo "Starting Cloudflare Worker in development mode..."
	@echo "Make sure you have configured wrangler.toml"
	cd worker && wrangler dev

.PHONY: worker-secrets
worker-secrets:
	@echo "Setting Cloudflare Worker secrets..."
	@echo "You will be prompted to enter values for each secret"
	cd worker && \
		echo "Setting BASE_URL..." && wrangler secret put BASE_URL && \
		echo "Setting GITHUB_USERNAME..." && wrangler secret put GITHUB_USERNAME && \
		echo "Setting SOURCE_URL (optional, press Enter to skip)..." && wrangler secret put SOURCE_URL || true
	@echo "✅ Secrets configured"

# Help
.PHONY: help
help:
	@echo "Available commands:"
	@echo ""
	@echo "Frontend commands:"
	@echo "  make serve      - Start dev server with watch mode (http://127.0.0.1:8080)"
	@echo "  make build      - Build WebAssembly application"
	@echo "  make build-prod - Build for production"
	@echo "  make build-deploy - Build for production deployment"
	@echo "  make deploy     - Deploy to GitHub Pages"
	@echo "  make check      - Check code without building"
	@echo "  make clean      - Clean build artifacts"
	@echo ""
	@echo "Worker commands:"
	@echo "  make worker-build   - Build Cloudflare Worker"
	@echo "  make worker-deploy  - Deploy Cloudflare Worker"
	@echo "  make worker-dev     - Start Worker in development mode"
	@echo "  make worker-secrets - Set Worker secrets interactively"
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
	@echo "  make serve         # Start dev server with auto-reload (.env will be loaded)"
	@echo "  make deploy      # Deploy frontend to production"
	@echo "  make worker-deploy # Deploy Cloudflare Worker"