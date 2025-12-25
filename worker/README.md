# Polyjuice Cloudflare Worker

This Cloudflare Worker dynamically injects meta tags for Farcaster shares, specifically for annual report routes. It detects Farcaster crawlers and injects appropriate meta tags with tarot card preview images.

## Features

- Detects Farcaster crawlers/bots via User-Agent
- Extracts FID from `/annual-report/{fid}` URLs
- Calculates tarot card based on FID (using same algorithm as frontend)
- Injects `fc:miniapp`, `fc:frame`, and Open Graph meta tags
- Uses tarot card image as preview for annual report shares

## Setup

### Prerequisites

1. Install Wrangler CLI:
```bash
npm install -g wrangler
```

2. Login to Cloudflare:
```bash
wrangler login
```

### Configuration

1. Copy the example configuration file (if you don't have `wrangler.toml`):
```bash
cp wrangler.example.toml wrangler.toml
```

2. Edit `wrangler.toml` and update:
   - `BASE_URL`: Your production URL (default: `https://miniapp.polyjuice.io`)
   - `SOURCE_URL`: (Optional) Custom source URL for fetching content. If not set, will use GitHub Pages format: `https://{GITHUB_USERNAME}.github.io`
   - `GITHUB_USERNAME`: Your GitHub username (used if `SOURCE_URL` is not set)

**Note**: `wrangler.toml` should be committed to Git (it doesn't contain sensitive information). Use `wrangler secret put` for sensitive values.

2. Set secrets (optional, if not using vars in wrangler.toml):
```bash
cd worker
wrangler secret put BASE_URL
wrangler secret put SOURCE_URL  # Optional: for custom source URL
wrangler secret put GITHUB_USERNAME
```

### Build and Deploy

```bash
# Build the worker
make worker-build

# Deploy to Cloudflare
make worker-deploy
```

Or manually:
```bash
cd worker
cargo build --target wasm32-unknown-unknown --release
wrangler deploy
```

### Development

To test locally:
```bash
make worker-dev
```

## How It Works

1. **Bot Detection**: Checks User-Agent for Farcaster-related keywords
2. **Route Matching**: Only processes `/annual-report/{fid}` routes for bots
3. **FID Extraction**: Parses FID from URL path (returns 400 error if invalid)
4. **Source Fetching**: Fetches HTML from `SOURCE_URL` or GitHub Pages
5. **Tarot Calculation**: Uses FID hash mod 22 to select tarot card (same as frontend)
6. **Meta Generation**: Creates Farcaster meta tags with tarot card image URL
7. **HTML Injection**: Injects meta tags into `<head>` before `</head>`
8. **Error Handling**: Returns appropriate HTTP status codes (400, 500, 502) for errors

## Meta Tag Format

The worker generates meta tags in this format:

```html
<meta name="fc:miniapp" content='{"version":"1","imageUrl":"https://miniapp.polyjuice.io/imgs/tarot/01-fool.jpg",...}' />
<meta name="fc:frame" content='...' />
<meta property="og:title" content="2025 Annual Report - Polyjuice" />
<meta property="og:image" content="https://miniapp.polyjuice.io/imgs/tarot/01-fool.jpg" />
```

## DNS Configuration

After deploying, configure your domain:

1. In Cloudflare Dashboard → Workers & Pages
2. Configure routes: `miniapp.polyjuice.io/*` → your worker
3. Ensure DNS CNAME points to GitHub Pages with proxy enabled (orange cloud)

## Testing

Test with a bot User-Agent:

```bash
curl -H "User-Agent: farcaster-bot" https://miniapp.polyjuice.io/annual-report/12345
```

Should return HTML with injected meta tags.

