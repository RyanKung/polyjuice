use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::models::EndpointData;
use polyendpoint_sdk::PolyEndpointClient;

#[wasm_bindgen(module = "/js/wallet.js")]
extern "C" {
    #[wasm_bindgen(js_name = initWallet)]
    pub async fn init_wallet() -> JsValue;

    #[wasm_bindgen(js_name = getWalletAccount)]
    pub async fn get_wallet_account() -> JsValue;

    #[wasm_bindgen(js_name = connectWallet)]
    pub async fn connect_wallet() -> JsValue;

    #[wasm_bindgen(js_name = disconnectWallet)]
    pub async fn disconnect_wallet() -> JsValue;

    #[wasm_bindgen(js_name = switchToChain)]
    pub async fn switch_to_chain(chain_id_hex: &str) -> JsValue;

    #[wasm_bindgen(js_name = signWalletMessage)]
    pub async fn sign_wallet_message(message: &str) -> JsValue;

    #[wasm_bindgen(js_name = signTypedData)]
    pub async fn sign_typed_data(typed_data: &str) -> JsValue;

    #[wasm_bindgen(js_name = cleanupWallet)]
    pub async fn cleanup_wallet() -> JsValue;

    #[wasm_bindgen(js_name = getPolyEndpoints)]
    pub async fn get_poly_endpoints(contract_address: &str, rpc_url: &str) -> JsValue;

    #[wasm_bindgen(js_name = callContract)]
    pub async fn call_contract(contract_address: &str, call_data: &str, rpc_url: &str) -> JsValue;

    #[wasm_bindgen(js_name = pingEndpoint)]
    pub async fn ping_endpoint(url: &str) -> JsValue;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletResponse {
    pub success: bool,
    pub error: Option<String>,
    pub data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletAccount {
    pub address: Option<String>,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub is_disconnected: bool,
    pub chain_id: Option<u64>,
    pub connector: Option<String>,
}

// Initialize wallet system
pub async fn initialize() -> Result<(), String> {
    let result = init_wallet().await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        Ok(())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// Get current wallet account info
pub async fn get_account() -> Result<WalletAccount, String> {
    let result = get_wallet_account().await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        if let Some(data) = response.data {
            let account: WalletAccount = serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse account data: {}", e))?;
            Ok(account)
        } else {
            Err("No account data returned".to_string())
        }
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// Connect wallet
pub async fn connect() -> Result<(), String> {
    let result = connect_wallet().await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        Ok(())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// Disconnect wallet
pub async fn disconnect() -> Result<(), String> {
    let result = disconnect_wallet().await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        Ok(())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}


// Sign typed data (EIP-712)
pub async fn sign_eip712(typed_data: &str) -> Result<String, String> {
    let result = sign_typed_data(typed_data).await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        Ok(response.data.unwrap_or_default())
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

// Get PolyEndpoint contract data
pub async fn get_endpoints(contract_address: &str, rpc_url: &str) -> Result<EndpointData, String> {
    web_sys::console::log_1(&format!("📋 Fetching endpoints from contract: {}", contract_address).into());
    
    // Create SDK client
    let client = PolyEndpointClient::new(contract_address);
    
    // Use network name from RPC URL or default to base-sepolia
    let network = if rpc_url.contains("sepolia") {
        "base-sepolia"
    } else if rpc_url.contains("mainnet") {
        "base-mainnet"
    } else {
        // Fallback to using RPC URL directly
        rpc_url
    };
    
    // Fetch endpoints using SDK
    match client.get_endpoints(network).await {
        Ok(endpoints) => {
            let urls: Vec<String> = endpoints.iter().map(|e| e.url.clone()).collect();
            web_sys::console::log_1(&format!("✅ Fetched {} endpoints from SDK", urls.len()).into());
            
            Ok(EndpointData {
                endpoints: urls,
                contract_address: contract_address.to_string(),
                network: network.to_string(),
            })
        }
        Err(e) => {
            web_sys::console::log_1(&format!("❌ SDK error: {}", e).into());
            // Fallback to empty array
            Ok(EndpointData {
                endpoints: vec![],
                contract_address: contract_address.to_string(),
                network: network.to_string(),
            })
        }
    }
}

/// Ping an endpoint and measure latency
pub async fn ping_endpoint_service(url: &str) -> Result<f64, String> {
    let result = ping_endpoint(url).await;
    let response: WalletResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if response.success {
        if let Some(data) = response.data {
            #[derive(Deserialize)]
            struct PingResponse {
                latency: f64,
            }
            let ping_data: PingResponse = serde_json::from_str(&data)
                .map_err(|e| format!("Failed to parse ping data: {}", e))?;
            Ok(ping_data.latency)
        } else {
            Err("No ping data returned".to_string())
        }
    } else {
        Err(response
            .error
            .unwrap_or_else(|| "Unknown error".to_string()))
    }
}

