use std::rc::Rc;

use wasm_bindgen::JsCast;

use crate::api::ApiResponse as RawApiResponse;
use crate::api::EndpointInfo;
use crate::models::*;
use crate::payment::PaymentRequirements;
use crate::wallet::WalletAccount;

/// Handle payment flow: sign and retry request with payment header
pub async fn handle_payment(
    requirements: &PaymentRequirements,
    account: &WalletAccount,
    api_url: &str,
    endpoint: &EndpointInfo,
    body: Option<String>,
) -> Result<RawApiResponse, String> {
    // Generate nonce and timestamp
    let nonce = crate::payment::generate_nonce();
    let timestamp = crate::payment::get_timestamp();

    // Get payer address
    let payer = account
        .address
        .as_ref()
        .ok_or("No wallet address available")?;

    // Build the full resource URI from current endpoint
    let full_resource = format!("{}{}", api_url, endpoint.path);

    // Create a new requirements with the actual resource URI
    let mut updated_requirements = requirements.clone();
    updated_requirements.resource = full_resource;

    // Create EIP-712 typed data
    let typed_data =
        crate::payment::create_eip712_typed_data(&updated_requirements, payer, &nonce, timestamp)?;

    // Debug: Log the typed data string
    web_sys::console::log_1(&format!("Typed data string: {}", typed_data).into());

    // Validate JSON parsing
    let _parsed_check: serde_json::Value = serde_json::from_str(&typed_data)
        .map_err(|e| format!("Failed to parse typed data as JSON: {}", e))?;
    web_sys::console::log_1(&"Typed data validation successful".to_string().into());

    // Sign with MetaMask
    let signature = crate::wallet::sign_eip712(&typed_data)
        .await
        .map_err(|e| format!("Failed to sign payment: {}", e))?;

    // Create payment payload with updated resource
    let payment_payload = crate::payment::create_payment_payload(
        &updated_requirements,
        payer,
        &signature,
        &nonce,
        timestamp,
    );

    // Debug: Log payment payload as JSON
    let payment_json = serde_json::to_string(&payment_payload)
        .unwrap_or_else(|_| "Failed to serialize payment payload".to_string());
    web_sys::console::log_1(&format!("Payment payload JSON: {}", payment_json).into());

    // Encode to base64
    let payment_header = payment_payload
        .to_base64()
        .map_err(|e| format!("Failed to encode payment: {}", e))?;

    // Debug: Log payment header
    let preview_len = payment_header.len().min(60);
    web_sys::console::log_1(
        &format!(
            "🔐 Payment header created (length: {}): {}...",
            payment_header.len(),
            &payment_header[..preview_len]
        )
        .into(),
    );

    // Retry request with payment
    web_sys::console::log_1(&"🔄 Retrying request with payment header...".into());
    let result = crate::api::make_request(api_url, endpoint, body, Some(payment_header))
        .await
        .map_err(|e| format!("Request with payment failed: {}", e));

    match &result {
        Ok(resp) => {
            web_sys::console::log_1(
                &format!("✅ Request with payment succeeded: status {}", resp.status).into(),
            );
        }
        Err(e) => {
            web_sys::console::log_1(&format!("❌ Request with payment failed: {}", e).into());
        }
    }

    result
}

/// Poll original API until completion with exponential backoff
/// Supports long-running tasks (up to 10+ minutes)
/// The API will return either pending status or actual data
async fn poll_original_api<T>(
    api_url: &str,
    endpoint: &crate::api::EndpointInfo,
    max_attempts: usize,
    initial_interval_ms: u64,
    on_status_detected: Option<StatusCallback>,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    web_sys::console::log_1(
        &format!(
            "⏳ Polling API endpoint: {} (max {} attempts)",
            endpoint.path, max_attempts
        )
        .into(),
    );

    // Exponential backoff: start with 2s, gradually increase to 15s
    // Formula: min(initial * 2^attempt, max_interval)
    let max_interval_ms = 15000u64; // Max 15 seconds between polls
    let mut current_interval = initial_interval_ms;

    for attempt in 0..max_attempts {
        // Wait before polling (except first attempt)
        if attempt > 0 {
            // Calculate exponential backoff interval
            current_interval = (current_interval * 2).min(max_interval_ms);
            let elapsed_seconds = (attempt as u64 * initial_interval_ms) / 1000;
            let total_seconds = (max_attempts as u64 * initial_interval_ms) / 1000;

            web_sys::console::log_1(
                &format!(
                    "⏸️  Waiting {}s before next poll ({}s / ~{}s elapsed)",
                    current_interval / 1000,
                    elapsed_seconds,
                    total_seconds
                )
                .into(),
            );

            let promise = js_sys::Promise::new(&mut |resolve, _| {
                let window = web_sys::window().unwrap();
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        current_interval as i32,
                    )
                    .unwrap();
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        }

        let elapsed_minutes = (attempt as u64 * initial_interval_ms) / 60000;
        web_sys::console::log_1(
            &format!(
                "📊 Polling attempt {}/{} (~{} min elapsed)",
                attempt + 1,
                max_attempts,
                elapsed_minutes
            )
            .into(),
        );

        match crate::api::make_request(api_url, endpoint, None, None).await {
            Ok(resp) => {
                if resp.status == 200 {
                    // Parse response - could be pending status or actual data
                    match serde_json::from_str::<crate::models::ApiResponse<serde_json::Value>>(
                        &resp.body,
                    ) {
                        Ok(api_resp) => {
                            if api_resp.success {
                                if let Some(data) = &api_resp.data {
                                    // Check if response contains job status
                                    if let Some(status) = data.get("status") {
                                        if let Some(status_str) = status.as_str() {
                                            // Handle different job statuses
                                            match status_str {
                                                "pending" | "processing" => {
                                                    // Still pending or processing, continue polling
                                                    // Call status callback to update UI with current status
                                                    if let Some(ref status_callback) =
                                                        on_status_detected
                                                    {
                                                        let job_key = data
                                                            .get("job_key")
                                                            .and_then(|k| k.as_str())
                                                            .unwrap_or("");
                                                        let message = data
                                                            .get("message")
                                                            .and_then(|m| m.as_str())
                                                            .unwrap_or("Processing in background");
                                                        status_callback(
                                                            status_str.to_string(),
                                                            job_key.to_string(),
                                                            message.to_string(),
                                                        );
                                                    }

                                                    let elapsed_seconds = (attempt as u64
                                                        * initial_interval_ms)
                                                        / 1000;
                                                    let elapsed_minutes = elapsed_seconds / 60;

                                                    // Show progress every 30 seconds or every 5 attempts
                                                    if attempt % 5 == 0
                                                        || elapsed_seconds.is_multiple_of(30)
                                                    {
                                                        let status_msg =
                                                            if status_str == "processing" {
                                                                "⏳ Processing..."
                                                            } else {
                                                                "⏳ Queued..."
                                                            };
                                                        web_sys::console::log_1(
                                                            &format!(
                                                                "{} (~{} min {} sec elapsed)",
                                                                status_msg,
                                                                elapsed_minutes,
                                                                elapsed_seconds % 60
                                                            )
                                                            .into(),
                                                        );

                                                        // If taking longer than 3 minutes, suggest user can refresh later
                                                        if elapsed_minutes >= 3 {
                                                            web_sys::console::log_1(
                                                            &"💡 Tip: This task is taking longer than usual. You can refresh the page later to check results.".into(),
                                                        );
                                                        }
                                                    }
                                                    // Continue polling
                                                }
                                                "completed" => {
                                                    // Job completed - try to get result from response
                                                    web_sys::console::log_1(
                                                        &"✅ Job completed!".into(),
                                                    );
                                                    // Try to deserialize as actual data (result might be in data field)
                                                    match serde_json::from_value::<T>(data.clone())
                                                    {
                                                        Ok(result) => {
                                                            web_sys::console::log_1(
                                                                &"✅ Data received!".into(),
                                                            );
                                                            return Ok(result);
                                                        }
                                                        Err(_) => {
                                                            // If data field doesn't contain result, continue polling to get actual data
                                                            web_sys::console::log_1(&"⏳ Job completed, fetching result...".into());
                                                        }
                                                    }
                                                }
                                                "failed" => {
                                                    // Job failed
                                                    let error_msg = data
                                                        .get("message")
                                                        .and_then(|m| m.as_str())
                                                        .unwrap_or("Analysis failed");
                                                    return Err(format!(
                                                        "Job failed: {}",
                                                        error_msg
                                                    ));
                                                }
                                                _ => {
                                                    // Unknown status, try to deserialize as actual data
                                                }
                                            }

                                            // If we didn't return above, try to deserialize as actual data
                                            if status_str != "pending" && status_str != "processing"
                                            {
                                                // Not pending, try to deserialize as actual data
                                                match serde_json::from_value::<T>(data.clone()) {
                                                    Ok(result) => {
                                                        web_sys::console::log_1(
                                                            &"✅ Data received!".into(),
                                                        );
                                                        return Ok(result);
                                                    }
                                                    Err(e) => {
                                                        web_sys::console::log_1(
                                                            &format!(
                                                                "⚠️ Failed to deserialize data: {}",
                                                                e
                                                            )
                                                            .into(),
                                                        );
                                                    }
                                                }
                                            }
                                        } else {
                                            // No status field, try to deserialize as actual data
                                            match serde_json::from_value::<T>(data.clone()) {
                                                Ok(result) => {
                                                    web_sys::console::log_1(
                                                        &"✅ Data received!".into(),
                                                    );
                                                    return Ok(result);
                                                }
                                                Err(e) => {
                                                    web_sys::console::log_1(
                                                        &format!(
                                                            "⚠️ Failed to deserialize data: {}",
                                                            e
                                                        )
                                                        .into(),
                                                    );
                                                }
                                            }
                                        }
                                    } else {
                                        // No status field, try to deserialize as actual data
                                        match serde_json::from_value::<T>(data.clone()) {
                                            Ok(result) => {
                                                web_sys::console::log_1(
                                                    &"✅ Data received!".into(),
                                                );
                                                return Ok(result);
                                            }
                                            Err(e) => {
                                                web_sys::console::log_1(
                                                    &format!(
                                                        "⚠️ Failed to deserialize data: {}",
                                                        e
                                                    )
                                                    .into(),
                                                );
                                            }
                                        }
                                    }
                                } else {
                                    return Err("No data in response".to_string());
                                }
                            } else {
                                return Err(api_resp
                                    .error
                                    .unwrap_or_else(|| "API request failed".to_string()));
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(
                                &format!("⚠️ Failed to parse response: {}", e).into(),
                            );
                        }
                    }
                } else {
                    web_sys::console::log_1(
                        &format!("⚠️ API request failed: {}", resp.status).into(),
                    );
                }
            }
            Err(e) => {
                web_sys::console::log_1(&format!("⚠️ API request error: {}", e).into());
            }
        }
    }

    let total_elapsed_minutes = (max_attempts as u64 * initial_interval_ms) / 60000;
    Err(format!(
        "Request did not complete within {} attempts (~{} minutes). The task may still be processing. Please try again later.",
        max_attempts, total_elapsed_minutes
    ))
}

/// Make API request with automatic payment handling and pending/polling support
///
/// # Arguments
/// * `on_polling_start` - Optional callback that gets called when polling starts, with the endpoint name
///
/// Note: In WASM, we don't need Send bound since it's single-threaded.
///
/// Callback function type for notifying about job status
pub type StatusCallback = Rc<Box<dyn Fn(String, String, String)>>; // (status, job_key, message) - Fn allows multiple calls, Rc allows cloning

pub async fn make_request_with_payment<T>(
    api_url: &str,
    endpoint: &EndpointInfo,
    body: Option<String>,
    wallet_account: Option<&WalletAccount>,
    on_polling_start: Option<Box<dyn FnOnce()>>,
    on_status_detected: Option<StatusCallback>,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    // First attempt without payment
    match crate::api::make_request(api_url, endpoint, body.clone(), None).await {
        Ok(resp) => {
            // Check if payment is required (402)
            if resp.status == 402 {
                web_sys::console::log_1(
                    &"💳 Received 402 Payment Required, initiating payment flow...".into(),
                );
                // Try to handle payment automatically
                if let Some(account) = wallet_account {
                    if account.is_connected {
                        // Parse payment requirements
                        if let Ok(payment_resp) = serde_json::from_str::<
                            crate::payment::PaymentRequirementsResponse,
                        >(&resp.body)
                        {
                            if let Some(requirements) = payment_resp.accepts.first() {
                                // Attempt payment
                                match handle_payment(requirements, account, api_url, endpoint, body)
                                    .await
                                {
                                    Ok(paid_resp) => {
                                        // Parse successful response
                                        serde_json::from_str::<ApiResponse<T>>(&paid_resp.body)
                                            .map_err(|e| format!("Failed to parse response: {}", e))
                                            .and_then(|api_response| {
                                                if api_response.success {
                                                    api_response.data.ok_or_else(|| {
                                                        api_response.error.unwrap_or_else(|| {
                                                            "No data returned".to_string()
                                                        })
                                                    })
                                                } else {
                                                    Err(api_response.error.unwrap_or_else(|| {
                                                        "Unknown error".to_string()
                                                    }))
                                                }
                                            })
                                    }
                                    Err(e) => Err(format!("Payment failed: {}", e)),
                                }
                            } else {
                                Err("No payment requirements found".to_string())
                            }
                        } else {
                            Err("Failed to parse payment requirements".to_string())
                        }
                    } else {
                        Err("Wallet not connected. Please connect your wallet to access paid features.".to_string())
                    }
                } else {
                    Err(
                        "No wallet connected. Please connect MetaMask to access paid features."
                            .to_string(),
                    )
                }
            } else if resp.status == 200 {
                // Parse response to check for pending status
                match serde_json::from_str::<ApiResponse<serde_json::Value>>(&resp.body) {
                    Ok(api_response) => {
                        if api_response.success {
                            // Check if response contains job status
                            if let Some(data) = &api_response.data {
                                if let Some(status) = data.get("status") {
                                    if let Some(status_str) = status.as_str() {
                                        // Handle different job statuses
                                        match status_str {
                                            "pending" | "processing" => {
                                                let status_msg = if status_str == "processing" {
                                                    "⏳ Request processing, starting background polling..."
                                                } else {
                                                    "⏳ Request pending, starting background polling..."
                                                };
                                                web_sys::console::log_1(&status_msg.into());

                                                // Extract job_key and message from response for status display
                                                let job_key = data
                                                    .get("job_key")
                                                    .and_then(|k| k.as_str())
                                                    .unwrap_or("");
                                                let message = data
                                                    .get("message")
                                                    .and_then(|m| m.as_str())
                                                    .unwrap_or("Processing in background");

                                                // Create a special error message that includes status info
                                                // Format: "JOB_STATUS:{status}:JOB_KEY:{job_key}:MESSAGE:{message}"
                                                // Return this immediately so handlers can update UI right away
                                                let status_error = format!(
                                                    "JOB_STATUS:{}:JOB_KEY:{}:MESSAGE:{}",
                                                    status_str, job_key, message
                                                );

                                                // Notify that polling is starting
                                                if let Some(callback) = on_polling_start {
                                                    callback();
                                                }

                                                // Start background polling to continue checking status
                                                // This will update the UI as status changes
                                                let api_url_clone = api_url.to_string();
                                                let endpoint_clone = endpoint.clone();
                                                let status_callback_for_poll =
                                                    on_status_detected.clone();

                                                wasm_bindgen_futures::spawn_local(async move {
                                                    // Continue polling in the background
                                                    // The polling will call status_callback to update UI
                                                    let _ = poll_original_api::<serde_json::Value>(
                                                        &api_url_clone,
                                                        &endpoint_clone,
                                                        200,
                                                        2000,
                                                        status_callback_for_poll,
                                                    )
                                                    .await;
                                                });

                                                // Return the status error immediately so handlers can update UI
                                                return Err(status_error);
                                            }
                                            "completed" => {
                                                // Job completed - try to get result from response
                                                web_sys::console::log_1(
                                                    &"✅ Job completed!".into(),
                                                );
                                                // Try to deserialize as actual data (result might be in data field)
                                                match serde_json::from_value::<T>(data.clone()) {
                                                    Ok(result) => {
                                                        return Ok(result);
                                                    }
                                                    Err(_) => {
                                                        // If data field doesn't contain result, continue polling to get actual data
                                                        web_sys::console::log_1(
                                                            &"⏳ Job completed, fetching result..."
                                                                .into(),
                                                        );
                                                        if let Some(callback) = on_polling_start {
                                                            callback();
                                                        }
                                                        let status_callback_clone =
                                                            on_status_detected.clone();
                                                        return poll_original_api(
                                                            api_url,
                                                            endpoint,
                                                            200,
                                                            2000,
                                                            status_callback_clone,
                                                        )
                                                        .await;
                                                    }
                                                }
                                            }
                                            "failed" => {
                                                // Job failed
                                                let error_msg = data
                                                    .get("message")
                                                    .and_then(|m| m.as_str())
                                                    .unwrap_or("Analysis failed");
                                                return Err(format!("Job failed: {}", error_msg));
                                            }
                                            "updating" => {
                                                // Fall through to updating handler below
                                            }
                                            _ => {
                                                // Unknown status, try to deserialize as normal response
                                            }
                                        }

                                        // Handle updating status - cache expired, return old data and trigger background update
                                        if status_str == "updating" {
                                            web_sys::console::log_1(
                                                &"🔄 Data is updating in background, returning cached data...".into(),
                                            );

                                            // Extract data from updating response
                                            if let Some(cached_data_value) = data.get("data") {
                                                match serde_json::from_value::<T>(
                                                    cached_data_value.clone(),
                                                ) {
                                                    Ok(cached_data) => {
                                                        web_sys::console::log_1(
                                                            &"✅ Using cached data while update is in progress".into(),
                                                        );

                                                        // Trigger background polling to check for updated data
                                                        // This ensures we get the fresh data once it's ready
                                                        let api_url_clone = api_url.to_string();
                                                        let endpoint_clone = endpoint.clone();
                                                        let status_callback_for_poll =
                                                            on_status_detected.clone();

                                                        wasm_bindgen_futures::spawn_local(
                                                            async move {
                                                                // Poll in background to get updated data
                                                                // The polling will call status_callback to update UI when data is ready
                                                                let _ = poll_original_api::<
                                                                    serde_json::Value,
                                                                >(
                                                                    &api_url_clone,
                                                                    &endpoint_clone,
                                                                    200,
                                                                    2000,
                                                                    status_callback_for_poll,
                                                                )
                                                                .await;
                                                            },
                                                        );

                                                        return Ok(cached_data);
                                                    }
                                                    Err(e) => {
                                                        web_sys::console::log_1(
                                                            &format!("⚠️ Failed to parse cached data: {}", e).into(),
                                                        );
                                                    }
                                                }
                                            }
                                            // If data extraction fails, continue to normal flow
                                        }
                                    }
                                }
                            }
                            // Not pending, try to deserialize as normal response
                            serde_json::from_str::<ApiResponse<T>>(&resp.body)
                                .map_err(|e| format!("Failed to parse response: {}", e))
                                .and_then(|api_response| {
                                    if api_response.success {
                                        api_response.data.ok_or_else(|| {
                                            api_response
                                                .error
                                                .unwrap_or_else(|| "No data returned".to_string())
                                        })
                                    } else {
                                        Err(api_response
                                            .error
                                            .unwrap_or_else(|| "Unknown error".to_string()))
                                    }
                                })
                        } else {
                            // Check if error message indicates pending
                            if let Some(error_msg) = &api_response.error {
                                if error_msg.contains("pending")
                                    || error_msg.contains("in progress")
                                {
                                    web_sys::console::log_1(
                                        &"⏳ Request pending (from error), starting background polling...".into(),
                                    );
                                    // Notify that polling is starting
                                    if let Some(callback) = on_polling_start {
                                        callback();
                                    }
                                    // Start polling the original API with exponential backoff
                                    return poll_original_api(api_url, endpoint, 200, 2000, None)
                                        .await;
                                }
                            }
                            Err(api_response
                                .error
                                .unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    }
                    Err(e) => Err(format!("Failed to parse response: {}", e)),
                }
            } else {
                // Other status codes
                Err(format!("Request failed with status: {}", resp.status))
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

/// Create profile endpoint info
pub fn create_profile_endpoint(search_query: &str, is_fid: bool) -> EndpointInfo {
    EndpointInfo {
        path: if is_fid {
            format!("/api/profiles/{}", search_query)
        } else {
            format!("/api/profiles/username/{}", search_query)
        },
        method: "GET".to_string(),
        name: "Get Profile".to_string(),
        description: "Get user profile".to_string(),
        tier: "Basic".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create social data endpoint info
pub fn create_social_endpoint(search_query: &str, is_fid: bool) -> EndpointInfo {
    EndpointInfo {
        path: if is_fid {
            format!("/api/social/{}", search_query)
        } else {
            format!("/api/social/username/{}", search_query)
        },
        method: "GET".to_string(),
        name: "Get Social Data".to_string(),
        description: "Get social analysis".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create chat session endpoint info
pub fn create_chat_session_endpoint() -> EndpointInfo {
    EndpointInfo {
        path: "/api/chat/create".to_string(),
        method: "POST".to_string(),
        name: "Create Chat".to_string(),
        description: "Create chat session".to_string(),
        tier: "Premium".to_string(),
        requires_payment: false,
        default_body: None,
    }
}

/// Create chat message endpoint info
pub fn create_chat_message_endpoint() -> EndpointInfo {
    EndpointInfo {
        path: "/api/chat/message".to_string(),
        method: "POST".to_string(),
        name: "Send Chat Message".to_string(),
        description: "Send chat message".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create MBTI endpoint info
pub fn create_mbti_endpoint(search_query: &str, is_fid: bool) -> EndpointInfo {
    EndpointInfo {
        path: if is_fid {
            format!("/api/mbti/{}", search_query)
        } else {
            format!("/api/mbti/username/{}", search_query)
        },
        method: "GET".to_string(),
        name: "Get MBTI Analysis".to_string(),
        description: "Get MBTI personality analysis".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Update URL path using History API (supports browser back/forward)
/// Format: /profile/{query}, /chat/{query}, or /annual-report/{fid}
pub fn update_url_path(query: &str, view: &str) {
    let window = web_sys::window().unwrap();
    let history = window.history().unwrap();
    let path = if view == "chat" {
        format!("/chat/{}", query)
    } else if view == "annual-report" {
        format!("/annual-report/{}", query)
    } else {
        format!("/profile/{}", query)
    };

    // Use pushState to update URL without page reload
    // This adds a new entry to browser history (supports back/forward)
    let state = js_sys::Object::new();
    let _ = history.push_state_with_url(&state, "", Some(&path));

    web_sys::console::log_1(&format!("📍 Updated URL path: {}", path).into());
}

/// Update URL to annual report path
pub fn update_annual_report_url(fid: i64) {
    update_url_path(&fid.to_string(), "annual-report");
}

/// Clear URL path (return to home)
pub fn clear_url_path() {
    let window = web_sys::window().unwrap();
    let history = window.history().unwrap();
    let state = js_sys::Object::new();

    // Use pushState to update URL to root
    let _ = history.push_state_with_url(&state, "", Some("/"));

    web_sys::console::log_1(&"📍 Cleared URL path (returned to home)".into());
}

/// Get current URL path and parse it
/// Returns (query, view) where view is "profile", "chat", or "annual-report"
/// For annual-report, query is the FID
pub fn get_url_path() -> Option<(String, String)> {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let pathname = location.pathname().ok()?;

    if pathname.is_empty() || pathname == "/" {
        return None;
    }

    // Parse format: /profile/{query}, /chat/{query}, or /annual-report/{fid}
    if let Some(path) = pathname.strip_prefix("/") {
        if let Some((view, query)) = path.split_once('/') {
            if view == "profile" || view == "chat" || view == "annual-report" {
                return Some((query.to_string(), view.to_string()));
            }
        }
    }

    None
}

/// Set up popstate event listener for browser back/forward navigation
/// This callback will be called when user clicks browser back/forward buttons
pub fn setup_popstate_listener(callback: impl Fn(Option<(String, String)>) + 'static) {
    let window = web_sys::window().unwrap();
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_event: web_sys::Event| {
        let path = get_url_path();
        web_sys::console::log_1(&format!("🔙 Browser navigation detected: {:?}", path).into());
        callback(path);
    }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref())
        .unwrap();

    // Keep the closure alive for the lifetime of the app
    closure.forget();
}

/// Create annual report endpoint info
pub fn create_annual_report_endpoint(fid: i64, year: i32) -> EndpointInfo {
    EndpointInfo {
        path: format!("/api/users/{}/annual-report/{}", fid, year),
        method: "GET".to_string(),
        name: "Get Annual Report".to_string(),
        description: format!("Get annual report for {}", year),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create engagement endpoint info
pub fn create_engagement_endpoint(fid: i64, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = format!("/api/users/{}/engagement", fid);
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Engagement".to_string(),
        description: "Get user engagement metrics".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create temporal activity endpoint info
pub fn create_temporal_activity_endpoint(fid: i64, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = format!("/api/users/{}/activity/temporal", fid);
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Temporal Activity".to_string(),
        description: "Get temporal activity analysis".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create content style endpoint info
pub fn create_content_style_endpoint(fid: i64, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = format!("/api/users/{}/content/style", fid);
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Content Style".to_string(),
        description: "Get content style analysis".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create follower growth endpoint info
pub fn create_follower_growth_endpoint(fid: i64, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = format!("/api/users/{}/followers/growth", fid);
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Follower Growth".to_string(),
        description: "Get follower growth metrics".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create domains endpoint info
pub fn create_domains_endpoint(fid: i64) -> EndpointInfo {
    EndpointInfo {
        path: format!("/api/users/{}/domains", fid),
        method: "GET".to_string(),
        name: "Get Domains".to_string(),
        description: "Get domain and username status".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Create network averages endpoint info
pub fn create_network_averages_endpoint(start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = "/api/stats/network/averages".to_string();
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Network Averages".to_string(),
        description: "Get network-wide statistics".to_string(),
        tier: "Basic".to_string(),
        requires_payment: false,
        default_body: None,
    }
}

/// Create casts stats endpoint info
pub fn create_casts_stats_endpoint(fid: i64, start_timestamp: Option<i64>, end_timestamp: Option<i64>) -> EndpointInfo {
    let mut path = format!("/api/casts/stats/{}", fid);
    let mut query_parts = Vec::new();
    
    if let Some(start) = start_timestamp {
        query_parts.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_parts.push(format!("end_timestamp={}", end));
    }
    
    if !query_parts.is_empty() {
        path.push_str("?");
        path.push_str(&query_parts.join("&"));
    }
    
    EndpointInfo {
        path,
        method: "GET".to_string(),
        name: "Get Casts Stats".to_string(),
        description: "Get detailed cast statistics".to_string(),
        tier: "Premium".to_string(),
        requires_payment: true,
        default_body: None,
    }
}

/// Get 2025 year timestamps (start and end)
pub fn get_2025_timestamps() -> (i64, i64) {
    // 2025-01-01 00:00:00 UTC
    let start = 1735689600;
    // 2025-12-31 23:59:59 UTC
    let end = 1767225600;
    (start, end)
}

/// Get 2024 year timestamps (start and end) for comparison
pub fn get_2024_timestamps() -> (i64, i64) {
    // 2024-01-01 00:00:00 UTC
    let start = 1704067200;
    // 2024-12-31 23:59:59 UTC
    let end = 1735689600;
    (start, end)
}
