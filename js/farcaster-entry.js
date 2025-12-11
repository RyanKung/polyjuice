// Farcaster Mini App SDK Entry Point
// This file loads the Farcaster Mini App SDK and makes it available on window
//
// Note: In a Farcaster Mini App environment (e.g., Warpcast), the SDK is automatically
// injected by the host application. We need to wait for it and make it accessible.
// The official way to detect Mini App is: await sdk.isInMiniApp()

if (typeof window !== 'undefined') {
  // Try to load SDK from @farcaster/miniapp-sdk package
  // The SDK should be available as window.sdk or we need to import it
  async function loadFarcasterSDK() {
    console.log('[Farcaster SDK] Attempting to load SDK...');
    
    // First, check if SDK is already available (injected by Farcaster client)
    if (typeof window.sdk !== 'undefined' && window.sdk !== null) {
      console.log('[Farcaster SDK] ✅ SDK found on window.sdk');
      if (typeof window.farcaster === 'undefined') {
        window.farcaster = window.sdk;
      }
      return window.sdk;
    }

    if (typeof window.farcaster !== 'undefined' && window.farcaster !== null) {
      console.log('[Farcaster SDK] ✅ SDK found on window.farcaster');
      if (typeof window.sdk === 'undefined') {
        window.sdk = window.farcaster;
      }
      return window.farcaster;
    }

    // Try to import from @farcaster/miniapp-sdk if available
    // This is a fallback if SDK is not injected by the host
    try {
      if (typeof import !== 'undefined') {
        const module = await import('@farcaster/miniapp-sdk');
        const sdk = module.default || module.sdk;
        if (sdk) {
          console.log('[Farcaster SDK] ✅ SDK loaded from @farcaster/miniapp-sdk package');
          window.sdk = sdk;
          window.farcaster = sdk;
          return sdk;
        }
      }
    } catch (e) {
      console.log('[Farcaster SDK] Could not import from @farcaster/miniapp-sdk:', e.message);
    }

    return null;
  }

  // Try to load SDK with retries
  let attempts = 0;
  const maxAttempts = 15;
  const retryInterval = 300;

  async function tryLoadSDK() {
    attempts++;
    console.log(`[Farcaster SDK] Attempt ${attempts}/${maxAttempts}: Loading SDK...`);
    
    const sdk = await loadFarcasterSDK();
    if (sdk) {
      console.log('[Farcaster SDK] ✅ SDK loaded successfully');
      return;
    }

    if (attempts < maxAttempts) {
      setTimeout(tryLoadSDK, retryInterval);
    } else {
      console.log('[Farcaster SDK] ⚠️ SDK not found after all attempts');
      console.log('[Farcaster SDK] This is normal if running outside a Mini App environment');
    }
  }

  // Start loading
  tryLoadSDK();
}

