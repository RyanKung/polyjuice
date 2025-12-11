// Farcaster Mini App SDK Entry Point
// This file loads the Farcaster Mini App SDK and exports it to window

if (typeof window !== 'undefined') {
  // Import the SDK - it should be available as a UMD or ES module
  // The SDK is typically imported from @farcaster/miniapp-sdk
  try {
    // Try to use dynamic import if available
    if (typeof import !== 'undefined') {
      import('@farcaster/miniapp-sdk').then((module) => {
        // The SDK exports a default sdk object
        const sdk = module.default || module.sdk;
        if (sdk) {
          window.sdk = sdk;
          window.farcaster = sdk;
          console.log('✅ Farcaster Mini App SDK loaded via ES module');
        }
      }).catch((err) => {
        console.warn('⚠️ Failed to load Farcaster SDK via ES module:', err);
        // Fallback: try to find it in window if already loaded
        setupFarcasterSDK();
      });
    } else {
      // Fallback for environments without dynamic import
      setupFarcasterSDK();
    }
  } catch (err) {
    console.warn('⚠️ Failed to load Farcaster SDK:', err);
    setupFarcasterSDK();
  }

  function setupFarcasterSDK() {
    // Check if SDK is already available (e.g., loaded by another script)
    // The SDK might be available as window.sdk, window.farcaster, or window.farcasterSDK
    if (typeof window.sdk !== 'undefined') {
      console.log('✅ Farcaster SDK already available on window.sdk');
      return;
    }

    if (typeof window.farcaster !== 'undefined') {
      window.sdk = window.farcaster;
      console.log('✅ Farcaster SDK found on window.farcaster');
      return;
    }

    // If SDK is not available, log a warning
    // In a Mini App environment, the SDK should be injected by the host
    console.warn('⚠️ Farcaster Mini App SDK not found. This is normal if running outside a Mini App.');
  }
}

