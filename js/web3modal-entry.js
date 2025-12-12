// Web3Modal Entry Point
// This file imports and exports Web3Modal to window

if (typeof window !== 'undefined') {
  try {
    // Import Web3Modal from the package
    const { createWeb3Modal, defaultConfig } = require('@web3modal/ethers');
    
    // Export to window for Rust to access
    window.Web3Modal = {
      createWeb3Modal,
      defaultConfig,
    };
    console.log('✅ Web3Modal loaded');
  } catch (e) {
    console.warn('⚠️ Failed to load Web3Modal:', e);
  }
}

