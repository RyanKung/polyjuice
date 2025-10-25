/**
 * Multi-Wallet Integration
 * Supports MetaMask, WalletConnect, and other wallet providers
 */

let ethereum = null;
let walletConnect = null;
let currentAccount = null;
let currentChainId = null;
let currentConnector = null;

/**
 * Initialize wallet connections
 */
export async function initWallet() {
    try {
        // Try MetaMask first
        if (typeof window.ethereum !== 'undefined') {
            ethereum = window.ethereum;
            currentConnector = 'MetaMask';
            
            // Setup event listeners
            ethereum.on('accountsChanged', handleAccountsChanged);
            ethereum.on('chainChanged', handleChainChanged);
            ethereum.on('disconnect', handleDisconnect);
            
            // Get initial state
            try {
                const accounts = await ethereum.request({ method: 'eth_accounts' });
                if (accounts.length > 0) {
                    currentAccount = accounts[0];
                }
                
                currentChainId = await ethereum.request({ method: 'eth_chainId' });
            } catch (err) {
                console.warn('Could not get initial MetaMask state:', err);
            }
            
            return {
                success: true,
                error: null,
                data: null
            };
        } else {
            // Try WalletConnect
            try {
                // Check if WalletConnect is available
                if (typeof window.WalletConnect !== 'undefined') {
                    currentConnector = 'WalletConnect';
                    return {
                        success: true,
                        error: null,
                        data: null
                    };
                }
            } catch (err) {
                console.warn('WalletConnect not available:', err);
            }
            
            return {
                success: false,
                error: 'No wallet provider found. Please install MetaMask or use WalletConnect.',
                data: null
            };
        }
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Get current wallet account info
 */
export async function getWalletAccount() {
    try {
        const account = {
            address: currentAccount,
            is_connected: !!currentAccount,
            is_connecting: false,
            is_disconnected: !currentAccount,
            chain_id: currentChainId ? parseInt(currentChainId, 16) : null,
            connector: currentConnector || 'Unknown'
        };
        
        return {
            success: true,
            error: null,
            data: JSON.stringify(account)
        };
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Connect wallet (request accounts)
 */
export async function connectWallet() {
    try {
        // Try MetaMask first
        if (ethereum && currentConnector === 'MetaMask') {
            const accounts = await ethereum.request({ 
                method: 'eth_requestAccounts' 
            });
            
            if (accounts.length > 0) {
                currentAccount = accounts[0];
                currentChainId = await ethereum.request({ method: 'eth_chainId' });
                
                return {
                    success: true,
                    error: null,
                    data: JSON.stringify({
                        address: currentAccount,
                        chain_id: parseInt(currentChainId, 16),
                        connector: 'MetaMask'
                    })
                };
            } else {
                throw new Error('No accounts found');
            }
        } else if (currentConnector === 'WalletConnect') {
            // WalletConnect implementation would go here
            // For now, show a message to use MetaMask
            return {
                success: false,
                error: 'WalletConnect integration coming soon. Please use MetaMask for now.',
                data: null
            };
        } else {
            // Try to detect and connect to any available wallet
            if (typeof window.ethereum !== 'undefined') {
                ethereum = window.ethereum;
                currentConnector = 'MetaMask';
                
                // Setup event listeners
                ethereum.on('accountsChanged', handleAccountsChanged);
                ethereum.on('chainChanged', handleChainChanged);
                ethereum.on('disconnect', handleDisconnect);
                
                const accounts = await ethereum.request({ 
                    method: 'eth_requestAccounts' 
                });
                
                if (accounts.length > 0) {
                    currentAccount = accounts[0];
                    currentChainId = await ethereum.request({ method: 'eth_chainId' });
                    
                    return {
                        success: true,
                        error: null,
                        data: JSON.stringify({
                            address: currentAccount,
                            chain_id: parseInt(currentChainId, 16),
                            connector: 'MetaMask'
                        })
                    };
                } else {
                    throw new Error('No accounts found');
                }
            } else {
                throw new Error('No wallet provider found');
            }
        }
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Disconnect wallet
 */
export async function disconnectWallet() {
    try {
        if (currentConnector === 'MetaMask' && ethereum) {
            // MetaMask doesn't have a disconnect method, just clear local state
            currentAccount = null;
            currentChainId = null;
        } else if (currentConnector === 'WalletConnect' && walletConnect) {
            // WalletConnect disconnect would go here
            await walletConnect.killSession();
            currentAccount = null;
            currentChainId = null;
        } else {
            // Clear state for any other connector
            currentAccount = null;
            currentChainId = null;
        }
        
        return {
            success: true,
            error: null,
            data: null
        };
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Switch to specific chain
 */
export async function switchToChain(chainIdHex) {
    try {
        if (!ethereum) {
            throw new Error('MetaMask not initialized');
        }
        
        await ethereum.request({
            method: 'wallet_switchEthereumChain',
            params: [{ chainId: chainIdHex }],
        });
        
        return {
            success: true,
            error: null,
            data: null
        };
    } catch (error) {
        // Chain not added, try to add it
        if (error.code === 4902) {
            return {
                success: false,
                error: 'Chain not added to MetaMask',
                data: null
            };
        }
        
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Sign message with wallet
 */
export async function signWalletMessage(message) {
    try {
        if (!ethereum || !currentAccount) {
            throw new Error('Wallet not connected');
        }
        
        const signature = await ethereum.request({
            method: 'personal_sign',
            params: [message, currentAccount],
        });
        
        return {
            success: true,
            error: null,
            data: signature
        };
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Sign typed data (EIP-712)
 */
export async function signTypedData(typedData) {
    try {
        if (!currentAccount) {
            throw new Error('Wallet not connected');
        }

        if (currentConnector === 'MetaMask' && ethereum) {
            const signature = await ethereum.request({
                method: 'eth_signTypedData_v4',
                params: [currentAccount, typedData],
            });
            
            return {
                success: true,
                error: null,
                data: signature
            };
        } else if (currentConnector === 'WalletConnect' && walletConnect) {
            // WalletConnect signing would go here
            throw new Error('WalletConnect signing not implemented yet');
        } else {
            throw new Error('Unsupported wallet connector');
        }
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

// Event handlers
function handleAccountsChanged(accounts) {
    if (accounts.length === 0) {
        currentAccount = null;
    } else {
        currentAccount = accounts[0];
    }
    console.log('Account changed:', currentAccount, 'via', currentConnector);
}

function handleChainChanged(chainId) {
    currentChainId = chainId;
    console.log('Chain changed:', chainId, 'via', currentConnector);
    // Reload page on chain change (recommended by MetaMask)
    window.location.reload();
}

function handleDisconnect() {
    currentAccount = null;
    currentChainId = null;
    console.log('Disconnected from', currentConnector);
}

/**
 * Cleanup event listeners
 */
export async function cleanupWallet() {
    try {
        if (currentConnector === 'MetaMask' && ethereum) {
            ethereum.removeListener('accountsChanged', handleAccountsChanged);
            ethereum.removeListener('chainChanged', handleChainChanged);
            ethereum.removeListener('disconnect', handleDisconnect);
        } else if (currentConnector === 'WalletConnect' && walletConnect) {
            // WalletConnect cleanup would go here
            await walletConnect.killSession();
        }
        
        // Reset state
        currentAccount = null;
        currentChainId = null;
        currentConnector = null;
        
        return {
            success: true,
            error: null,
            data: null
        };
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}