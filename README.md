# Polyjuice

<div align="center">
  <img src="logo.png" alt="Polyjuice Logo" width="200" height="200">
  
  **Discover & Chat with Farcaster Users**
  
  [![Deploy Status](https://github.com/RyanKung/polyjuice/workflows/Deploy%20Polyjuice%20to%20GitHub%20Pages/badge.svg)](https://github.com/RyanKung/polyjuice/actions)
  [![Live Demo](https://img.shields.io/badge/Live%20Demo-polyjuice.0xbase.ai-blue)](https://polyjuice.0xbase.ai)
  [![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange)](https://www.rust-lang.org/)
  [![WebAssembly](https://img.shields.io/badge/WebAssembly-WASM-purple)](https://webassembly.org/)
</div>

## 🎯 Overview

Polyjuice is a beautiful, modern web application that provides a search engine-like interface for discovering and interacting with Farcaster users. Built with Rust and WebAssembly, it offers lightning-fast performance and a seamless user experience.

### ✨ Key Features

- 🔍 **Smart Search**: Search users by FID or username with instant results
- 💬 **AI Chat**: Interactive chat sessions with users powered by SnapRAG
- 📊 **Social Analytics**: Deep insights into user behavior and social circles
- 💳 **Web3 Payments**: Integrated x402 payment system for premium features
- 📱 **Responsive Design**: Perfect experience on desktop and mobile
- ⚡ **Lightning Fast**: Built with Rust WebAssembly for optimal performance
- 🎨 **Modern UI**: Clean, Google-inspired interface with smooth animations

## 🚀 Live Demo

Visit **[polyjuice.0xbase.ai](http://polyjuice.0xbase.ai)** to try Polyjuice right now!

## 🛠️ Technology Stack

- **Frontend**: [Yew](https://yew.rs/) (Rust WebAssembly framework)
- **Build Tool**: [Trunk](https://trunkrs.dev/)
- **Styling**: Pure CSS with modern design principles
- **API**: RESTful integration with SnapRAG backend
- **Payments**: x402 protocol for Web3 payments
- **Deployment**: GitHub Pages with GitHub Actions

## 📖 How to Use

### Basic Search
1. **Enter a FID**: Type any Farcaster ID (e.g., 1, 2, 3, 100, 1000) in the search box
2. **Search**: Click the search button or press Enter
3. **View Results**: See detailed user information including:
   - Profile picture and display name
   - Username and FID
   - Bio/description
   - Social analytics and influence scores
   - Social circles and interaction patterns

### AI Chat
1. **Search for a user** to load their profile
2. **Click the chat button** 💭 to start an AI-powered conversation
3. **Ask questions** about the user's activity, interests, or opinions
4. **Get contextual responses** based on their Farcaster history

### Web3 Payments
1. **Connect MetaMask** wallet for premium features
2. **Automatic payment prompts** for paid API calls
3. **Secure EIP-712 signatures** for payment authorization
4. **Transparent pricing** with x402 protocol

## 🏗️ Development

### Prerequisites

- **Rust** (nightly toolchain)
- **Trunk** (Rust WASM build tool)
- **Node.js** (for JavaScript wallet integration)

### Quick Start

1. **Clone the repository**:
```bash
git clone https://github.com/RyanKung/polyjuice.git
cd polyjuice
```

2. **Install Trunk**:
```bash
cargo install trunk
```

3. **Start development server**:
```bash
make serve
```

4. **Open your browser** to `http://localhost:8080`

### Available Commands

```bash
# Development
make serve          # Start dev server on port 8080
make serve-dev      # Start dev server on port 8081
make watch          # Watch for changes and rebuild

# Building
make build          # Build WebAssembly application
make build-prod     # Build for production
make build-deploy   # Build for deployment (uses snaprag.0xbase.ai)

# Code Quality
make check          # Check code without building
make fmt            # Format code
make clippy         # Run clippy linter

# Deployment
make deploy         # Deploy to GitHub Pages
make clean          # Clean build artifacts
make help           # Show all commands
```

### Environment Configuration

Configure the API server URL and WalletConnect projectId using **one of two methods**:

#### Method 1: Build-Time Environment Variable
Set environment variable during build:
```bash
# Development with custom API URL and WalletConnect projectId
SNAPRAG_API_URL=http://192.168.1.100:3000 WALLETCONNECT_PROJECT_ID=your_project_id make serve

# Production build with custom API URL and WalletConnect projectId
SNAPRAG_API_URL=https://api.yourdomain.com WALLETCONNECT_PROJECT_ID=your_project_id make build-prod
```

#### Method 2: .env File (Build-time)
1. **Create `.env` file** (if it doesn't exist):
```bash
touch .env
```

2. **Edit `.env`** to set your configuration:
```bash
SNAPRAG_API_URL=http://127.0.0.1:3000
WALLETCONNECT_PROJECT_ID=your_project_id_here
```

3. **Run**:
```bash
make serve
```

**Note**: The `.env` file is automatically loaded when running `make serve` or `make build-prod`.

#### Getting WalletConnect Project ID

To get your WalletConnect `projectId`:

1. **Visit [WalletConnect Cloud](https://cloud.walletconnect.com)**
2. **Sign up or log in** to your account
3. **Create a new project**:
   - Click "Create Project" or "New Project"
   - Enter your project name (e.g., "Polyjuice")
   - Enter your website URL (e.g., `http://127.0.0.1:8080` for development)
   - Submit the form
4. **Copy your Project ID**:
   - After creating the project, you'll see your unique `projectId` in the dashboard
   - It looks like: `a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`
5. **Add it to your `.env` file**:
   ```bash
   WALLETCONNECT_PROJECT_ID=a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
   ```

**Important**: 
- While WalletConnect can work without a valid projectId for basic functionality, you may see 403 errors when fetching wallet lists and analytics.
- **403 errors and empty wallet list**: If you see 403 errors in the browser console and the wallet list is empty, this is likely because:
  1. Your domain is not added to the allowed domains list in WalletConnect Cloud
  2. The projectId is not properly configured in WalletConnect Cloud
  3. The projectId in your `.env` file doesn't match the one in WalletConnect Cloud
  
  **To fix this**:
  1. Go to [WalletConnect Cloud](https://cloud.walletconnect.com)
  2. Select your project
  3. Go to "Settings" or "Project Settings"
  4. Add your domain (e.g., `127.0.0.1:8080` for development, or your production domain) to the "Allowed Domains" list
  5. Save the changes
  6. Rebuild and restart your application
  
- For production use, a valid projectId and proper domain configuration are required.

## 🚀 Deployment

### Automatic Deployment

Polyjuice is automatically deployed to GitHub Pages at `polyjuice.0xbase.ai` when changes are pushed to the master branch.

### Manual Deployment

```bash
# Deploy to production
make deploy
```

This will:
1. Build the application with production API URL (`https://snaprag.0xbase.ai/`)
2. Commit changes to git
3. Push to master branch
4. Trigger GitHub Actions deployment

### GitHub Actions Workflow

The deployment process includes:
- **Rust + Trunk build** with WASM optimization
- **Automatic API configuration** for production
- **GitHub Pages deployment** with custom domain
- **Caching** for faster builds

## 🔌 API Integration

Polyjuice integrates with the SnapRAG API to fetch user data:

- **Base URL**: `https://snaprag.0xbase.ai/` (production)
- **Endpoints**: Multiple RESTful endpoints for profiles, social data, and chat
- **Authentication**: x402 payment protocol for premium features
- **Response Format**: JSON with comprehensive user data

### Supported Endpoints

- `/api/health` - Health check
- `/api/profiles/{fid}` - User profile data
- `/api/social/{fid}` - Social analytics
- `/api/search/profiles` - Semantic profile search
- `/api/search/casts` - Semantic cast search
- `/api/rag/query` - AI-powered RAG queries
- `/api/chat/create` - Create chat sessions
- `/api/chat/message` - Send chat messages

## 🎨 Design Philosophy

Polyjuice embodies modern web design principles:

- **Minimalism**: Clean, uncluttered interface focusing on essential elements
- **Performance**: Rust WebAssembly for lightning-fast user experience
- **Accessibility**: Responsive design that works on all devices
- **User-Centric**: Intuitive navigation and clear information hierarchy

## 📊 Example Usage

Try searching for these popular Farcaster users:

- **@vitalik.eth** - Ethereum founder
- **@jesse.base.eth** - Base protocol lead
- **@ryankung.base.eth** - Developer and builder
- **FID 1, 2, 3** - Early Farcaster users

## 🤝 Contributing

We welcome contributions! Please ensure all code follows our standards:

- **Written in Rust** with proper error handling
- **Fully documented** with clear comments
- **Security-focused** with input validation
- **Performance-optimized** for WebAssembly

### Development Guidelines

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Run tests and linting**
5. **Submit a pull request**

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- **Live Demo**: [polyjuice.0xbase.ai](http://polyjuice.0xbase.ai)
- **SnapRAG API**: [snaprag.0xbase.ai](https://snaprag.0xbase.ai)
- **GitHub Repository**: [github.com/RyanKung/polyjuice](https://github.com/RyanKung/polyjuice)
- **Deployment Status**: [GitHub Actions](https://github.com/RyanKung/polyjuice/actions)

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/RyanKung/polyjuice/issues)
- **Discussions**: [GitHub Discussions](https://github.com/RyanKung/polyjuice/discussions)

---

<div align="center">
  <p>Built with ❤️ and Rust by the 0xbase.ai team</p>
  <p>Part of the <a href="https://github.com/RyanKung/snaprag">SnapRAG</a> ecosystem</p>
</div>