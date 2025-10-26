use crate::models::*;
use crate::api::{ApiResponse as RawApiResponse, EndpointInfo};
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

    // Create EIP-712 typed data
    let typed_data = crate::payment::create_eip712_typed_data(requirements, payer, &nonce, timestamp)?;
    
    // Debug: Log the typed data string
    web_sys::console::log_1(&format!("Typed data string: {}", typed_data).into());
    
    // Validate JSON parsing
    let _parsed_check: serde_json::Value = serde_json::from_str(&typed_data)
        .map_err(|e| format!("Failed to parse typed data as JSON: {}", e))?;
    web_sys::console::log_1(&format!("Typed data validation successful").into());

    // Sign with MetaMask
    let signature = crate::wallet::sign_eip712(&typed_data)
        .await
        .map_err(|e| format!("Failed to sign payment: {}", e))?;

    // Create payment payload
    let payment_payload =
        crate::payment::create_payment_payload(requirements, payer, &signature, &nonce, timestamp);

    // Debug: Log payment payload as JSON
    let payment_json = serde_json::to_string(&payment_payload)
        .unwrap_or_else(|_| "Failed to serialize payment payload".to_string());
    web_sys::console::log_1(&format!("Payment payload JSON: {}", payment_json).into());

    // Encode to base64
    let payment_header = payment_payload
        .to_base64()
        .map_err(|e| format!("Failed to encode payment: {}", e))?;

    // Debug: Log payment header
    web_sys::console::log_1(&format!("Payment header: {}", payment_header).into());

    // Retry request with payment
    crate::api::make_request(api_url, endpoint, body, Some(payment_header))
        .await
        .map_err(|e| format!("Request with payment failed: {}", e))
}

/// Make API request with automatic payment handling
pub async fn make_request_with_payment<T>(
    api_url: &str,
    endpoint: &EndpointInfo,
    body: Option<String>,
    wallet_account: Option<&WalletAccount>,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    // First attempt without payment
    match crate::api::make_request(api_url, endpoint, body.clone(), None).await {
        Ok(resp) => {
            // Check if payment is required (402)
            if resp.status == 402 {
                // Try to handle payment automatically
                if let Some(account) = wallet_account {
                    if account.is_connected {
                        // Parse payment requirements
                        if let Ok(payment_resp) = serde_json::from_str::<crate::payment::PaymentRequirementsResponse>(&resp.body) {
                            if let Some(requirements) = payment_resp.accepts.first() {
                                // Attempt payment
                                match handle_payment(requirements, account, api_url, endpoint, body).await {
                                    Ok(paid_resp) => {
                                        // Parse successful response
                                        serde_json::from_str::<ApiResponse<T>>(&paid_resp.body)
                                            .map_err(|e| format!("Failed to parse response: {}", e))
                                            .and_then(|api_response| {
                                                if api_response.success {
                                                    api_response.data.ok_or_else(|| api_response.error.unwrap_or_else(|| "No data returned".to_string()))
                                                } else {
                                                    Err(api_response.error.unwrap_or_else(|| "Unknown error".to_string()))
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
                    Err("No wallet connected. Please connect MetaMask to access paid features.".to_string())
                }
            } else {
                // Parse successful response
                serde_json::from_str::<ApiResponse<T>>(&resp.body)
                    .map_err(|e| format!("Failed to parse response: {}", e))
                    .and_then(|api_response| {
                        if api_response.success {
                            api_response.data.ok_or_else(|| api_response.error.unwrap_or_else(|| "No data returned".to_string()))
                        } else {
                            Err(api_response.error.unwrap_or_else(|| "Unknown error".to_string()))
                        }
                    })
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
