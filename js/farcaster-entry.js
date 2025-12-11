// Farcaster Mini App SDK Entry Point
// This file checks for the Farcaster Mini App SDK and makes it available on window
//
// Note: In a Farcaster Mini App environment (e.g., Warpcast), the SDK is automatically
// injected by the host application. We don't need to load it manually.
// This script only checks if it's available and makes it accessible.

if (typeof window !== 'undefined') {
  function setupFarcasterSDK() {
    // Check if SDK is already available (injected by Farcaster client)
    // The SDK is typically available as window.sdk in Mini App environments
    if (typeof window.sdk !== 'undefined' && window.sdk !== null) {
      // Ensure it's also available as window.farcaster for consistency
      if (typeof window.farcaster === 'undefined') {
        window.farcaster = window.sdk;
      }
      console.log('✅ Farcaster Mini App SDK found on window.sdk');
      return true;
    }

    // Check alternative locations
    if (typeof window.farcaster !== 'undefined' && window.farcaster !== null) {
      // Make it available as window.sdk for consistency
      if (typeof window.sdk === 'undefined') {
        window.sdk = window.farcaster;
      }
      console.log('✅ Farcaster SDK found on window.farcaster');
      return true;
    }

    // Check for ReactNativeWebView (indicates Mini App environment)
    // Even if SDK isn't loaded yet, we're in a Mini App environment
    if (typeof window.ReactNativeWebView !== 'undefined') {
      console.log('📱 Mini App environment detected (ReactNativeWebView found)');
      console.log('ℹ️ Waiting for Farcaster SDK to be injected by host...');
      // SDK will be injected by the host, so we'll just wait
      return false;
    }

    // Not in Mini App environment - this is normal for regular browser usage
    console.log('🌐 Running in regular browser (not a Mini App)');
    return false;
  }

  // Try to setup immediately
  if (!setupFarcasterSDK()) {
    // If not found immediately, wait a bit and try again
    // (SDK might be injected asynchronously)
    setTimeout(() => {
      setupFarcasterSDK();
    }, 100);
  }
}

