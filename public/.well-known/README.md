# Farcaster Mini App Manifest

## Using Farcaster Hosted Manifest

This app now uses **Farcaster Hosted Manifest** instead of self-hosted manifest.

**Hosted Manifest URL**: `https://api.farcaster.xyz/miniapps/hosted-manifest/019b55e6-573a-26ad-d0cf-61e71688bd07`

### Important Note for GitHub Pages

GitHub Pages is a static site hosting service and **does not support HTTP 307 redirects**. To properly redirect `/.well-known/farcaster.json` to the hosted manifest, you have two options:

#### Option 1: Use a CDN/Proxy (Recommended)
Configure your domain (miniapp.polyjuice.io) to use a CDN or reverse proxy (like Cloudflare) that supports HTTP redirects, and set up a 307 redirect from `/.well-known/farcaster.json` to the hosted manifest URL.

#### Option 2: Keep Self-Hosted Manifest
If you prefer to keep the manifest self-hosted, restore the `farcaster.json` file in this directory.

---

## Adding Ownership Verification (accountAssociation)

To verify app ownership, follow these steps:

1. Visit [Farcaster Mini App Manifest Tool](https://farcaster.xyz/~/developers/mini-apps/manifest)
2. Enter domain: `miniapp.polyjuice.io`
3. Fill in app information
4. Generate the `accountAssociation` object (contains `header`, `payload`, and `signature` fields)
5. Add the generated `accountAssociation` object to the `farcaster.json` file

### Example Format

The generated `accountAssociation` object should look like this:

```json
{
  "accountAssociation": {
    "header": "eyJmaWQiOjkxNTIsInR5cGUiOiJjdXN0b2R5Iiwia2V5IjoiMHgwMmVmNzkwRGQ3OTkzQTM1ZkQ4NDdDMDUzRURkQUU5NDBEMDU1NTk2In0",
    "payload": "eyJkb21haW4iOiJtaW5pYXBwLnBvbHlqdWljZS5pbyJ9",
    "signature": "MHgxMGQwZGU4ZGYwZDUwZTdmMGIxN2YxMTU2NDI1MjRmZTY0MTUyZGU4ZGU1MWU0MThiYjU4ZjVmZmQxYjRjNDBiNGVlZTRhNDcwNmVmNjhlMzQ0ZGQ5MDBkYmQyMmNlMmVlZGY5ZGQ0N2JlNWRmNzMwYzUxNjE4OWVjZDJjY2Y0MDFj"
  }
}
```

Add the above object to the root level of `farcaster.json` (at the same level as `miniapp`).

## Configuring Webhook URL

The `webhookUrl` is used to receive events from Farcaster clients. When users interact with your Mini App, Farcaster clients will send HTTP POST requests to this URL.

### How it works:

1. **Purpose**: Receive real-time notifications and events from Farcaster clients
2. **When triggered**: Farcaster clients POST events to your webhook URL when:
   - Users interact with your Mini App
   - Notifications are sent to users
   - Specific events occur in your app

3. **Setup**:
   - Replace `https://YOUR_WEBHOOK_URL_HERE` in the `webhookUrl` field with your actual webhook endpoint URL
   - Your server must be able to receive and handle HTTP POST requests
   - The endpoint should return a 200 status code to acknowledge receipt

4. **Requirements**:
   - Must be set if your Mini App uses notifications
   - Must be a valid HTTPS URL (max 1024 characters)
   - Your server must handle POST requests with JSON payloads

### Example webhook endpoint:

Your webhook endpoint should handle POST requests like this:

```javascript
// Example Express.js webhook handler
app.post('/webhook', (req, res) => {
  const event = req.body;
  console.log('Received event:', event);
  
  // Process the event (e.g., notification, user action, etc.)
  // ...
  
  res.status(200).json({ received: true });
});
```

For more details, refer to the [Farcaster Mini Apps documentation](https://miniapps.farcaster.xyz/docs/guides/sending-notifications).
