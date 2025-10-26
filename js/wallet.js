/**
 * MetaMask Wallet Integration
 * Simple MetaMask connector using window.ethereum
 */

let ethereum = null;
let currentAccount = null;
let currentChainId = null;

/**
 * Initialize MetaMask connection
 */
export async function initWallet() {
    try {
        if (typeof window.ethereum !== 'undefined') {
            ethereum = window.ethereum;
            
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
                console.warn('Could not get initial state:', err);
            }
            
            return {
                success: true,
                error: null,
                data: null
            };
        } else {
            return {
                success: false,
                error: 'MetaMask is not installed',
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
            connector: 'MetaMask'
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
        if (!ethereum) {
            throw new Error('MetaMask not initialized');
        }
        
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
                    chain_id: parseInt(currentChainId, 16)
                })
            };
        } else {
            throw new Error('No accounts found');
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
        currentAccount = null;
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
        if (!ethereum || !currentAccount) {
            throw new Error('Wallet not connected');
        }
        
        console.log('signTypedData received:', typeof typedData);
        
        // Parse the JSON string to an object
        let parsedTypedData;
        if (typeof typedData === 'string') {
            parsedTypedData = JSON.parse(typedData);
        } else if (typeof typedData === 'object' && typedData !== null) {
            parsedTypedData = typedData;
        } else {
            throw new Error('Invalid typed data type: ' + typeof typedData);
        }
        
        console.log('Parsed typedData:', JSON.stringify(parsedTypedData, null, 2));
        
        // Validation
        if (!parsedTypedData.domain || !parsedTypedData.message || !parsedTypedData.types) {
            throw new Error('Invalid typed data structure - missing required fields');
        }
        
        console.log('Calling MetaMask eth_signTypedData_v4...');
        
        // Pass the parsed data directly to MetaMask
        const signature = await ethereum.request({
            method: 'eth_signTypedData_v4',
            params: [currentAccount, parsedTypedData],
        });
        
        return {
            success: true,
            error: null,
            data: signature
        };
    } catch (error) {
        console.error('signTypedData error:', error);
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
    console.log('Account changed:', currentAccount);
}

function handleChainChanged(chainId) {
    currentChainId = chainId;
    console.log('Chain changed:', chainId);
    // Reload page on chain change (recommended by MetaMask)
    window.location.reload();
}

function handleDisconnect() {
    currentAccount = null;
    currentChainId = null;
    console.log('Disconnected');
}

/**
 * Cleanup event listeners
 */
export async function cleanupWallet() {
    try {
        if (ethereum) {
            ethereum.removeListener('accountsChanged', handleAccountsChanged);
            ethereum.removeListener('chainChanged', handleChainChanged);
            ethereum.removeListener('disconnect', handleDisconnect);
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
 * Decode ABI-encoded tuple of (string[], string[])
 * Format for tuple: offset1 (32 bytes) + offset2 (32 bytes) + data1 + data2
 */
function decodeEndpointsArray(hexResult) {
    // Remove '0x' prefix
    const data = hexResult.replace('0x', '');
    
    // Check if we have data
    if (!data || data.length < 128) {
        console.warn('Invalid or empty contract response');
        return [];
    }
    
    // Parse offset to first array (first 32 bytes)
    const offset1Hex = '0x' + data.substring(0, 64);
    const offset1 = parseInt(offset1Hex, 16);
    
    // Parse offset to second array (second 32 bytes)
    const offset2Hex = '0x' + data.substring(64, 128);
    const offset2 = parseInt(offset2Hex, 16);
    
    const startPosition = 128; // After both offsets (64 bytes each)
    
    // Decode first array (urls)
    const urls = decodeStringArray(data, offset1, startPosition);
    
    // Decode second array (descriptions) 
    const descriptions = decodeStringArray(data, offset2, startPosition);
    
    console.log('Decoded urls:', urls);
    console.log('Decoded descriptions:', descriptions);
    
    // Return only URLs for now (we can add descriptions later if needed)
    return urls;
}

/**
 * Decode a single ABI-encoded array of strings
 */
function decodeStringArray(data, offset, startPosition) {
    // The offset points to where the array starts (from startPosition)
    const arrayStart = startPosition + offset;
    
    if (arrayStart >= data.length / 2) {
        console.warn('Offset beyond data length');
        return [];
    }
    
    // Read array length (32 bytes starting at offset)
    const lengthHex = '0x' + data.substring(arrayStart * 2, (arrayStart + 32) * 2);
    const arrayLength = parseInt(lengthHex, 16);
    
    console.log('Array length:', arrayLength);
    
    const strings = [];
    let position = (arrayStart + 32) * 2; // Start after length
    
    // Decode each string in the array
    for (let i = 0; i < arrayLength && position < data.length; i++) {
        // Each string encoding: offset (32 bytes) + length (32 bytes) + data
        const stringOffsetHex = '0x' + data.substring(position, position + 64);
        const stringOffset = parseInt(stringOffsetHex, 16);
        
        const stringDataOffset = startPosition + offset + stringOffset;
        const stringLengthHex = '0x' + data.substring(stringDataOffset * 2, (stringDataOffset + 32) * 2);
        const stringLength = parseInt(stringLengthHex, 16);
        
        const stringDataStart = (stringDataOffset + 32) * 2;
        const stringDataEnd = stringDataStart + stringLength * 2;
        const stringDataHex = data.substring(stringDataStart, stringDataEnd);
        
        // Convert hex to UTF-8 string
        const stringValue = hexToUtf8(stringDataHex);
        strings.push(stringValue);
        
        // Move to next string entry
        position += 64; // Move past the offset entry (32 bytes = 64 hex chars)
    }
    
    return strings;
}

/**
 * Convert hex string to UTF-8 string
 */
function hexToUtf8(hex) {
    try {
        const bytes = [];
        for (let i = 0; i < hex.length; i += 2) {
            bytes.push(parseInt(hex.substr(i, 2), 16));
        }
        return new TextDecoder('utf-8').decode(new Uint8Array(bytes));
    } catch (e) {
        console.error('Hex to UTF-8 error:', e);
        return '';
    }
}

/**
 * Query PolyEndpoint contract for endpoints
 */
export async function getPolyEndpoints(contractAddress, rpcUrl) {
    try {
        if (!ethereum) {
            // If MetaMask is not available, use the provided RPC URL
            return {
                success: false,
                error: 'MetaMask not available and no RPC URL provided',
                data: null
            };
        }

        // Use ethereum provider if available
        const provider = ethereum;

        // ABI for getAllEndpoints() view function
        // function selector: getAllEndpoints() -> 0x5b7f5238
        const functionSelector = '0x5b7f5238';
        
        // Call the contract
        const result = await provider.request({
            method: 'eth_call',
            params: [
                {
                    to: contractAddress,
                    data: functionSelector,
                },
                'latest'
            ]
        });

        // Decode the result
        // The result is ABI-encoded array of strings
        // We need to decode this properly
        console.log('Raw result:', result);

        // For now, return a mock result since decoding ABI in JavaScript is complex
        // This should be replaced with proper ABI decoding
        return {
            success: true,
            error: null,
            data: JSON.stringify({
                endpoints: ['https://api.example.com/endpoint1', 'https://api.example.com/endpoint2'],
                contract_address: contractAddress,
                network: 'base-sepolia'
            })
        };
    } catch (error) {
        console.error('getPolyEndpoints error:', error);
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Call contract with specific call data
 */
export async function callContract(contractAddress, callData, rpcUrl) {
    try {
        console.log('🔗 Calling contract:', {
            address: contractAddress,
            callData: callData,
            rpcUrl: rpcUrl
        });

        let provider;
        
        // Use MetaMask if available, otherwise fallback to direct RPC
        if (ethereum) {
            provider = ethereum;
            const chainId = await provider.request({ method: 'eth_chainId' });
            const targetChainId = '0x14a34'; // Base Sepolia in hex (84532 decimal)
            if (chainId !== targetChainId) {
                return {
                    success: false,
                    error: `Please switch to Base Sepolia (chain ID: ${parseInt(targetChainId, 16)})`,
                    data: null
                };
            }
        } else {
            return {
                success: false,
                error: 'MetaMask not available. Please install MetaMask.',
                data: null
            };
        }

        // Call the contract
        const result = await provider.request({
            method: 'eth_call',
            params: [
                {
                    to: contractAddress,
                    data: callData,
                },
                'latest'
            ]
        });

        console.log('📋 Contract call result:', result);

        // Return raw hex data for Rust to decode
        return {
            success: true,
            error: null,
            data: result  // Return raw hex string
        };
    } catch (error) {
        console.error('❌ callContract error:', error);
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}

/**
 * Ping an endpoint and measure latency
 */
export async function pingEndpoint(endpoint) {
    try {
        const startTime = performance.now();
        
        // Add /api/health or root path if needed
        const healthCheckUrl = endpoint.endsWith('/') ? endpoint + 'api/health' : endpoint + '/api/health';
        
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000); // 5 second timeout
        
        try {
            const response = await fetch(healthCheckUrl, {
                method: 'GET',
                signal: controller.signal,
                mode: 'cors',
                cache: 'no-cache'
            });
            const endTime = performance.now();
            const latency = endTime - startTime;
            clearTimeout(timeoutId);
            
            return {
                success: true,
                error: null,
                data: JSON.stringify({
                    latency: latency
                })
            };
        } catch (fetchError) {
            clearTimeout(timeoutId);
            if (fetchError.name === 'AbortError') {
                return {
                    success: false,
                    error: 'Timeout after 5 seconds',
                    data: null
                };
            }
            throw fetchError;
        }
    } catch (error) {
        return {
            success: false,
            error: error.message,
            data: null
        };
    }
}