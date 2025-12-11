// Farcaster Mini App SDK Entry Point
// This file checks for the Farcaster Mini App SDK and makes it available on window
//
// Note: In a Farcaster Mini App environment (e.g., Warpcast), the SDK is automatically
// injected by the host application. We don't need to load it manually.
// This script only checks if it's available and makes it accessible.

if (typeof window !== 'undefined') {
  function setupFarcasterSDK(attempt = 0) {
    console.log(`[Farcaster SDK Detection] Attempt ${attempt + 1}: Checking for SDK...`);
    
    // Official method: Check if window.farcaster property exists
    // According to Farcaster docs, property existence indicates Mini App environment
    // Even if null initially, the property will be populated by the host
    const farcasterPropExists = 'farcaster' in window;
    const sdkPropExists = 'sdk' in window;
    
    console.log(`[Farcaster SDK Detection] Property check: window.farcaster=${farcasterPropExists}, window.sdk=${sdkPropExists}`);
    
    // Check if SDK is already available (injected by Farcaster client)
    // The SDK is typically available as window.sdk in Mini App environments
    console.log(`[Farcaster SDK Detection] Checking window.sdk: ${typeof window.sdk !== 'undefined' ? 'exists' : 'undefined'}`);
    if (typeof window.sdk !== 'undefined' && window.sdk !== null) {
      // Ensure it's also available as window.farcaster for consistency
      if (typeof window.farcaster === 'undefined') {
        window.farcaster = window.sdk;
      }
      console.log('✅ Farcaster Mini App SDK found on window.sdk');
      console.log('[Farcaster SDK Detection] SDK type:', typeof window.sdk);
      console.log('[Farcaster SDK Detection] SDK keys:', Object.keys(window.sdk || {}).slice(0, 10));
      return true;
    }

    // Check alternative locations
    console.log(`[Farcaster SDK Detection] Checking window.farcaster: ${typeof window.farcaster !== 'undefined' ? 'exists' : 'undefined'}`);
    if (typeof window.farcaster !== 'undefined' && window.farcaster !== null) {
      // Make it available as window.sdk for consistency
      if (typeof window.sdk === 'undefined') {
        window.sdk = window.farcaster;
      }
      console.log('✅ Farcaster SDK found on window.farcaster');
      console.log('[Farcaster SDK Detection] SDK type:', typeof window.farcaster);
      console.log('[Farcaster SDK Detection] SDK keys:', Object.keys(window.farcaster || {}).slice(0, 10));
      return true;
    }

    // Check for other Farcaster Mini App indicators
    // Some implementations might use different global objects
    console.log(`[Farcaster SDK Detection] Checking window.farcasterSDK: ${typeof window.farcasterSDK !== 'undefined' ? 'exists' : 'undefined'}`);
    if (typeof window.farcasterSDK !== 'undefined' && window.farcasterSDK !== null) {
      window.sdk = window.farcasterSDK;
      window.farcaster = window.farcasterSDK;
      console.log('✅ Farcaster SDK found on window.farcasterSDK');
      console.log('[Farcaster SDK Detection] SDK type:', typeof window.farcasterSDK);
      console.log('[Farcaster SDK Detection] SDK keys:', Object.keys(window.farcasterSDK || {}).slice(0, 10));
      return true;
    }


    // Check for ReactNativeWebView (indicates Mini App environment)
    // Even if SDK isn't loaded yet, we're in a Mini App environment
    console.log(`[Farcaster SDK Detection] Checking window.ReactNativeWebView: ${typeof window.ReactNativeWebView !== 'undefined' ? 'exists' : 'undefined'}`);
    if (typeof window.ReactNativeWebView !== 'undefined') {
      console.log('📱 Mini App environment detected (ReactNativeWebView found)');
      console.log('ℹ️ Waiting for Farcaster SDK to be injected by host...');
      // SDK will be injected by the host, so we'll just wait
      return false;
    }

    // Debug: Log all window properties that might be relevant
    if (attempt === 0 || attempt === 9) {
      const relevantKeys = Object.keys(window).filter(k => 
        k.toLowerCase().includes('farcaster') || 
        k.toLowerCase().includes('sdk') ||
        k.toLowerCase().includes('miniapp') ||
        k.toLowerCase().includes('mini') ||
        k === 'sdk' ||
        k.startsWith('__') ||
        k.includes('Farcaster')
      );
      if (relevantKeys.length > 0) {
        console.log('[Farcaster SDK Detection] Relevant window properties found:', relevantKeys);
        // Log the values of these properties
        relevantKeys.forEach(key => {
          const value = window[key];
          console.log(`[Farcaster SDK Detection] window.${key}:`, typeof value, value === null ? 'null' : value === undefined ? 'undefined' : 'object');
          if (value && typeof value === 'object' && !Array.isArray(value)) {
            try {
              const keys = Object.keys(value).slice(0, 10);
              console.log(`[Farcaster SDK Detection] window.${key} keys:`, keys);
            } catch (e) {
              // Ignore errors accessing keys
            }
          }
        });
      }
      
      // Also check for nested objects
      const nestedChecks = ['__farcaster__', '__farcasterSDK__', 'FarcasterSDK', 'Farcaster'];
      nestedChecks.forEach(path => {
        if (window[path] && typeof window[path] === 'object') {
          console.log(`[Farcaster SDK Detection] Found window.${path}:`, typeof window[path]);
          try {
            if (window[path].sdk) {
              console.log(`[Farcaster SDK Detection] ✅ Found nested SDK in window.${path}.sdk`);
            }
          } catch (e) {
            // Ignore
          }
        }
      });
    }

    // If properties exist but SDK is null, we're likely in Mini App waiting for injection
    if (farcasterPropExists || sdkPropExists) {
      console.log('📱 Mini App environment detected (properties exist but SDK not injected yet)');
      console.log('ℹ️ Waiting for Farcaster SDK to be injected by host...');
      return false; // Will retry
    }

    // Not in Mini App environment - this is normal for regular browser usage
    if (attempt === 0) {
      console.log('🌐 Running in regular browser (not a Mini App)');
    }
    return false;
  }

  // Try to setup immediately
  console.log('[Farcaster SDK Detection] Starting SDK detection...');
  if (!setupFarcasterSDK(0)) {
    // If not found immediately, wait and retry multiple times
    // (SDK might be injected asynchronously by the host)
    let attempts = 0;
    const maxAttempts = 10;
    const retryInterval = 200; // 200ms between attempts
    
    console.log(`[Farcaster SDK Detection] SDK not found immediately, will retry ${maxAttempts} times with ${retryInterval}ms intervals`);
    
    const retrySetup = () => {
      attempts++;
      console.log(`[Farcaster SDK Detection] Retry ${attempts}/${maxAttempts}`);
      if (setupFarcasterSDK(attempts)) {
        console.log(`[Farcaster SDK Detection] ✅ SDK found on attempt ${attempts + 1}`);
        return; // Found, stop retrying
      }
      
      if (attempts < maxAttempts) {
        setTimeout(retrySetup, retryInterval);
      } else {
        // After max attempts, check one more time for ReactNativeWebView
        // If it exists, we're in Mini App but SDK might not be available yet
        console.log(`[Farcaster SDK Detection] ❌ SDK not found after ${maxAttempts} attempts`);
        if (typeof window.ReactNativeWebView !== 'undefined') {
          console.log('📱 Mini App environment detected but SDK not found after ' + maxAttempts + ' attempts');
          console.log('ℹ️ SDK may be injected later by the host');
        } else {
          console.log('🌐 No Mini App environment indicators found');
        }
      }
    };
    
    setTimeout(retrySetup, retryInterval);
  } else {
    console.log('[Farcaster SDK Detection] ✅ SDK found immediately');
  }
}

