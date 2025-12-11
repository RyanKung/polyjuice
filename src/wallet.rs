use js_sys::Array;
use js_sys::Function;
use js_sys::Object;
use js_sys::Promise;
use js_sys::Reflect;
use polyendpoint_sdk::PolyEndpointClient;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

use crate::models::EndpointData;

#[allow(dead_code)]
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

// Helper function to get window object
fn get_window() -> Result<web_sys::Window, String> {
    window().ok_or("No window object".to_string())
}

// Helper function to get WalletConnect provider from window
fn get_wallet_connect_provider(window: &web_sys::Window) -> Result<JsValue, String> {
    Reflect::get(window, &"__walletConnect".into())
        .map_err(|_| "Failed to get WalletConnect provider".to_string())
}

// Helper function to check if provider has accounts
fn get_provider_accounts(provider: &JsValue) -> Option<Array> {
    Reflect::get(provider, &"accounts".into())
        .ok()
        .and_then(|a| a.dyn_ref::<Array>().cloned())
}

// Helper function to check if provider is connected (has accounts)
fn is_provider_connected(provider: &JsValue) -> bool {
    if let Some(accounts) = get_provider_accounts(provider) {
        accounts.length() > 0
    } else {
        false
    }
}

// Initialize wallet system (WalletConnect v2 only)
pub async fn initialize() -> Result<(), String> {
    web_sys::console::log_1(&"🔌 Initializing WalletConnect v2...".into());
    // WalletConnect v2 will be initialized on first connect
    Ok(())
}

// Get current wallet account info
pub async fn get_account() -> Result<WalletAccount, String> {
    let window = get_window()?;

    // Check WalletConnect v2 provider
    let provider = get_wallet_connect_provider(&window).unwrap_or(JsValue::NULL);

    if !provider.is_null() && !provider.is_undefined() {
        // WalletConnect v2 uses accounts property directly
        let accounts = get_provider_accounts(&provider);

        // Get chainId from provider
        let chain_id = Reflect::get(&provider, &"chainId".into())
            .ok()
            .and_then(|c| {
                // ChainId might be a number or a string like "eip155:84532"
                if let Some(chain_str) = c.as_string() {
                    // Parse "eip155:84532" format
                    if let Some(colon_pos) = chain_str.find(':') {
                        chain_str[colon_pos + 1..].parse::<u64>().ok()
                    } else {
                        None
                    }
                } else {
                    c.as_f64().map(|n| n as u64)
                }
            });

        if let Some(accounts_array) = accounts {
            if accounts_array.length() > 0 {
                let account = accounts_array.get(0).as_string();
                return Ok(WalletAccount {
                    address: account,
                    is_connected: true,
                    is_connecting: false,
                    is_disconnected: false,
                    chain_id,
                    connector: Some("WalletConnect".to_string()),
                });
            }
        }
    }

    // No wallet connected
    Ok(WalletAccount {
        address: None,
        is_connected: false,
        is_connecting: false,
        is_disconnected: true,
        chain_id: None,
        connector: None,
    })
}

// Connect wallet using WalletConnect
pub async fn connect() -> Result<(), String> {
    web_sys::console::log_1(&"🔌 Attempting to connect wallet with WalletConnect...".into());
    connect_wallet_connect(None)
        .await
        .map_err(|e| format!("WalletConnect connection failed: {}", e))?;

    web_sys::console::log_1(&"✅ WalletConnect connection initiated".into());
    Ok(())
}

// Initialize WalletConnect v2
pub async fn initialize_wallet_connect(_bridge_url: Option<String>) -> Result<(), String> {
    let window = get_window()?;

    // Get EthereumProvider from global scope (bundled by webpack)
    let ethereum_provider = Reflect::get(&window, &"EthereumProvider".into()).map_err(|_| {
        "EthereumProvider not found. Make sure walletconnect-bundle.js is loaded.".to_string()
    })?;

    if ethereum_provider.is_undefined() || ethereum_provider.is_null() {
        return Err(
            "EthereumProvider is not available. Please ensure walletconnect-bundle.js is loaded."
                .to_string(),
        );
    }

    // Check if already initialized
    let existing_provider = get_wallet_connect_provider(&window).unwrap_or(JsValue::NULL);

    if !existing_provider.is_null() && !existing_provider.is_undefined() {
        web_sys::console::log_1(&"✅ WalletConnect already initialized".into());
        return Ok(());
    }

    // Initialize EthereumProvider with configuration
    // Note: WalletConnect v2 requires a projectId from https://cloud.walletconnect.com
    // For now, we'll use a placeholder - user should replace with their own projectId
    // Get the init static method from EthereumProvider class
    let init_fn = Reflect::get(&ethereum_provider, &"init".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("EthereumProvider.init is not a function".to_string())?;

    let config = Object::new();

    // Set projectId - get from build-time environment variable or use placeholder
    // To get your projectId:
    // 1. Visit https://cloud.walletconnect.com
    // 2. Sign up or log in
    // 3. Create a new project
    // 4. Copy the projectId from the dashboard
    // 5. Add WALLETCONNECT_PROJECT_ID=your_project_id to your .env file
    let project_id = option_env!("WALLETCONNECT_PROJECT_ID").unwrap_or("YOUR_PROJECT_ID");

    if project_id == "YOUR_PROJECT_ID" {
        web_sys::console::warn_1(&"⚠️ Using placeholder projectId. Please set WALLETCONNECT_PROJECT_ID in .env file. Get your projectId from https://cloud.walletconnect.com".into());
    }

    Reflect::set(&config, &"projectId".into(), &project_id.into())
        .map_err(|_| "Failed to set projectId".to_string())?;

    // Set optionalChains - support Base Sepolia (chainId: 84532) and Base Mainnet (chainId: 8453)
    // Using optionalChains instead of chains is recommended for multi-chain dapps
    // This ensures compatibility with Smart Contract Wallets that may only support one chain
    let optional_chains = Array::new();
    optional_chains.push(&84532.into()); // Base Sepolia
    optional_chains.push(&8453.into()); // Base Mainnet
    Reflect::set(&config, &"optionalChains".into(), &optional_chains.into())
        .map_err(|_| "Failed to set optionalChains".to_string())?;

    // Set metadata
    // Important: The URL must match your domain and subdomain for Verify API
    // Use origin (protocol + hostname + port) instead of full href to match domain verification
    let metadata = Object::new();
    Reflect::set(&metadata, &"name".into(), &"Polyjuice".into()).ok();
    Reflect::set(
        &metadata,
        &"description".into(),
        &"Discover & Chat with Farcaster Users".into(),
    )
    .ok();

    // Use origin instead of href to match domain verification requirements
    // origin format: "http://127.0.0.1:8080" or "https://yourdomain.com"
    let url = window
        .location()
        .origin()
        .map_err(|_| "Failed to get window location origin".to_string())?;
    Reflect::set(&metadata, &"url".into(), &url.clone().into())
        .map_err(|_| "Failed to set metadata url".to_string())?;

    // Set icons - use logo.png if available, otherwise empty array
    let icons = Array::new();
    // Try to use logo.png from the root, fallback to empty array
    let logo_url = format!("{}/logo.png", url);
    icons.push(&logo_url.into());
    Reflect::set(&metadata, &"icons".into(), &icons.into())
        .map_err(|_| "Failed to set metadata icons".to_string())?;

    Reflect::set(&config, &"metadata".into(), &metadata.into())
        .map_err(|_| "Failed to set metadata".to_string())?;

    // Note: showQrModal is deprecated. The modal is now handled by AppKit or display_uri event.
    // For backward compatibility, we still set it, but it may not be used in newer versions.
    Reflect::set(&config, &"showQrModal".into(), &JsValue::TRUE)
        .map_err(|_| "Failed to set showQrModal".to_string())?;

    // Call init() as a static method - it returns a Promise
    // For static methods, we call with the class itself as 'this'
    let init_promise = init_fn
        .call1(&ethereum_provider, &config.into())
        .map_err(|e| format!("Failed to call EthereumProvider.init: {:?}", e))?;

    let promise = Promise::from(init_promise);
    let provider_instance = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to initialize EthereumProvider: {:?}", e))?;

    // Store provider instance in window for later use
    Reflect::set(&window, &"__walletConnect".into(), &provider_instance)
        .map_err(|_| "Failed to store WalletConnect instance".to_string())?;

    // Setup event listeners
    setup_wallet_connect_events(&provider_instance)?;

    web_sys::console::log_1(&"✅ WalletConnect v2 initialized".into());

    Ok(())
}

// Setup WalletConnect v2 event listeners
fn setup_wallet_connect_events(provider: &JsValue) -> Result<(), String> {
    let on_fn = Reflect::get(provider, &"on".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned());

    if let Some(on) = on_fn {
        // Connect event - WalletConnect v2 emits 'connect' with { accounts, chainId }
        let connect_handler = Closure::wrap(Box::new(|connect_info: JsValue| {
            web_sys::console::log_1(&"✅ WalletConnect v2 connected event received".into());
            if let Some(info) = connect_info.dyn_ref::<Object>() {
                // Store accounts
                if let Ok(accounts) = Reflect::get(info, &"accounts".into()) {
                    if let Some(accounts_arr) = accounts.dyn_ref::<Array>() {
                        if accounts_arr.length() > 0 {
                            let account = accounts_arr.get(0);
                            let window = window().unwrap();
                            Reflect::set(&window, &"__wcAccount".into(), &account).ok();
                            web_sys::console::log_1(
                                &format!("✅ WalletConnect account stored: {:?}", account).into(),
                            );
                        }
                    }
                }
                // Store chainId
                if let Ok(chain_id) = Reflect::get(info, &"chainId".into()) {
                    let window = window().unwrap();
                    Reflect::set(&window, &"__wcChainId".into(), &chain_id).ok();
                }
            }
        }) as Box<dyn Fn(JsValue)>);

        let connect_str = "connect".into();
        on.call2(
            provider,
            &connect_str,
            connect_handler.as_ref().unchecked_ref(),
        )
        .map_err(|_| "Failed to setup connect event".to_string())?;
        connect_handler.forget();

        // Disconnect event - WalletConnect v2 emits 'disconnect' with error info
        let disconnect_handler = Closure::wrap(Box::new(|_error: JsValue| {
            web_sys::console::log_1(&"WalletConnect v2 disconnected".into());
            let window = window().unwrap();
            Reflect::set(&window, &"__wcAccount".into(), &JsValue::NULL).ok();
            Reflect::set(&window, &"__wcChainId".into(), &JsValue::NULL).ok();
        }) as Box<dyn Fn(JsValue)>);

        let disconnect_str = "disconnect".into();
        on.call2(
            provider,
            &disconnect_str,
            disconnect_handler.as_ref().unchecked_ref(),
        )
        .map_err(|_| "Failed to setup disconnect event".to_string())?;
        disconnect_handler.forget();
    }

    Ok(())
}

// Connect wallet using WalletConnect v2
pub async fn connect_wallet_connect(_bridge_url: Option<String>) -> Result<(), String> {
    let window = get_window()?;

    // Initialize if not already initialized
    let provider_js = get_wallet_connect_provider(&window).unwrap_or(JsValue::NULL);

    let provider = if provider_js.is_null() || provider_js.is_undefined() {
        web_sys::console::log_1(&"🔧 Initializing WalletConnect v2...".into());
        initialize_wallet_connect(None).await?;
        get_wallet_connect_provider(&window)?
    } else {
        provider_js
    };

    // Check if already connected by checking accounts
    if is_provider_connected(&provider) {
        web_sys::console::log_1(&"✅ WalletConnect v2 already connected".into());
        return Ok(());
    }

    // Connect using WalletConnect v2 API
    // The modal will open automatically if showQrModal is true in init config
    web_sys::console::log_1(&"📱 Connecting with WalletConnect v2...".into());
    let connect_fn = Reflect::get(&provider, &"connect".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("connect method not found on provider".to_string())?;

    // connect() returns a Promise that resolves when user connects
    let connect_promise = connect_fn
        .call0(&provider)
        .map_err(|e| format!("Failed to call connect: {:?}", e))?;

    let promise = Promise::from(connect_promise);
    let _result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to connect with WalletConnect v2: {:?}", e))?;

    web_sys::console::log_1(&"✅ WalletConnect v2 connection initiated".into());

    Ok(())
}

// Disconnect wallet
pub async fn disconnect() -> Result<(), String> {
    let window = get_window()?;

    // Disconnect WalletConnect v2 provider if connected
    if let Ok(provider) = get_wallet_connect_provider(&window) {
        if !provider.is_null() && !provider.is_undefined() {
            // Check if connected by checking accounts
            if is_provider_connected(&provider) {
                let disconnect_fn = Reflect::get(&provider, &"disconnect".into())
                    .ok()
                    .and_then(|f| f.dyn_ref::<Function>().cloned())
                    .ok_or("disconnect method not found".to_string())?;

                let disconnect_promise = disconnect_fn
                    .call0(&provider)
                    .map_err(|e| format!("Failed to call disconnect: {:?}", e))?;

                let promise = Promise::from(disconnect_promise);
                JsFuture::from(promise)
                    .await
                    .map_err(|e| format!("Failed to disconnect WalletConnect v2: {:?}", e))?;

                web_sys::console::log_1(&"✅ WalletConnect v2 disconnected".into());
            }
        }
    }

    Ok(())
}

// Sign typed data (EIP-712) using WalletConnect v2
pub async fn sign_eip712(typed_data: &str) -> Result<String, String> {
    let account = get_account().await?;

    if account.address.is_none() {
        return Err("Wallet not connected".to_string());
    }

    let address = account.address.as_ref().unwrap();
    let window = get_window()?;
    let provider = get_wallet_connect_provider(&window)
        .map_err(|_| "WalletConnect v2 not initialized".to_string())?;

    // WalletConnect v2 EthereumProvider uses request method for signing
    // Format: provider.request({ method: 'eth_signTypedData_v4', params: [address, typedData] })
    let request_fn = Reflect::get(&provider, &"request".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("request method not found on provider".to_string())?;

    let request_params = Object::new();
    Reflect::set(
        &request_params,
        &"method".into(),
        &"eth_signTypedData_v4".into(),
    )
    .map_err(|_| "Failed to set method".to_string())?;

    let params_array = Array::new();
    params_array.push(&address.into());
    params_array.push(&typed_data.into());
    Reflect::set(&request_params, &"params".into(), &params_array.into())
        .map_err(|_| "Failed to set params".to_string())?;

    let sign_promise = request_fn
        .call1(&provider, &request_params.into())
        .map_err(|e| format!("Failed to call request: {:?}", e))?;

    let promise = Promise::from(sign_promise);
    let signature = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Signing failed: {:?}", e))?;

    signature
        .as_string()
        .ok_or("Signature is not a string".to_string())
}

// Get PolyEndpoint contract data
pub async fn get_endpoints(contract_address: &str, rpc_url: &str) -> Result<EndpointData, String> {
    web_sys::console::log_1(
        &format!("📋 Fetching endpoints from contract: {}", contract_address).into(),
    );

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
            web_sys::console::log_1(
                &format!("✅ Fetched {} endpoints from SDK", urls.len()).into(),
            );

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
    let window = get_window()?;
    let performance = Reflect::get(&window, &"performance".into())
        .map_err(|_| "Performance API not available".to_string())?;

    let now = Reflect::get(&performance, &"now".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("performance.now not available".to_string())?;

    let start_time = now
        .call0(&performance)
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let fetch_fn = Reflect::get(&window, &"fetch".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("fetch not available".to_string())?;

    let health_url = if url.ends_with('/') {
        format!("{}api/health", url)
    } else {
        format!("{}/api/health", url)
    };

    let fetch_promise = fetch_fn
        .call1(&window, &health_url.into())
        .map_err(|e| format!("Failed to call fetch: {:?}", e))?;

    let promise = Promise::from(fetch_promise);
    let _response = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let end_time = now
        .call0(&performance)
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    Ok(end_time - start_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_response_serialization() {
        // Test successful response
        let response = WalletResponse {
            success: true,
            error: None,
            data: Some("test_data".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: WalletResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.data, deserialized.data);
        assert_eq!(response.error, deserialized.error);

        // Test error response
        let error_response = WalletResponse {
            success: false,
            error: Some("Test error".to_string()),
            data: None,
        };
        let json = serde_json::to_string(&error_response).unwrap();
        let deserialized: WalletResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(error_response.success, deserialized.success);
        assert_eq!(error_response.error, deserialized.error);
    }

    #[test]
    fn test_wallet_account_serialization() {
        let account = WalletAccount {
            address: Some("0x1234567890abcdef1234567890abcdef12345678".to_string()),
            is_connected: true,
            is_connecting: false,
            is_disconnected: false,
            chain_id: Some(84532),
            connector: Some("WalletConnect".to_string()),
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: WalletAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(account, deserialized);
    }

    #[test]
    fn test_wallet_account_with_walletconnect_v2() {
        let account = WalletAccount {
            address: Some("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd".to_string()),
            is_connected: true,
            is_connecting: false,
            is_disconnected: false,
            chain_id: Some(84532), // Base Sepolia
            connector: Some("WalletConnect".to_string()),
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: WalletAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(account.connector, deserialized.connector);
        assert_eq!(account.address, deserialized.address);
        assert_eq!(account.chain_id, deserialized.chain_id);
    }

    #[test]
    fn test_wallet_account_disconnected() {
        let account = WalletAccount {
            address: None,
            is_connected: false,
            is_connecting: false,
            is_disconnected: true,
            chain_id: None,
            connector: None,
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: WalletAccount = serde_json::from_str(&json).unwrap();
        assert_eq!(account, deserialized);
    }

    #[test]
    fn test_wallet_response_parsing() {
        // Test parsing a JSON string that would come from JavaScript
        let json_str =
            r#"{"success":true,"error":null,"data":"{\"address\":\"0x123\",\"chain_id\":84532}"}"#;
        let response: WalletResponse = serde_json::from_str(json_str).unwrap();
        assert!(response.success);
        assert!(response.error.is_none());
        assert!(response.data.is_some());

        // Test parsing error response
        let error_json = r#"{"success":false,"error":"Connection failed","data":null}"#;
        let error_response: WalletResponse = serde_json::from_str(error_json).unwrap();
        assert!(!error_response.success);
        assert_eq!(error_response.error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_wallet_account_from_response_data() {
        // Simulate the data that would come from get_wallet_account (WalletConnect v2)
        let account_data = r#"{"address":"0x1234567890abcdef1234567890abcdef12345678","is_connected":true,"is_connecting":false,"is_disconnected":false,"chain_id":84532,"connector":"WalletConnect"}"#;
        let account: WalletAccount = serde_json::from_str(account_data).unwrap();
        assert_eq!(
            account.address,
            Some("0x1234567890abcdef1234567890abcdef12345678".to_string())
        );
        assert_eq!(account.chain_id, Some(84532)); // Base Sepolia
        assert_eq!(account.connector, Some("WalletConnect".to_string()));
        assert!(account.is_connected);
        assert!(!account.is_disconnected);
    }
}
