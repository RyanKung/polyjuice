// WalletConnect v2 Bundle Entry Point
// This file loads the UMD build and exports EthereumProvider to window

// The UMD file is loaded by a script tag in index.html before this file
// UMD bundles typically create a module namespace, we need to extract EthereumProvider

if (typeof window !== 'undefined') {
  function setupWalletConnect() {
    // UMD bundles typically export to module.exports or a global namespace
    // Check the actual UMD export - it should be available after the script loads
    let EthereumProvider;
    
    // The UMD file exports to a module object, check if it's available
    // UMD pattern: (function(e){...})(typeof exports !== 'undefined' ? exports : {})
    // So it might be on window with a specific name, or in a module namespace
    
    // Try to find it in common UMD export locations
    // Pattern 1: Direct on window (if UMD set it)
    if (typeof window.EthereumProvider !== 'undefined') {
      EthereumProvider = window.EthereumProvider;
    }
    // Pattern 2: Check if there's a module namespace created by UMD
    // UMD files often create a global variable with the library name
    else {
      // Check all window properties that might contain EthereumProvider
      for (const key in window) {
        try {
          const obj = window[key];
          if (obj && typeof obj === 'object' && obj.EthereumProvider) {
            EthereumProvider = obj.EthereumProvider;
            console.log('Found EthereumProvider in window.' + key);
            break;
          }
        } catch (e) {
          // Skip if we can't access the property
        }
      }
    }
    
    if (EthereumProvider) {
      // Ensure it's available on window for Rust to access
      window.EthereumProvider = EthereumProvider;
      window.WalletConnect = {
        EthereumProvider,
      };
      console.log('✅ WalletConnect v2 bundle loaded');
      return true;
    }
    return false;
  }
  
  // Try immediately (UMD should be loaded by now since it's loaded before this script)
  if (!setupWalletConnect()) {
    // If not found, wait a bit and try again (in case of async loading)
    setTimeout(() => {
      if (!setupWalletConnect()) {
        console.warn('⚠️ WalletConnect UMD bundle not found. Make sure walletconnect-umd.js is loaded before this script.');
        // Debug: log all window properties that might be relevant
        const relevantKeys = Object.keys(window).filter(k => 
          k.toLowerCase().includes('wallet') || 
          k.toLowerCase().includes('ethereum') ||
          k.toLowerCase().includes('wc')
        );
        if (relevantKeys.length > 0) {
          console.warn('Relevant window properties:', relevantKeys);
        }
      }
    }, 200);
  }
}

