// WalletConnect Modal Entry Point
// This file imports and exports WalletConnectModal to window

if (typeof window !== 'undefined') {
  try {
    // Import WalletConnectModal from the package
    const { WalletConnectModal } = require('@walletconnect/modal');
    
    // Export to window for Rust to access
    window.WalletConnectModal = WalletConnectModal;
    console.log('✅ WalletConnectModal loaded');
  } catch (e) {
    console.warn('⚠️ Failed to load WalletConnectModal:', e);
  }
}

