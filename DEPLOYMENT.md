# Polyjuice Deployment Guide

## Overview

Polyjuice is deployed to GitHub Pages at `polyjuice.0xbase.ai` using GitHub Actions. The deployment process automatically builds the application with the production API URL pointing to `https://snaprag.0xbase.ai/`.

## Deployment Process

### Automatic Deployment

When code is pushed to the `master` branch, GitHub Actions automatically:

1. **Builds** the WebAssembly application using Trunk
2. **Configures** the API URL to `https://snaprag.0xbase.ai/`
3. **Deploys** to GitHub Pages
4. **Updates** the live site at `polyjuice.0xbase.ai`

### Manual Deployment

Use the Makefile command for manual deployment:

```bash
make deploy
```

This command will:
- Build the application for production
- Commit changes to git
- Push to the master branch
- Trigger the GitHub Actions deployment

### Build Commands

```bash
# Development build (default localhost API)
make build-prod

# Custom API URL build
SNAPRAG_API_URL=https://api.example.com make build-prod-custom

# Production deployment build (uses snaprag.0xbase.ai)
make build-deploy
```

## Configuration Files

### GitHub Actions Workflow

The deployment workflow is defined in `.github/workflows/deploy.yml`:

- **Trigger**: Push to `master` or `main` branch
- **Build**: Rust + Trunk with WASM target
- **API URL**: Hardcoded to `https://snaprag.0xbase.ai/`
- **Deploy**: GitHub Pages with custom domain

### Custom Domain

The `CNAME` file specifies the custom domain:
```
polyjuice.0xbase.ai
```

### Environment Variables

- **SNAPRAG_API_URL**: API server URL (default: `http://127.0.0.1:3000`)
- **Production**: Always uses `https://snaprag.0xbase.ai/`

## GitHub Pages Setup

To enable GitHub Pages for this repository:

1. Go to repository **Settings**
2. Navigate to **Pages** section
3. Set **Source** to "GitHub Actions"
4. The workflow will automatically deploy to Pages

## Troubleshooting

### Build Failures

- Check Rust toolchain compatibility
- Verify Trunk installation
- Review GitHub Actions logs

### Deployment Issues

- Ensure GitHub Pages is enabled
- Check CNAME file is present
- Verify repository permissions

### API Connection Issues

- Confirm `https://snaprag.0xbase.ai/` is accessible
- Check CORS configuration on API server
- Review browser console for errors

## Development vs Production

| Environment | API URL | Build Command |
|-------------|---------|---------------|
| Development | `http://127.0.0.1:3000` | `make serve` |
| Production | `https://snaprag.0xbase.ai/` | `make deploy` |

## Security Considerations

- API URL is hardcoded in production build
- No sensitive data in client-side code
- GitHub Actions uses secure token authentication
- Custom domain requires DNS configuration
