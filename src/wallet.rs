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
use web_sys::Event;
use web_sys::MessageEvent;
use web_sys::Window;

use crate::models::EndpointData;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WalletAccount {
    pub address: Option<String>,
    pub is_connected: bool,
    pub is_connecting: bool,
    pub is_disconnected: bool,
    pub chain_id: Option<u64>,
    pub connector: Option<String>,
    pub fid: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredWallet {
    pub info: WalletInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WalletInfo {
    pub uuid: String,
    pub name: String,
    pub icon: String,
    pub rdns: Option<String>,
}

// Helper function to get window object
fn get_window() -> Result<Window, String> {
    window().ok_or("No window object".to_string())
}

// Get current wallet provider from window.__walletProvider
fn get_current_provider(window: &Window) -> Option<JsValue> {
    Reflect::get(window, &"__walletProvider".into())
        .ok()
        .and_then(|p| {
            if p.is_null() || p.is_undefined() {
                None
            } else {
                Some(p)
            }
        })
}

// Set current wallet provider in window.__walletProvider
fn set_current_provider(window: &Window, provider: &JsValue) -> Result<(), String> {
    Reflect::set(window, &"__walletProvider".into(), provider)
        .map_err(|_| "Failed to set wallet provider".to_string())?;
    Ok(())
}

// Save wallet connection info to localStorage
pub fn save_wallet_to_storage(wallet_uuid: &str, address: &str) -> Result<(), String> {
    let window = get_window()?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage".to_string())?
        .ok_or("localStorage not available".to_string())?;

    let data = serde_json::json!({
        "uuid": wallet_uuid,
        "address": address,
        "timestamp": js_sys::Date::new_0().get_time() as u64,
    });

    storage
        .set_item("polyjuice_wallet", &data.to_string())
        .map_err(|_| "Failed to save wallet to localStorage".to_string())?;

    web_sys::console::log_1(&format!("💾 Saved wallet to localStorage: {}", wallet_uuid).into());
    Ok(())
}

// Load wallet connection info from localStorage
pub fn load_wallet_from_storage() -> Result<Option<(String, String)>, String> {
    let window = get_window()?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage".to_string())?
        .ok_or("localStorage not available".to_string())?;

    if let Ok(Some(data_str)) = storage.get_item("polyjuice_wallet") {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&data_str) {
            if let (Some(uuid), Some(address)) = (data.get("uuid"), data.get("address")) {
                if let (Some(uuid_str), Some(addr_str)) = (uuid.as_str(), address.as_str()) {
                    web_sys::console::log_1(
                        &format!("📂 Loaded wallet from localStorage: {}", uuid_str).into(),
                    );
                    return Ok(Some((uuid_str.to_string(), addr_str.to_string())));
                }
            }
        }
    }

    Ok(None)
}

// Clear wallet connection info from localStorage
pub fn clear_wallet_from_storage() -> Result<(), String> {
    let window = get_window()?;
    let storage = window
        .local_storage()
        .map_err(|_| "Failed to get localStorage".to_string())?
        .ok_or("localStorage not available".to_string())?;

    storage
        .remove_item("polyjuice_wallet")
        .map_err(|_| "Failed to clear wallet from localStorage".to_string())?;

    web_sys::console::log_1(&"🗑️ Cleared wallet from localStorage".into());
    Ok(())
}


// Initialize wallet system - discover wallets via EIP-6963
pub async fn initialize() -> Result<(), String> {
    web_sys::console::log_1(&"🔌 Initializing EIP-6963 wallet discovery...".into());

    let window = get_window()?;

    // Store discovered wallets in window.__discoveredWallets
    let wallets_array = Array::new();
    Reflect::set(
        &window,
        &"__discoveredWallets".into(),
        &wallets_array.into(),
    )
    .map_err(|_| "Failed to initialize discovered wallets array".to_string())?;

    // Set up EIP-6963 event listener
    setup_eip6963_listener(&window)?;

    // Request wallet announcement (EIP-6963)
    request_eip6963_announcement(&window)?;

    web_sys::console::log_1(&"✅ Wallet discovery initialized".into());
    Ok(())
}

// Set up EIP-6963 event listener to discover wallets
fn setup_eip6963_listener(window: &Window) -> Result<(), String> {
    let window_clone = window.clone();
    let listener = Closure::wrap(Box::new(move |event: Event| {
        if let Some(custom_event) = event.dyn_ref::<MessageEvent>() {
            if let Ok(detail) = Reflect::get(custom_event, &"detail".into()) {
                if let Some(detail_obj) = detail.dyn_ref::<Object>() {
                    if let Ok(info) = Reflect::get(detail_obj, &"info".into()) {
                        if let Ok(provider) = Reflect::get(detail_obj, &"provider".into()) {
                            let window = &window_clone;

                            // Store in discovered wallets array
                            if let Ok(wallets) =
                                Reflect::get(&window, &"__discoveredWallets".into())
                            {
                                if let Some(wallets_arr) = wallets.dyn_ref::<Array>() {
                                    let wallet_obj = Object::new();
                                    Reflect::set(&wallet_obj, &"info".into(), &info).ok();
                                    Reflect::set(&wallet_obj, &"provider".into(), &provider).ok();
                                    wallets_arr.push(&wallet_obj);

                                    web_sys::console::log_1(
                                        &format!(
                                            "✅ Discovered wallet via EIP-6963: {:?}",
                                            Reflect::get(&info, &"name".into())
                                                .ok()
                                                .and_then(|n| n.as_string())
                                        )
                                        .into(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }) as Box<dyn Fn(Event)>);

    window
        .add_event_listener_with_callback(
            "eip6963:announceProvider",
            listener.as_ref().unchecked_ref(),
        )
        .map_err(|_| "Failed to add EIP-6963 event listener".to_string())?;

    listener.forget();
    Ok(())
}

// Request wallet announcement (EIP-6963)
fn request_eip6963_announcement(window: &Window) -> Result<(), String> {
    // Dispatch request event for wallets to announce themselves
    let event = Event::new("eip6963:requestProvider")
        .map_err(|_| "Failed to create request event".to_string())?;

    window
        .dispatch_event(&event)
        .map_err(|_| "Failed to dispatch request event".to_string())?;

    Ok(())
}

// Get all discovered wallets
pub async fn discover_wallets() -> Result<Vec<DiscoveredWallet>, String> {
    let window = get_window()?;

    // Get discovered wallets array
    let wallets_js = Reflect::get(&window, &"__discoveredWallets".into())
        .ok()
        .and_then(|w| w.dyn_ref::<Array>().cloned());

    let mut wallets = Vec::new();
    // Track wallet names and UUIDs to avoid duplicates
    let mut seen_wallets = std::collections::HashSet::<String>::new();

    web_sys::console::log_1(&"🔍 Starting wallet discovery...".into());

    if let Some(wallets_arr) = wallets_js {
        web_sys::console::log_1(
            &format!("📋 Found {} wallets from EIP-6963", wallets_arr.length()).into(),
        );
        for i in 0..wallets_arr.length() {
            if let Some(wallet_obj) = wallets_arr.get(i).dyn_ref::<Object>() {
                if let Ok(info) = Reflect::get(wallet_obj, &"info".into()) {
                    if let Ok(_provider) = Reflect::get(wallet_obj, &"provider".into()) {
                        // Parse wallet info from JS object
                        if let Some(info_obj) = info.dyn_ref::<Object>() {
                            let uuid = Reflect::get(info_obj, &"uuid".into())
                                .ok()
                                .and_then(|v| v.as_string())
                                .unwrap_or_else(|| format!("wallet_{}", i));

                            let name = Reflect::get(info_obj, &"name".into())
                                .ok()
                                .and_then(|v| v.as_string())
                                .unwrap_or_else(|| "Unknown Wallet".to_string());

                            // Try to get icon from EIP-6963 info, otherwise use wallet name to get icon URL
                            let icon = Reflect::get(info_obj, &"icon".into())
                                .ok()
                                .and_then(|v| v.as_string())
                                .unwrap_or_else(|| get_wallet_icon_url(&name));

                            let rdns = Reflect::get(info_obj, &"rdns".into())
                                .ok()
                                .and_then(|v| v.as_string());

                            // Add to seen_wallets to prevent duplicates
                            if !seen_wallets.contains(&name) {
                                seen_wallets.insert(name.clone());
                                wallets.push(DiscoveredWallet {
                                    info: WalletInfo {
                                        uuid: uuid.clone(),
                                        name: name.clone(),
                                        icon: icon.clone(),
                                        rdns: rdns.clone(),
                                    },
                                });
                                web_sys::console::log_1(
                                    &format!("✅ Added EIP-6963 wallet: {} (uuid: {})", name, uuid)
                                        .into(),
                                );
                            } else {
                                web_sys::console::log_1(
                                    &format!("⏭️ Skipped duplicate EIP-6963 wallet: {}", name)
                                        .into(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Check window.rabby (Rabby Wallet) - priority check before window.ethereum
    // Only add if not already added via EIP-6963 or other methods
    web_sys::console::log_1(&"🔍 Checking window.rabby...".into());
    if !seen_wallets.contains("Rabby") {
        if let Ok(rabby) = Reflect::get(&window, &"rabby".into()) {
            if !rabby.is_null() && !rabby.is_undefined() {
                web_sys::console::log_1(&"✅ Found window.rabby".into());
                if let Some(_provider) = rabby.dyn_ref::<Object>() {
                    seen_wallets.insert("Rabby".to_string());
                    wallets.push(DiscoveredWallet {
                        info: WalletInfo {
                            uuid: "rabby_wallet".to_string(),
                            name: "Rabby".to_string(),
                            icon: get_wallet_icon_url("Rabby"),
                            rdns: Some("io.rabby".to_string()),
                        },
                    });
                    web_sys::console::log_1(&"✅ Added wallet from window.rabby: Rabby".into());
                }
            } else {
                web_sys::console::log_1(&"⚠️ window.rabby is null or undefined".into());
            }
        } else {
            web_sys::console::log_1(&"⚠️ Could not access window.rabby".into());
        }
    } else {
        web_sys::console::log_1(&"⏭️ Skipped window.rabby (already added via EIP-6963)".into());
    }

    // Check window.phantom (Phantom Wallet - also supports Ethereum) - priority check before window.ethereum
    // Only add if not already added via EIP-6963 or other methods
    web_sys::console::log_1(&"🔍 Checking window.phantom...".into());
    if !seen_wallets.contains("Phantom") {
        if let Ok(phantom) = Reflect::get(&window, &"phantom".into()) {
            if !phantom.is_null() && !phantom.is_undefined() {
                web_sys::console::log_1(&"✅ Found window.phantom".into());
                if let Some(provider) = phantom.dyn_ref::<Object>() {
                    // Check if it has ethereum provider
                    if let Ok(ethereum) = Reflect::get(provider, &"ethereum".into()) {
                        if !ethereum.is_null() && !ethereum.is_undefined() {
                            web_sys::console::log_1(&"✅ Found window.phantom.ethereum".into());
                            if let Some(_eth_provider) = ethereum.dyn_ref::<Object>() {
                                seen_wallets.insert("Phantom".to_string());
                                wallets.push(DiscoveredWallet {
                                    info: WalletInfo {
                                        uuid: "phantom_ethereum".to_string(),
                                        name: "Phantom".to_string(),
                                        icon: get_wallet_icon_url("Phantom"),
                                        rdns: Some("app.phantom".to_string()),
                                    },
                                });
                                web_sys::console::log_1(
                                    &"✅ Added wallet from window.phantom: Phantom".into(),
                                );
                            }
                        } else {
                            web_sys::console::log_1(
                                &"⚠️ window.phantom.ethereum is null or undefined".into(),
                            );
                        }
                    } else {
                        web_sys::console::log_1(
                            &"⚠️ Could not access window.phantom.ethereum".into(),
                        );
                    }
                }
            } else {
                web_sys::console::log_1(&"⚠️ window.phantom is null or undefined".into());
            }
        } else {
            web_sys::console::log_1(&"⚠️ Could not access window.phantom".into());
        }
    } else {
        web_sys::console::log_1(&"⏭️ Skipped window.phantom (already added via EIP-6963)".into());
    }

    // Check window.ethereum (MetaMask, Base, Rainbow, and other EIP-1193 providers)
    // This should include all wallets that register with window.ethereum
    // IMPORTANT: EIP-6963 wallets are preferred over window.ethereum to avoid Rabby hijacking
    if let Ok(ethereum) = Reflect::get(&window, &"ethereum".into()) {
        if !ethereum.is_null() && !ethereum.is_undefined() {
            web_sys::console::log_1(
                &format!(
                    "🔍 Checking window.ethereum, is_array: {}",
                    ethereum.dyn_ref::<Array>().is_some()
                )
                .into(),
            );

            // Check if MetaMask was already found via EIP-6963 (preferred, avoids Rabby hijacking)
            let metamask_found_via_eip6963 = wallets.iter().any(|w| {
                w.info.name == "MetaMask"
                    || (w.info.rdns.is_some() && w.info.rdns.as_ref().unwrap().contains("metamask"))
            });

            if metamask_found_via_eip6963 {
                web_sys::console::log_1(&"✅ MetaMask already found via EIP-6963, skipping window.ethereum to avoid Rabby hijacking".into());
            }

            // Check if it's an array (multiple providers) or single provider
            if let Some(providers) = ethereum.dyn_ref::<Array>() {
                // Multiple providers (e.g., when multiple wallets are installed)
                web_sys::console::log_1(
                    &format!(
                        "📋 Found {} providers in window.ethereum array",
                        providers.length()
                    )
                    .into(),
                );
                for i in 0..providers.length() {
                    if let Some(provider) = providers.get(i).dyn_ref::<Object>() {
                        // Log all available properties for debugging
                        web_sys::console::log_1(
                            &format!("🔍 Examining provider at ethereum[{}]", i).into(),
                        );

                        // Check common identification properties
                        let props_to_check = vec![
                            "isRainbow",
                            "isMetaMask",
                            "isRabby",
                            "isPhantom",
                            "isBase",
                            "providerName",
                            "wallet",
                        ];
                        for prop in props_to_check {
                            if let Ok(val) = Reflect::get(provider, &prop.into()) {
                                if !val.is_null() && !val.is_undefined() {
                                    if let Some(val_str) = val.as_string() {
                                        web_sys::console::log_1(
                                            &format!("  - {}: {}", prop, val_str).into(),
                                        );
                                    } else if let Some(val_bool) = val.as_bool() {
                                        web_sys::console::log_1(
                                            &format!("  - {}: {}", prop, val_bool).into(),
                                        );
                                    }
                                }
                            }
                        }

                        // Skip MetaMask from window.ethereum if we already have it via EIP-6963 (native provider)
                        if metamask_found_via_eip6963 {
                            if let Ok(is_meta) = Reflect::get(provider, &"isMetaMask".into()) {
                                if is_meta.as_bool().unwrap_or(false) {
                                    web_sys::console::log_1(&"⏭️ Skipping MetaMask from window.ethereum (preferring EIP-6963 native provider to avoid Rabby hijacking)".into());
                                    continue;
                                }
                            }
                        }

                        // If Rabby is already added, skip isRabby check to allow MetaMask to be identified
                        let skip_rabby = seen_wallets.contains("Rabby");
                        if let Some(name) = get_provider_name(provider, skip_rabby) {
                            web_sys::console::log_1(
                                &format!("✅ Identified wallet in ethereum[{}]: {}", i, name)
                                    .into(),
                            );
                            // Skip if already added via specific wallet check (e.g., window.rabby, window.phantom)
                            if !seen_wallets.contains(&name) {
                                seen_wallets.insert(name.clone());

                                // Use a consistent UUID format based on wallet name for better matching
                                let uuid = match name.as_str() {
                                    "MetaMask" => "metamask".to_string(),
                                    "Rainbow" => "rainbow".to_string(),
                                    "Rabby" => "rabby_wallet".to_string(),
                                    "Base" => "base".to_string(),
                                    "Phantom" => "phantom_ethereum".to_string(),
                                    _ => format!("ethereum_{}", i),
                                };

                                wallets.push(DiscoveredWallet {
                                    info: WalletInfo {
                                        uuid,
                                        name: name.clone(),
                                        icon: get_wallet_icon_url(&name),
                                        rdns: None,
                                    },
                                });
                                web_sys::console::log_1(
                                    &format!("✅ Added wallet: {}", name).into(),
                                );
                            } else {
                                web_sys::console::log_1(
                                    &format!("⏭️ Skipped duplicate wallet: {}", name).into(),
                                );
                            }
                        } else {
                            web_sys::console::log_1(
                                &format!("⚠️ Could not identify wallet at ethereum[{}]", i).into(),
                            );
                        }
                    }
                }
            } else if let Some(_provider) = ethereum.dyn_ref::<Object>() {
                // Single provider

                // Check if MetaMask was already found via EIP-6963 (preferred, avoids Rabby hijacking)
                let metamask_found_via_eip6963 = wallets.iter().any(|w| {
                    w.info.name == "MetaMask"
                        || (w.info.rdns.is_some()
                            && w.info.rdns.as_ref().unwrap().contains("metamask"))
                });

                if metamask_found_via_eip6963 {
                    if let Ok(is_meta) = Reflect::get(&ethereum, &"isMetaMask".into()) {
                        if is_meta.as_bool().unwrap_or(false) {
                            web_sys::console::log_1(&"⏭️ Skipping MetaMask from window.ethereum (preferring EIP-6963 native provider to avoid Rabby hijacking)".into());
                            // Continue to check other wallets, but skip MetaMask
                        }
                    }
                }

                // If Rabby is already added, skip isRabby check to allow MetaMask to be identified
                let skip_rabby = seen_wallets.contains("Rabby");
                if let Some(name) = get_provider_name(&ethereum, skip_rabby) {
                    web_sys::console::log_1(
                        &format!("🔍 Found single wallet in window.ethereum: {}", name).into(),
                    );

                    // Skip MetaMask if we already have it via EIP-6963
                    if name == "MetaMask" && metamask_found_via_eip6963 {
                        web_sys::console::log_1(&"⏭️ Skipping MetaMask from window.ethereum (preferring EIP-6963 native provider)".into());
                    } else if !seen_wallets.contains(&name) {
                        let name_clone = name.clone();
                        seen_wallets.insert(name.clone());

                        // Use a consistent UUID format based on wallet name
                        let uuid = match name.as_str() {
                            "MetaMask" => "metamask".to_string(),
                            "Rainbow" => "rainbow".to_string(),
                            "Rabby" => "rabby_wallet".to_string(),
                            "Base" => "base".to_string(),
                            "Phantom" => "phantom_ethereum".to_string(),
                            _ => "ethereum_default".to_string(),
                        };

                        wallets.push(DiscoveredWallet {
                            info: WalletInfo {
                                uuid,
                                name: name_clone.clone(),
                                icon: get_wallet_icon_url(&name_clone),
                                rdns: None,
                            },
                        });
                        web_sys::console::log_1(
                            &format!("✅ Added wallet: {} with UUID", name).into(),
                        );
                    } else {
                        web_sys::console::log_1(
                            &format!("⏭️ Skipped duplicate wallet: {}", name).into(),
                        );
                    }
                } else {
                    web_sys::console::log_1(
                        &"⚠️ Could not identify wallet in window.ethereum".into(),
                    );
                }
            }
        } else {
            web_sys::console::log_1(&"⚠️ window.ethereum is null or undefined".into());
        }
    } else {
        web_sys::console::log_1(&"⚠️ Could not access window.ethereum".into());
    }

    web_sys::console::log_1(&format!("✅ Total wallets discovered: {}", wallets.len()).into());

    // Get all supported wallets and merge with discovered ones
    let all_wallets = get_all_supported_wallets(wallets, seen_wallets);

    Ok(all_wallets)
}

// Get all supported wallets list
// Returns wallets with detected ones first, then undetected ones
fn get_all_supported_wallets(
    detected_wallets: Vec<DiscoveredWallet>,
    detected_names: std::collections::HashSet<String>,
) -> Vec<DiscoveredWallet> {
    // List of all supported wallets (in preferred order)
    let supported_wallets = vec![
        ("MetaMask", "io.metamask"),
        ("Rainbow", "me.rainbow"),
        ("Rabby", "io.rabby"),
        ("Base", "base"),
        ("Phantom", "app.phantom"),
    ];

    let mut all_wallets = Vec::new();

    // First, add all detected wallets
    for wallet in detected_wallets.iter() {
        all_wallets.push(wallet.clone());
    }

    // Then, add undetected supported wallets (placeholders)
    for (name, rdns) in supported_wallets {
        if !detected_names.contains(name) {
            // Create a placeholder wallet without a real provider
            // The provider will be null/undefined, which we can check when connecting
            all_wallets.push(DiscoveredWallet {
                info: WalletInfo {
                    uuid: format!("placeholder_{}", name.to_lowercase().replace(" ", "_")),
                    name: name.to_string(),
                    icon: get_wallet_icon_url(name),
                    rdns: Some(rdns.to_string()),
                },
            });
            web_sys::console::log_1(&format!("➕ Added placeholder wallet: {}", name).into());
        }
    }

    web_sys::console::log_1(
        &format!(
            "📋 Total wallets (detected + placeholders): {}",
            all_wallets.len()
        )
        .into(),
    );

    all_wallets
}

// Get wallet icon URL by wallet name
// Uses local logo files from imgs directory
fn get_wallet_icon_url(name: &str) -> String {
    match name {
        "Rabby" => "/imgs/rabby-logo.png".to_string(),
        "Base" => "/imgs/base-logo.png".to_string(),
        "Phantom" => "/imgs/phantom-logo.png".to_string(),
        "MetaMask" => "/imgs/metamask-logo.svg".to_string(),
        "Rainbow" => "/imgs/rainbow-logo.png".to_string(),
        _ => String::new(),
    }
}

// Get provider name from provider object
// skip_rabby_check: if true, skip isRabby check (useful when checking window.ethereum
//                   after window.rabby has already been added)
fn get_provider_name(provider: &JsValue, skip_rabby_check: bool) -> Option<String> {
    // Try isRabby first (Rabby Wallet), unless we're skipping it
    if !skip_rabby_check {
        if let Ok(is_rabby) = Reflect::get(provider, &"isRabby".into()) {
            if is_rabby.as_bool().unwrap_or(false) {
                return Some("Rabby".to_string());
            }
        }
    }

    // Try isMetaMask (check before other wallets since Rabby may wrap MetaMask)
    // If both isRabby and isMetaMask are true, and we're checking window.ethereum,
    // we want to identify it as MetaMask if Rabby was already added via window.rabby
    if let Ok(is_meta) = Reflect::get(provider, &"isMetaMask".into()) {
        if is_meta.as_bool().unwrap_or(false) {
            // If isRabby is also true, check if we should still identify as MetaMask
            if skip_rabby_check {
                web_sys::console::log_1(
                    &"✅ Identified as MetaMask (skipping Rabby check as it's already added)"
                        .into(),
                );
                return Some("MetaMask".to_string());
            }
            // Only return MetaMask if isRabby is false, otherwise let isRabby take precedence
            if let Ok(is_rabby) = Reflect::get(provider, &"isRabby".into()) {
                if !is_rabby.as_bool().unwrap_or(false) {
                    return Some("MetaMask".to_string());
                }
            } else {
                return Some("MetaMask".to_string());
            }
        }
    }

    // Try isRainbow (Rainbow Wallet)
    if let Ok(is_rainbow) = Reflect::get(provider, &"isRainbow".into()) {
        if is_rainbow.as_bool().unwrap_or(false) {
            web_sys::console::log_1(&"✅ Identified Rainbow via isRainbow".into());
            return Some("Rainbow".to_string());
        }
    }

    // Try isBase (Base Wallet)
    if let Ok(is_base) = Reflect::get(provider, &"isBase".into()) {
        if is_base.as_bool().unwrap_or(false) {
            return Some("Base".to_string());
        }
    }

    // Try isPhantom (Phantom Wallet)
    if let Ok(is_phantom) = Reflect::get(provider, &"isPhantom".into()) {
        if is_phantom.as_bool().unwrap_or(false) {
            return Some("Phantom".to_string());
        }
    }

    // Try provider name property (some wallets use this)
    if let Ok(name) = Reflect::get(provider, &"providerName".into()) {
        if let Some(name_str) = name.as_string() {
            web_sys::console::log_1(&format!("🔍 Found providerName: {}", name_str).into());
            // Normalize common wallet names
            let normalized = match name_str.as_str() {
                name if name.to_lowercase().contains("metamask") => "MetaMask".to_string(),
                name if name.to_lowercase().contains("rainbow") => {
                    web_sys::console::log_1(&"✅ Identified Rainbow via providerName".into());
                    "Rainbow".to_string()
                }
                name if name.to_lowercase().contains("base") => "Base".to_string(),
                name if name.to_lowercase().contains("phantom") => "Phantom".to_string(),
                name if name.to_lowercase().contains("rabby") => "Rabby".to_string(),
                _ => name_str,
            };
            return Some(normalized);
        }
    }

    // Try chainId and other properties that might help identify Rainbow
    // Rainbow sometimes uses specific chainId or network properties
    if let Ok(chain_id) = Reflect::get(provider, &"chainId".into()) {
        if let Some(chain_id_str) = chain_id.as_string() {
            web_sys::console::log_1(&format!("🔍 Found chainId: {}", chain_id_str).into());
        }
    }

    // Try to check if provider has Rainbow-specific methods or properties
    if let Ok(_) = Reflect::get(provider, &"request".into()) {
        // Check if it's a valid EIP-1193 provider
        // Try to get the constructor name or other identifying info
        if let Ok(proto) = Reflect::get(provider, &"constructor".into()) {
            if let Some(proto_obj) = proto.dyn_ref::<Object>() {
                if let Ok(constructor_name) = Reflect::get(proto_obj, &"name".into()) {
                    if let Some(name_str) = constructor_name.as_string() {
                        web_sys::console::log_1(
                            &format!("🔍 Found constructor name: {}", name_str).into(),
                        );
                        if name_str.to_lowercase().contains("rainbow") {
                            return Some("Rainbow".to_string());
                        }
                    }
                }
            }
        }
    }

    // Try wallet name from wallet property (some wallets expose this)
    if let Ok(wallet_name) = Reflect::get(provider, &"wallet".into()) {
        if let Some(wallet_obj) = wallet_name.dyn_ref::<Object>() {
            if let Ok(name) = Reflect::get(wallet_obj, &"name".into()) {
                if let Some(name_str) = name.as_string() {
                    web_sys::console::log_1(&format!("🔍 Found wallet.name: {}", name_str).into());
                    if name_str.to_lowercase().contains("rainbow") {
                        return Some("Rainbow".to_string());
                    }
                    return Some(name_str);
                }
            }
        }
    }

    // Try _state property (Rainbow sometimes uses this)
    if let Ok(state) = Reflect::get(provider, &"_state".into()) {
        if let Some(state_obj) = state.dyn_ref::<Object>() {
            if let Ok(name) = Reflect::get(state_obj, &"name".into()) {
                if let Some(name_str) = name.as_string() {
                    web_sys::console::log_1(&format!("🔍 Found _state.name: {}", name_str).into());
                    if name_str.to_lowercase().contains("rainbow") {
                        return Some("Rainbow".to_string());
                    }
                }
            }
        }
    }

    // Default - return None to skip unknown providers
    web_sys::console::log_1(&"⚠️ Could not identify provider type".into());
    None
}

// Get current wallet account info
pub async fn get_account() -> Result<WalletAccount, String> {
    let window = get_window()?;

    // Check current provider
    if let Some(provider) = get_current_provider(&window) {
        // Get accounts via EIP-1193 request
        let accounts = request_accounts(&provider).await?;

        if let Some(accounts_arr) = accounts {
            if accounts_arr.length() > 0 {
                let account = accounts_arr.get(0).as_string();

                // Get chain ID
                let chain_id = get_chain_id(&provider).await.ok();

                // Get provider name (don't skip Rabby check here as we want to identify the actual provider)
                let connector = get_provider_name(&provider, false);

                return Ok(WalletAccount {
                    address: account,
                    is_connected: true,
                    is_connecting: false,
                    is_disconnected: false,
                    chain_id,
                    connector,
                    fid: None, // Will be fetched separately
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
        fid: None,
    })
}

// Request accounts from provider (EIP-1193)
async fn request_accounts(provider: &JsValue) -> Result<Option<Array>, String> {
    let request_fn = Reflect::get(provider, &"request".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("request method not found on provider".to_string())?;

    let request_params = Object::new();
    Reflect::set(&request_params, &"method".into(), &"eth_accounts".into())
        .map_err(|_| "Failed to set method".to_string())?;

    let params_array = Array::new();
    Reflect::set(&request_params, &"params".into(), &params_array.into())
        .map_err(|_| "Failed to set params".to_string())?;

    let accounts_promise = request_fn
        .call1(provider, &request_params.into())
        .map_err(|e| format!("Failed to call request: {:?}", e))?;

    let promise = Promise::from(accounts_promise);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to get accounts: {:?}", e))?;

    if let Some(accounts) = result.dyn_ref::<Array>() {
        Ok(Some(accounts.clone()))
    } else {
        Ok(None)
    }
}

// Get chain ID from provider (EIP-1193)
async fn get_chain_id(provider: &JsValue) -> Result<u64, String> {
    let request_fn = Reflect::get(provider, &"request".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("request method not found on provider".to_string())?;

    let request_params = Object::new();
    Reflect::set(&request_params, &"method".into(), &"eth_chainId".into())
        .map_err(|_| "Failed to set method".to_string())?;

    let params_array = Array::new();
    Reflect::set(&request_params, &"params".into(), &params_array.into())
        .map_err(|_| "Failed to set params".to_string())?;

    let chain_id_promise = request_fn
        .call1(provider, &request_params.into())
        .map_err(|e| format!("Failed to call request: {:?}", e))?;

    let promise = Promise::from(chain_id_promise);
    let result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to get chainId: {:?}", e))?;

    // Parse hex string to u64
    if let Some(chain_id_str) = result.as_string() {
        if chain_id_str.starts_with("0x") {
            u64::from_str_radix(&chain_id_str[2..], 16)
                .map_err(|e| format!("Failed to parse chainId: {}", e))
        } else {
            chain_id_str
                .parse::<u64>()
                .map_err(|e| format!("Failed to parse chainId: {}", e))
        }
    } else {
        result
            .as_f64()
            .map(|n| n as u64)
            .ok_or("ChainId is not a valid number".to_string())
    }
}

// Connect to a specific wallet by provider
pub async fn connect_to_wallet(provider_uuid: &str) -> Result<(), String> {
    web_sys::console::log_1(&format!("🔌 Connecting to wallet: {}", provider_uuid).into());

    let window = get_window()?;

    // Check if this is a placeholder wallet (not installed)
    if provider_uuid.starts_with("placeholder_") {
        // Extract wallet name from UUID (e.g., "placeholder_metamask" -> "MetaMask")
        let wallet_name = provider_uuid
            .strip_prefix("placeholder_")
            .unwrap_or(provider_uuid)
            .replace("_", " ")
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        // Special case for known wallets
        let wallet_name = match wallet_name.as_str() {
            "Meta Mask" => "MetaMask",
            _ => &wallet_name,
        };

        web_sys::console::log_1(
            &format!(
                "⚠️ Placeholder wallet clicked: {} - wallet may not be detected correctly",
                wallet_name
            )
            .into(),
        );
        return Err(format!("{} appears to not be installed or was not detected. Please ensure the extension is enabled and refresh the page.", wallet_name));
    }

    let provider = find_provider_by_uuid(&window, provider_uuid)?;

    // Check if provider is null or undefined (placeholder)
    if provider.is_null() || provider.is_undefined() {
        return Err("Wallet provider not found. Please make sure the wallet extension is installed and refresh the page.".to_string());
    }

    // Ensure we always reconnect, even if wallet was previously connected
    // First, clear the current provider to force fresh connection
    // Then try to revoke existing permissions to force re-authorization
    let request_fn = Reflect::get(&provider, &"request".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("request method not found on provider".to_string())?;

    // Clear current provider first to ensure we start fresh
    Reflect::set(&window, &"__walletProvider".into(), &JsValue::NULL).ok();

    // According to EIP-6963 best practices, we should use the provider directly
    // without revoking permissions. However, to force popup, we can try to revoke first.
    // But note: MetaMask's behavior is that if already authorized, eth_requestAccounts won't show popup.
    // To force popup, user needs to disconnect in MetaMask settings first.

    // Try to revoke permissions first (optional - may not work for all wallets)
    web_sys::console::log_1(
        &"🔌 Attempting to revoke existing permissions to force re-authorization...".into(),
    );
    let revoke_params = Object::new();
    Reflect::set(
        &revoke_params,
        &"method".into(),
        &"wallet_revokePermissions".into(),
    )
    .ok();
    let permissions_array = Array::new();
    let permission_obj = Object::new();
    Reflect::set(
        &permission_obj,
        &"parentCapability".into(),
        &"eth_accounts".into(),
    )
    .ok();
    permissions_array.push(&permission_obj);
    Reflect::set(&revoke_params, &"params".into(), &permissions_array.into()).ok();

    let revoke_result = request_fn.call1(&provider, &revoke_params.into());
    if let Ok(revoke_promise) = revoke_result {
        let promise = Promise::from(revoke_promise);
        let _ = JsFuture::from(promise).await; // Ignore errors
        gloo_timers::future::TimeoutFuture::new(300).await;
    }

    // Now set the provider and request connection
    // Following EIP-6963 best practices: use provider.request() directly
    set_current_provider(&window, &provider)?;

    web_sys::console::log_1(&"🔌 Requesting wallet connection via eth_requestAccounts...".into());

    // Use provider.request() directly as per EIP-6963 documentation
    let request_params = Object::new();
    Reflect::set(
        &request_params,
        &"method".into(),
        &"eth_requestAccounts".into(),
    )
    .map_err(|_| "Failed to set method".to_string())?;

    let params_array = Array::new();
    Reflect::set(&request_params, &"params".into(), &params_array.into())
        .map_err(|_| "Failed to set params".to_string())?;

    // Call provider.request() directly - this is the standard way per EIP-6963
    let connect_promise = request_fn
        .call1(&provider, &request_params.into())
        .map_err(|e| format!("Failed to call request: {:?}", e))?;

    let promise = Promise::from(connect_promise);
    let _result = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Wallet connection failed: {:?}", e))?;

    // Set up event listeners
    setup_provider_events(&window, &provider)?;

    web_sys::console::log_1(&"✅ Wallet connected".into());
    Ok(())
}

// Find provider by UUID
fn find_provider_by_uuid(window: &Window, uuid: &str) -> Result<JsValue, String> {
    web_sys::console::log_1(&format!("🔍 Finding provider for UUID: {}", uuid).into());

    // Check discovered wallets (EIP-6963) - this is the most reliable way
    // According to EIP-6963 best practices, we should prioritize EIP-6963 discovered wallets
    if let Ok(wallets) = Reflect::get(window, &"__discoveredWallets".into()) {
        if let Some(wallets_arr) = wallets.dyn_ref::<Array>() {
            for i in 0..wallets_arr.length() {
                if let Some(wallet_obj) = wallets_arr.get(i).dyn_ref::<Object>() {
                    if let Ok(info) = Reflect::get(wallet_obj, &"info".into()) {
                        if let Ok(provider) = Reflect::get(wallet_obj, &"provider".into()) {
                            if let Some(info_obj) = info.dyn_ref::<Object>() {
                                // First try exact UUID match (EIP-6963 UUID)
                                if let Ok(info_uuid) = Reflect::get(info_obj, &"uuid".into()) {
                                    if let Some(uuid_str) = info_uuid.as_string() {
                                        if uuid_str == uuid {
                                            web_sys::console::log_1(&format!("✅ Found provider in EIP-6963 discovered wallets (exact UUID match): {}", uuid).into());
                                            return Ok(provider);
                                        }
                                    }
                                }

                                // If UUID doesn't match, try matching by name or rdns for common wallets
                                // This handles cases where UI passes "metamask" but EIP-6963 UUID is different
                                if uuid == "metamask"
                                    || uuid == "rainbow"
                                    || uuid == "base"
                                    || uuid == "rabby_wallet"
                                {
                                    let name = Reflect::get(info_obj, &"name".into())
                                        .ok()
                                        .and_then(|v| v.as_string());
                                    let rdns = Reflect::get(info_obj, &"rdns".into())
                                        .ok()
                                        .and_then(|v| v.as_string());

                                    let matches = match uuid {
                                        "metamask" => {
                                            name.as_ref().map(|n| n == "MetaMask").unwrap_or(false)
                                                || rdns
                                                    .as_ref()
                                                    .map(|r| r.contains("metamask"))
                                                    .unwrap_or(false)
                                        }
                                        "rainbow" => {
                                            name.as_ref().map(|n| n == "Rainbow").unwrap_or(false)
                                                || rdns
                                                    .as_ref()
                                                    .map(|r| r.contains("rainbow"))
                                                    .unwrap_or(false)
                                        }
                                        "base" => {
                                            name.as_ref().map(|n| n == "Base").unwrap_or(false)
                                                || rdns
                                                    .as_ref()
                                                    .map(|r| r.contains("base"))
                                                    .unwrap_or(false)
                                        }
                                        "rabby_wallet" => {
                                            name.as_ref().map(|n| n == "Rabby").unwrap_or(false)
                                                || rdns
                                                    .as_ref()
                                                    .map(|r| r.contains("rabby"))
                                                    .unwrap_or(false)
                                        }
                                        _ => false,
                                    };

                                    if matches {
                                        web_sys::console::log_1(&format!("✅ Found provider in EIP-6963 discovered wallets (name/rdns match for {}): {:?}", uuid, name).into());
                                        return Ok(provider);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check window.rabby
    if uuid == "rabby_wallet" {
        if let Ok(rabby) = Reflect::get(window, &"rabby".into()) {
            if !rabby.is_null() && !rabby.is_undefined() {
                return Ok(rabby);
            }
        }
    }

    // Check window.phantom
    if uuid == "phantom_ethereum" {
        if let Ok(phantom) = Reflect::get(window, &"phantom".into()) {
            if !phantom.is_null() && !phantom.is_undefined() {
                if let Some(provider) = phantom.dyn_ref::<Object>() {
                    if let Ok(ethereum) = Reflect::get(provider, &"ethereum".into()) {
                        if !ethereum.is_null() && !ethereum.is_undefined() {
                            return Ok(ethereum);
                        }
                    }
                }
            }
        }
    }

    // Check window.ethereum with wallet-specific UUIDs (metamask, rainbow, base, etc.)
    if uuid == "metamask"
        || uuid == "rainbow"
        || uuid == "base"
        || uuid == "ethereum_default"
        || uuid.starts_with("ethereum_")
    {
        if let Ok(ethereum) = Reflect::get(window, &"ethereum".into()) {
            if !ethereum.is_null() && !ethereum.is_undefined() {
                if let Some(providers) = ethereum.dyn_ref::<Array>() {
                    // Multiple providers - search by name to find the right one
                    for i in 0..providers.length() {
                        if let Some(provider) = providers.get(i).dyn_ref::<Object>() {
                            // Skip Rabby check when looking for MetaMask/Rainbow/Base
                            let skip_rabby =
                                uuid == "metamask" || uuid == "rainbow" || uuid == "base";
                            if let Some(name) = get_provider_name(provider, skip_rabby) {
                                let expected_uuid = match name.as_str() {
                                    "MetaMask" => "metamask",
                                    "Rainbow" => "rainbow",
                                    "Base" => "base",
                                    _ => "",
                                };
                                if expected_uuid == uuid {
                                    // Verify it's actually the right provider by checking isMetaMask/isRainbow/isBase
                                    if uuid == "metamask" {
                                        if let Ok(is_metamask) =
                                            Reflect::get(provider, &"isMetaMask".into())
                                        {
                                            if !is_metamask.as_bool().unwrap_or(false) {
                                                web_sys::console::log_1(&"⚠️ Provider identified as MetaMask but isMetaMask is false, skipping".into());
                                                continue;
                                            }
                                        }
                                    }
                                    web_sys::console::log_1(&format!("✅ Found {} provider in window.ethereum array (index {})", uuid, i).into());
                                    return Ok(provider.into());
                                }
                            }
                        }
                    }
                    // Legacy support: try index-based lookup
                    if uuid.starts_with("ethereum_") {
                        if let Some(index_str) = uuid.strip_prefix("ethereum_") {
                            if let Ok(index) = index_str.parse::<u32>() {
                                if index < providers.length() {
                                    return Ok(providers.get(index));
                                }
                            }
                        }
                    }
                } else if let Some(_provider) = ethereum.dyn_ref::<Object>() {
                    // Single provider
                    // Skip Rabby check when looking for MetaMask/Rainbow/Base
                    let skip_rabby = uuid == "metamask" || uuid == "rainbow" || uuid == "base";
                    if let Some(name) = get_provider_name(&ethereum, skip_rabby) {
                        let expected_uuid = match name.as_str() {
                            "MetaMask" => "metamask",
                            "Rainbow" => "rainbow",
                            "Base" => "base",
                            _ => "ethereum_default",
                        };
                        if expected_uuid == uuid || uuid == "ethereum_default" {
                            // Verify it's actually the right provider
                            if uuid == "metamask" {
                                if let Ok(is_metamask) =
                                    Reflect::get(&ethereum, &"isMetaMask".into())
                                {
                                    if !is_metamask.as_bool().unwrap_or(false) {
                                        return Err("Provider identified as MetaMask but isMetaMask is false".to_string());
                                    }
                                }
                            }
                            web_sys::console::log_1(
                                &format!("✅ Found {} provider in window.ethereum", uuid).into(),
                            );
                            return Ok(ethereum);
                        }
                    } else if uuid == "ethereum_default" {
                        // If we can't identify the provider, but UUID is ethereum_default, return it
                        return Ok(ethereum);
                    }
                }
            }
        }
    }

    Err(format!("Provider with UUID {} not found", uuid))
}

// Set up provider event listeners (accountsChanged, chainChanged)
fn setup_provider_events(window: &Window, provider: &JsValue) -> Result<(), String> {
    // Set up accountsChanged listener
    if let Ok(on_fn) = Reflect::get(provider, &"on".into()) {
        if let Some(on) = on_fn.dyn_ref::<Function>() {
            // accountsChanged event
            let window_clone1 = window.clone();
            let accounts_changed = Closure::wrap(Box::new(move |_accounts: JsValue| {
                web_sys::console::log_1(&"✅ Accounts changed".into());
                // Trigger a custom event that the app can listen to
                if let Ok(event) = Event::new("wallet:accountsChanged") {
                    window_clone1.dispatch_event(&event).ok();
                }
            }) as Box<dyn Fn(JsValue)>);

            on.call2(
                provider,
                &"accountsChanged".into(),
                accounts_changed.as_ref().unchecked_ref(),
            )
            .ok();
            accounts_changed.forget();

            // chainChanged event
            let window_clone2 = window.clone();
            let chain_changed = Closure::wrap(Box::new(move |_chain_id: JsValue| {
                web_sys::console::log_1(&"✅ Chain changed".into());
                if let Ok(event) = Event::new("wallet:chainChanged") {
                    window_clone2.dispatch_event(&event).ok();
                }
            }) as Box<dyn Fn(JsValue)>);

            on.call2(
                provider,
                &"chainChanged".into(),
                chain_changed.as_ref().unchecked_ref(),
            )
            .ok();
            chain_changed.forget();
        }
    }

    Ok(())
}


// Disconnect wallet
pub async fn disconnect() -> Result<(), String> {
    let window = get_window()?;
    set_current_provider(&window, &JsValue::NULL)?;
    web_sys::console::log_1(&"✅ Wallet disconnected".into());
    Ok(())
}

// Sign typed data (EIP-712)
pub async fn sign_eip712(typed_data: &str) -> Result<String, String> {
    let account = get_account().await?;

    if account.address.is_none() {
        return Err("Wallet not connected".to_string());
    }

    let address = account.address.as_ref().unwrap();
    let window = get_window()?;
    let provider =
        get_current_provider(&window).ok_or("No wallet provider connected".to_string())?;

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

    let client = PolyEndpointClient::new(contract_address);

    let network = if rpc_url.contains("sepolia") {
        "base-sepolia"
    } else if rpc_url.contains("mainnet") {
        "base-mainnet"
    } else {
        rpc_url
    };

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
            Ok(EndpointData {
                endpoints: vec![],
                contract_address: contract_address.to_string(),
                network: network.to_string(),
            })
        }
    }
}

// Ping an endpoint and measure latency
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


// Get full profile data for an address (includes avatar, username, etc.)
pub async fn get_profile_for_address(
    api_url: &str,
    address: &str,
) -> Result<Option<crate::models::ProfileData>, String> {
    web_sys::console::log_1(&format!("🔍 Fetching profile for address: {}", address).into());

    let window = get_window()?;
    let fetch_fn = Reflect::get(&window, &"fetch".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("fetch not available".to_string())?;

    // Use the ethereum-specific endpoint
    let url = format!(
        "{}/api/profiles/address/ethereum/{}",
        api_url.trim_end_matches('/'),
        address
    );

    web_sys::console::log_1(&format!("🌐 Fetching profile: {}", url).into());

    // Create request with authentication
    let request = web_sys::Request::new_with_str(&url)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    // Add authentication headers if configured
    if let Err(e) = crate::api::add_auth_headers(&request.headers(), "GET", &url, None) {
        web_sys::console::warn_1(&format!("⚠️ Failed to add auth headers: {}", e).into());
    }

    let fetch_promise = fetch_fn
        .call1(&window, &request.into())
        .map_err(|e| format!("Failed to call fetch: {:?}", e))?;

    let promise = Promise::from(fetch_promise);
    let response_value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;

    let status = response.status();

    if status == 200 {
        let text_promise = response.text().map_err(|_| "Failed to get response text")?;
        let text_value = JsFuture::from(text_promise)
            .await
            .map_err(|e| format!("Failed to read response: {:?}", e))?;
        let text = text_value.as_string().ok_or("Response is not a string")?;

        // Parse the API response
        match serde_json::from_str::<crate::models::ApiResponse<serde_json::Value>>(&text) {
            Ok(api_response) => {
                if api_response.success {
                    if let Some(data) = api_response.data {
                        // The API returns data in nested structure: data.data
                        if let Some(inner_data) = data.get("data") {
                            // Try to parse as ProfileData
                            match serde_json::from_value::<crate::models::ProfileData>(
                                inner_data.clone(),
                            ) {
                                Ok(profile) => {
                                    web_sys::console::log_1(
                                        &format!(
                                            "✅ Found profile: FID {}, username: {:?}",
                                            profile.fid, profile.username
                                        )
                                        .into(),
                                    );
                                    return Ok(Some(profile));
                                }
                                Err(e) => {
                                    web_sys::console::log_1(
                                        &format!("⚠️ Failed to parse profile data: {}", e).into(),
                                    );
                                }
                            }
                        }
                    }
                    Ok(None)
                } else {
                    let error_msg = api_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string());
                    web_sys::console::log_1(&format!("⚠️ API error: {}", error_msg).into());
                    Ok(None)
                }
            }
            Err(e) => {
                web_sys::console::log_1(&format!("⚠️ Failed to parse API response: {}", e).into());
                Ok(None)
            }
        }
    } else if status == 404 {
        web_sys::console::log_1(&"ℹ️ No profile found for this address".into());
        Ok(None)
    } else {
        web_sys::console::log_1(&format!("⚠️ API request failed with status: {}", status).into());
        Ok(None)
    }
}

// Get FID associated with an Ethereum address
pub async fn get_fid_for_address(api_url: &str, address: &str) -> Result<Option<i64>, String> {
    web_sys::console::log_1(&format!("🔍 Fetching FID for address: {}", address).into());

    let window = get_window()?;
    let fetch_fn = Reflect::get(&window, &"fetch".into())
        .ok()
        .and_then(|f| f.dyn_ref::<Function>().cloned())
        .ok_or("fetch not available".to_string())?;

    // Use the ethereum-specific endpoint
    let url = format!(
        "{}/api/profiles/address/ethereum/{}",
        api_url.trim_end_matches('/'),
        address
    );

    web_sys::console::log_1(&format!("🌐 Fetching: {}", url).into());

    // Create request with authentication
    let request = web_sys::Request::new_with_str(&url)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    // Add authentication headers if configured
    if let Err(e) = crate::api::add_auth_headers(&request.headers(), "GET", &url, None) {
        web_sys::console::warn_1(&format!("⚠️ Failed to add auth headers: {}", e).into());
    }

    let fetch_promise = fetch_fn
        .call1(&window, &request.into())
        .map_err(|e| format!("Failed to call fetch: {:?}", e))?;

    let promise = Promise::from(fetch_promise);
    let response_value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;

    let status = response.status();

    if status == 200 {
        let text_promise = response.text().map_err(|_| "Failed to get response text")?;
        let text_value = JsFuture::from(text_promise)
            .await
            .map_err(|e| format!("Failed to read response: {:?}", e))?;
        let text = text_value.as_string().ok_or("Response is not a string")?;

        // Parse the API response
        match serde_json::from_str::<crate::models::ApiResponse<serde_json::Value>>(&text) {
            Ok(api_response) => {
                if api_response.success {
                    if let Some(data) = api_response.data {
                        // The API returns data in nested structure: data.data.fid
                        // First try to get fid from data.data (nested structure)
                        if let Some(inner_data) = data.get("data") {
                            if let Some(inner_data_obj) = inner_data.as_object() {
                                // Try to get fid from inner data
                                if let Some(fid_value) = inner_data_obj.get("fid") {
                                    if let Some(fid) = fid_value.as_i64() {
                                        web_sys::console::log_1(
                                            &format!("✅ Found FID in data.data: {}", fid).into(),
                                        );
                                        return Ok(Some(fid));
                                    }
                                }

                                // Try to get fids array from inner data
                                if let Some(fids_value) = inner_data_obj.get("fids") {
                                    if let Some(fids_array) = fids_value.as_array() {
                                        if let Some(first) = fids_array.first() {
                                            if let Some(fid) = first.as_i64() {
                                                web_sys::console::log_1(&format!("✅ Found FID (first of {}) in data.data: {}", fids_array.len(), fid).into());
                                                return Ok(Some(fid));
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback: try to get fid directly from data (flat structure)
                        if let Some(fid_value) = data.get("fid") {
                            if let Some(fid) = fid_value.as_i64() {
                                web_sys::console::log_1(
                                    &format!("✅ Found FID in data: {}", fid).into(),
                                );
                                return Ok(Some(fid));
                            }
                        }

                        // Try fids array directly from data
                        if let Some(fids_value) = data.get("fids") {
                            if let Some(fids_array) = fids_value.as_array() {
                                if let Some(first) = fids_array.first() {
                                    if let Some(fid) = first.as_i64() {
                                        web_sys::console::log_1(
                                            &format!(
                                                "✅ Found FID (first of {}) in data: {}",
                                                fids_array.len(),
                                                fid
                                            )
                                            .into(),
                                        );
                                        return Ok(Some(fid));
                                    }
                                }
                            }
                        }

                        web_sys::console::log_1(&"⚠️ No FID found in response".into());
                        Ok(None)
                    } else {
                        web_sys::console::log_1(&"⚠️ No data in API response".into());
                        Ok(None)
                    }
                } else {
                    let error_msg = api_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string());
                    web_sys::console::log_1(&format!("⚠️ API error: {}", error_msg).into());
                    Ok(None)
                }
            }
            Err(e) => {
                web_sys::console::log_1(&format!("⚠️ Failed to parse API response: {}", e).into());
                Ok(None)
            }
        }
    } else if status == 404 {
        // Address not found - this is normal, just return None
        web_sys::console::log_1(&"ℹ️ No profile found for this address".into());
        Ok(None)
    } else {
        web_sys::console::log_1(&format!("⚠️ API request failed with status: {}", status).into());
        Ok(None)
    }
}
