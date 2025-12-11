use js_sys::Object;
use js_sys::Promise;
use js_sys::Reflect;
use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

// Farcaster Mini App Context
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MiniAppContext {
    pub user: Option<ContextUser>,
    pub cast: Option<ContextCast>,
    pub channel: Option<ContextChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextUser {
    pub fid: Option<i64>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub pfp_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextCast {
    pub hash: Option<String>,
    pub author: Option<ContextUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextChannel {
    pub id: Option<String>,
    pub name: Option<String>,
    pub image_url: Option<String>,
}

// Helper function to get window object
fn get_window() -> Result<web_sys::Window, String> {
    window().ok_or("No window object".to_string())
}

// Helper function to get Farcaster SDK from window
fn get_farcaster_sdk(window: &web_sys::Window) -> Result<JsValue, String> {
    // The SDK is typically available as window.sdk or imported module
    // Try common locations
    web_sys::console::log_1(&"[Farcaster SDK] Checking window.sdk...".into());
    if let Ok(sdk) = Reflect::get(window, &"sdk".into()) {
        if !sdk.is_undefined() && !sdk.is_null() {
            web_sys::console::log_1(&"[Farcaster SDK] ✅ Found on window.sdk".into());
            return Ok(sdk);
        } else {
            web_sys::console::log_1(
                &"[Farcaster SDK] window.sdk exists but is null/undefined".into(),
            );
        }
    } else {
        web_sys::console::log_1(&"[Farcaster SDK] window.sdk does not exist".into());
    }

    // Try window.farcaster or window.farcasterSDK
    web_sys::console::log_1(&"[Farcaster SDK] Checking window.farcaster...".into());
    if let Ok(sdk) = Reflect::get(window, &"farcaster".into()) {
        if !sdk.is_undefined() && !sdk.is_null() {
            web_sys::console::log_1(&"[Farcaster SDK] ✅ Found on window.farcaster".into());
            return Ok(sdk);
        } else {
            web_sys::console::log_1(
                &"[Farcaster SDK] window.farcaster exists but is null/undefined".into(),
            );
        }
    } else {
        web_sys::console::log_1(&"[Farcaster SDK] window.farcaster does not exist".into());
    }

    web_sys::console::log_1(&"[Farcaster SDK] Checking window.farcasterSDK...".into());
    if let Ok(sdk) = Reflect::get(window, &"farcasterSDK".into()) {
        if !sdk.is_undefined() && !sdk.is_null() {
            web_sys::console::log_1(&"[Farcaster SDK] ✅ Found on window.farcasterSDK".into());
            return Ok(sdk);
        } else {
            web_sys::console::log_1(
                &"[Farcaster SDK] window.farcasterSDK exists but is null/undefined".into(),
            );
        }
    } else {
        web_sys::console::log_1(&"[Farcaster SDK] window.farcasterSDK does not exist".into());
    }

    // Check for other possible locations - sometimes SDK might be nested
    // Check window.__farcaster__ or similar
    let possible_paths = vec![
        "__farcaster__",
        "__farcasterSDK__",
        "FarcasterSDK",
        "Farcaster",
    ];
    for path in possible_paths {
        web_sys::console::log_1(&format!("[Farcaster SDK] Checking window.{}...", path).into());
        if let Ok(sdk) = Reflect::get(window, &path.into()) {
            if !sdk.is_undefined() && !sdk.is_null() {
                // Check if it's an object with an sdk property
                if let Some(sdk_obj) = sdk.dyn_ref::<Object>() {
                    if let Ok(nested_sdk) = Reflect::get(sdk_obj, &"sdk".into()) {
                        if !nested_sdk.is_undefined() && !nested_sdk.is_null() {
                            web_sys::console::log_1(
                                &format!(
                                    "[Farcaster SDK] ✅ Found nested SDK in window.{}.sdk",
                                    path
                                )
                                .into(),
                            );
                            return Ok(nested_sdk);
                        }
                    }
                }
                // Or it might be the SDK itself
                if let Some(sdk_obj) = sdk.dyn_ref::<Object>() {
                    // Check if it has typical SDK methods
                    if Reflect::get(sdk_obj, &"isInMiniApp".into()).is_ok()
                        || Reflect::get(sdk_obj, &"actions".into()).is_ok()
                    {
                        web_sys::console::log_1(
                            &format!("[Farcaster SDK] ✅ Found SDK in window.{}", path).into(),
                        );
                        return Ok(sdk);
                    }
                }
            }
        }
    }

    web_sys::console::log_1(&"[Farcaster SDK] ❌ SDK not found in any expected location".into());
    Err("Farcaster SDK not found. Make sure @farcaster/miniapp-sdk is loaded.".to_string())
}

/// Check if the app is running inside a Farcaster Mini App
/// Official method: Use sdk.isInMiniApp() from @farcaster/miniapp-sdk
pub async fn is_in_mini_app() -> Result<bool, String> {
    let window = get_window()?;
    web_sys::console::log_1(&"[Farcaster SDK] Starting is_in_mini_app() check...".into());

    // Official method: Get SDK and call sdk.isInMiniApp()
    // Try to get SDK with retries (SDK might be loading asynchronously)
    for attempt in 0..15 {
        web_sys::console::log_1(
            &format!(
                "[Farcaster SDK] Attempt {}/15: Getting SDK and calling isInMiniApp()...",
                attempt + 1
            )
            .into(),
        );

        match get_farcaster_sdk(&window) {
            Ok(sdk) => {
                web_sys::console::log_1(
                    &"[Farcaster SDK] ✅ SDK found! Calling sdk.isInMiniApp()...".into(),
                );
                // Use official method: sdk.isInMiniApp()
                let is_in_mini_app_fn = Reflect::get(&sdk, &"isInMiniApp".into())
                    .ok()
                    .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
                    .ok_or_else(|| {
                        "[Farcaster SDK] SDK found but isInMiniApp() method not available"
                            .to_string()
                    })?;

                let promise = is_in_mini_app_fn
                    .call0(&sdk)
                    .map_err(|e| format!("Failed to call isInMiniApp: {:?}", e))?;

                let promise_value = promise
                    .dyn_into::<Promise>()
                    .map_err(|_| "isInMiniApp did not return a Promise".to_string())?;

                let result = JsFuture::from(promise_value)
                    .await
                    .map_err(|e| format!("Failed to await isInMiniApp: {:?}", e))?;

                let is_mini_app = result
                    .as_bool()
                    .ok_or_else(|| "isInMiniApp returned non-boolean value".to_string())?;

                web_sys::console::log_1(
                    &format!(
                        "[Farcaster SDK] sdk.isInMiniApp() returned: {}",
                        is_mini_app
                    )
                    .into(),
                );
                return Ok(is_mini_app);
            }
            Err(e) => {
                web_sys::console::log_1(
                    &format!(
                        "[Farcaster SDK] Attempt {}: SDK not available yet: {}",
                        attempt + 1,
                        e
                    )
                    .into(),
                );
                // SDK not available yet, wait and retry
                if attempt < 14 {
                    gloo_timers::future::TimeoutFuture::new(300).await;
                    continue;
                }
            }
        }
    }

    // SDK not found after all attempts
    web_sys::console::log_1(&"[Farcaster SDK] ❌ SDK not found after 15 attempts".into());
    web_sys::console::log_1(
        &"[Farcaster SDK] 🌐 Not in Mini App environment (SDK not available)".into(),
    );
    Ok(false)
}

/// Initialize Farcaster Mini App SDK and call ready()
/// This should be called when the app is fully loaded
pub async fn initialize() -> Result<(), String> {
    let window = get_window()?;
    let sdk = get_farcaster_sdk(&window)?;

    // Get actions.ready method
    let actions =
        Reflect::get(&sdk, &"actions".into()).map_err(|_| "SDK actions not found".to_string())?;

    let ready_fn = Reflect::get(&actions, &"ready".into())
        .ok()
        .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
        .ok_or("actions.ready is not a function".to_string())?;

    // Call ready() with empty options
    let ready_promise = ready_fn
        .call0(&actions)
        .map_err(|e| format!("Failed to call ready: {:?}", e))?;

    let promise = Promise::from(ready_promise);
    JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to await ready: {:?}", e))?;

    web_sys::console::log_1(&"✅ Farcaster Mini App SDK ready".into());
    Ok(())
}

/// Get the current Mini App context (user, cast, channel)
pub async fn get_context() -> Result<MiniAppContext, String> {
    let window = get_window()?;
    let sdk = get_farcaster_sdk(&window)?;

    // Get context property (it's a Promise)
    let context_promise =
        Reflect::get(&sdk, &"context".into()).map_err(|_| "SDK context not found".to_string())?;

    let promise = Promise::from(context_promise);
    let context_value = JsFuture::from(promise)
        .await
        .map_err(|e| format!("Failed to await context: {:?}", e))?;

    // Parse context from JavaScript object
    let context_obj = context_value
        .dyn_ref::<Object>()
        .ok_or("Context is not an object".to_string())?;

    // Parse user
    let user = if let Ok(user_value) = Reflect::get(context_obj, &"user".into()) {
        if !user_value.is_null() && !user_value.is_undefined() {
            let user_str = js_sys::JSON::stringify(&user_value)
                .map_err(|_| "Failed to stringify user".to_string())?
                .as_string()
                .ok_or("User stringify failed".to_string())?;
            serde_json::from_str::<ContextUser>(&user_str).ok()
        } else {
            None
        }
    } else {
        None
    };

    // Parse cast
    let cast = if let Ok(cast_value) = Reflect::get(context_obj, &"cast".into()) {
        if !cast_value.is_null() && !cast_value.is_undefined() {
            let cast_str = js_sys::JSON::stringify(&cast_value)
                .map_err(|_| "Failed to stringify cast".to_string())?
                .as_string()
                .ok_or("Cast stringify failed".to_string())?;
            serde_json::from_str::<ContextCast>(&cast_str).ok()
        } else {
            None
        }
    } else {
        None
    };

    // Parse channel
    let channel = if let Ok(channel_value) = Reflect::get(context_obj, &"channel".into()) {
        if !channel_value.is_null() && !channel_value.is_undefined() {
            let channel_str = js_sys::JSON::stringify(&channel_value)
                .map_err(|_| "Failed to stringify channel".to_string())?
                .as_string()
                .ok_or("Channel stringify failed".to_string())?;
            serde_json::from_str::<ContextChannel>(&channel_str).ok()
        } else {
            None
        }
    } else {
        None
    };

    Ok(MiniAppContext {
        user,
        cast,
        channel,
    })
}

/// Get the Ethereum provider from Farcaster SDK
#[allow(dead_code)]
pub async fn get_ethereum_provider() -> Result<JsValue, String> {
    let window = get_window()?;
    let sdk = get_farcaster_sdk(&window)?;

    // Get wallet.ethProvider
    let wallet =
        Reflect::get(&sdk, &"wallet".into()).map_err(|_| "SDK wallet not found".to_string())?;

    let eth_provider = Reflect::get(&wallet, &"ethProvider".into())
        .map_err(|_| "Ethereum provider not found".to_string())?;

    if eth_provider.is_null() || eth_provider.is_undefined() {
        return Err("Ethereum provider is not available".to_string());
    }

    Ok(eth_provider)
}

/// Trigger haptic feedback
#[allow(dead_code)]
pub fn haptic_impact(style: &str) -> Result<(), String> {
    let window = get_window()?;
    let sdk = get_farcaster_sdk(&window)?;

    let haptics =
        Reflect::get(&sdk, &"haptics".into()).map_err(|_| "SDK haptics not found".to_string())?;

    let impact_fn = Reflect::get(&haptics, &"impactOccurred".into())
        .ok()
        .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
        .ok_or("impactOccurred is not a function".to_string())?;

    // Call with style: "light", "medium", "heavy"
    let style_value = JsValue::from_str(style);
    impact_fn
        .call1(&haptics, &style_value)
        .map_err(|e| format!("Failed to call impactOccurred: {:?}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mini_app_context_serialization() {
        let context = MiniAppContext {
            user: Some(ContextUser {
                fid: Some(123),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                pfp_url: Some("https://example.com/pfp.png".to_string()),
            }),
            cast: None,
            channel: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        let parsed: MiniAppContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context, parsed);
    }

    #[test]
    fn test_mini_app_context_with_all_fields() {
        let context = MiniAppContext {
            user: Some(ContextUser {
                fid: Some(123),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                pfp_url: Some("https://example.com/pfp.png".to_string()),
            }),
            cast: Some(ContextCast {
                hash: Some("0xabc123".to_string()),
                author: Some(ContextUser {
                    fid: Some(456),
                    username: Some("author".to_string()),
                    display_name: Some("Author User".to_string()),
                    pfp_url: Some("https://example.com/author.png".to_string()),
                }),
            }),
            channel: Some(ContextChannel {
                id: Some("channel-1".to_string()),
                name: Some("Test Channel".to_string()),
                image_url: Some("https://example.com/channel.png".to_string()),
            }),
        };

        let json = serde_json::to_string(&context).unwrap();
        let parsed: MiniAppContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context, parsed);
    }

    #[test]
    fn test_context_user_serialization() {
        let user = ContextUser {
            fid: Some(456),
            username: Some("alice".to_string()),
            display_name: Some("Alice".to_string()),
            pfp_url: None,
        };

        let json = serde_json::to_string(&user).unwrap();
        let parsed: ContextUser = serde_json::from_str(&json).unwrap();
        assert_eq!(user, parsed);
    }

    #[test]
    fn test_context_user_with_all_fields() {
        let user = ContextUser {
            fid: Some(789),
            username: Some("bob".to_string()),
            display_name: Some("Bob Smith".to_string()),
            pfp_url: Some("https://example.com/bob.png".to_string()),
        };

        let json = serde_json::to_string(&user).unwrap();
        let parsed: ContextUser = serde_json::from_str(&json).unwrap();
        assert_eq!(user, parsed);
        assert_eq!(user.fid, Some(789));
        assert_eq!(user.username, Some("bob".to_string()));
    }

    #[test]
    fn test_context_cast_serialization() {
        let cast = ContextCast {
            hash: Some("0x123abc".to_string()),
            author: Some(ContextUser {
                fid: Some(789),
                username: Some("bob".to_string()),
                display_name: Some("Bob".to_string()),
                pfp_url: Some("https://example.com/bob.png".to_string()),
            }),
        };

        let json = serde_json::to_string(&cast).unwrap();
        let parsed: ContextCast = serde_json::from_str(&json).unwrap();
        assert_eq!(cast, parsed);
    }

    #[test]
    fn test_context_cast_without_author() {
        let cast = ContextCast {
            hash: Some("0x456def".to_string()),
            author: None,
        };

        let json = serde_json::to_string(&cast).unwrap();
        let parsed: ContextCast = serde_json::from_str(&json).unwrap();
        assert_eq!(cast, parsed);
    }

    #[test]
    fn test_context_channel_serialization() {
        let channel = ContextChannel {
            id: Some("channel-1".to_string()),
            name: Some("Test Channel".to_string()),
            image_url: Some("https://example.com/channel.png".to_string()),
        };

        let json = serde_json::to_string(&channel).unwrap();
        let parsed: ContextChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(channel, parsed);
    }

    #[test]
    fn test_context_channel_minimal() {
        let channel = ContextChannel {
            id: Some("channel-2".to_string()),
            name: None,
            image_url: None,
        };

        let json = serde_json::to_string(&channel).unwrap();
        let parsed: ContextChannel = serde_json::from_str(&json).unwrap();
        assert_eq!(channel, parsed);
    }

    #[test]
    fn test_empty_mini_app_context() {
        let context = MiniAppContext {
            user: None,
            cast: None,
            channel: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        let parsed: MiniAppContext = serde_json::from_str(&json).unwrap();
        assert_eq!(context, parsed);
    }
}
