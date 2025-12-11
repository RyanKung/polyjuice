// Farcaster Mini App SDK Entry Point
// This file imports the Farcaster Mini App SDK and makes it available on window
// Official usage: import { sdk } from '@farcaster/miniapp-sdk'

import { sdk } from '@farcaster/miniapp-sdk';

if (typeof window !== 'undefined') {
  // Export SDK to window for Rust/WASM to access
  window.sdk = sdk;
  window.farcaster = sdk;
  
  console.log('[Farcaster SDK] ✅ SDK imported and available on window.sdk');
  console.log('[Farcaster SDK] SDK type:', typeof sdk);
  
  // Verify SDK has isInMiniApp method
  if (typeof sdk.isInMiniApp === 'function') {
    console.log('[Farcaster SDK] ✅ SDK.isInMiniApp() method available');
  } else {
    console.warn('[Farcaster SDK] ⚠️ SDK.isInMiniApp() method not found');
  }
}

