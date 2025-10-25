use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;
use web_sys::InputEvent;

mod api;
mod payment;
mod wallet;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileData {
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    pfp_url: Option<String>,
    location: Option<String>,
    twitter_username: Option<String>,
    github_username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SocialData {
    fid: i64,
    following_count: usize,
    followers_count: usize,
    influence_score: f32,
    top_followed_users: Vec<UserMention>,
    top_followers: Vec<UserMention>,
    most_mentioned_users: Vec<UserMention>,
    social_circles: SocialCircles,
    interaction_style: InteractionStyle,
    word_cloud: WordCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SocialCircles {
    tech_builders: f32,
    content_creators: f32,
    web3_natives: f32,
    casual_users: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionStyle {
    reply_frequency: f32,
    mention_frequency: f32,
    network_connector: bool,
    community_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserMention {
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    count: usize,
    category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordCloud {
    top_words: Vec<WordFrequency>,
    top_phrases: Vec<WordFrequency>,
    signature_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordFrequency {
    word: String,
    count: usize,
    percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    profile: ProfileData,
    social: Option<SocialData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatSession {
    session_id: String,
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    conversation_history: Vec<ChatMessage>,
    created_at: u64,
    last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateChatRequest {
    user: String,
    context_limit: usize,
    temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateChatResponse {
    session_id: String,
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    total_casts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessageRequest {
    session_id: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessageResponse {
    session_id: String,
    message: String,
    relevant_casts_count: usize,
    conversation_length: usize,
}

/// Handle payment flow: sign and retry request with payment header
async fn handle_payment(
    requirements: &payment::PaymentRequirements,
    account: &wallet::WalletAccount,
    api_url: &str,
    endpoint: &api::EndpointInfo,
    body: Option<String>,
) -> Result<api::ApiResponse, String> {
    // Generate nonce and timestamp
    let nonce = payment::generate_nonce();
    let timestamp = payment::get_timestamp();

    // Get payer address
    let payer = account
        .address
        .as_ref()
        .ok_or("No wallet address available")?;

    // Create EIP-712 typed data
    let typed_data = payment::create_eip712_typed_data(requirements, payer, &nonce, timestamp)?;

    // Sign with MetaMask
    let signature = wallet::sign_eip712(&typed_data)
        .await
        .map_err(|e| format!("Failed to sign payment: {}", e))?;

    // Create payment payload
    let payment_payload =
        payment::create_payment_payload(requirements, payer, &signature, &nonce, timestamp);

    // Encode to base64
    let payment_header = payment_payload
        .to_base64()
        .map_err(|e| format!("Failed to encode payment: {}", e))?;

    // Retry request with payment
    api::make_request(api_url, endpoint, body, Some(payment_header))
        .await
        .map_err(|e| format!("Request with payment failed: {}", e))
}

#[function_component]
fn App() -> Html {
    // Wallet state
    let wallet_account = use_state(|| None::<wallet::WalletAccount>);
    let wallet_initialized = use_state(|| false);
    let wallet_error = use_state(|| None::<String>);

    // State management
    let search_input = use_state(|| String::new());
    let search_result = use_state(|| None::<SearchResult>);
    let is_loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let api_url = use_state(|| "http://127.0.0.1:3000".to_string());
    
    // Chat state management
    let chat_session = use_state(|| None::<ChatSession>);
    let chat_message = use_state(|| String::new());
    let chat_messages = use_state(|| Vec::<ChatMessage>::new());
    let is_chat_loading = use_state(|| false);
    let chat_error = use_state(|| None::<String>);
    let current_view = use_state(|| "profile"); // "profile" or "chat"

    // Initialize wallet on mount
    {
        let wallet_initialized = wallet_initialized.clone();
        let wallet_account = wallet_account.clone();
        let wallet_error = wallet_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match wallet::initialize().await {
                    Ok(_) => {
                        wallet_initialized.set(true);
                        if let Ok(account) = wallet::get_account().await {
                            wallet_account.set(Some(account));
                        }
                    }
                    Err(e) => {
                        wallet_initialized.set(true); // Set initialized even on error
                        wallet_error.set(Some(e));
                    }
                }
            });
            || ()
        });
    }

    // Poll wallet account state
    {
        let wallet_account = wallet_account.clone();

        use_effect_with((), move |_| {
            let interval = gloo_timers::callback::Interval::new(1000, move || {
                let wallet_account = wallet_account.clone();
                spawn_local(async move {
                    if let Ok(account) = wallet::get_account().await {
                        wallet_account.set(Some(account));
                    }
                });
            });

            move || drop(interval)
        });
    }

    // Wallet handlers
    let on_connect_wallet = {
        let wallet_error = wallet_error.clone();
        let wallet_account = wallet_account.clone();
        
        Callback::from(move |_| {
            let wallet_error = wallet_error.clone();
            let wallet_account = wallet_account.clone();
            spawn_local(async move {
                match wallet::connect().await {
                    Ok(_) => {
                        wallet_error.set(None);
                        if let Ok(account) = wallet::get_account().await {
                            wallet_account.set(Some(account));
                        }
                    }
                    Err(e) => {
                        wallet_error.set(Some(e));
                    }
                }
            });
        })
    };

    let on_disconnect_wallet = {
        let wallet_account = wallet_account.clone();
        
        Callback::from(move |_| {
            let wallet_account = wallet_account.clone();
            spawn_local(async move {
                let _ = wallet::disconnect().await;
                wallet_account.set(None);
            });
        })
    };

    // Search handler
    let on_search = {
        let search_input = search_input.clone();
        let search_result = search_result.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        let api_url = api_url.clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();
        let wallet_account = wallet_account.clone();

        Callback::from(move |_| {
            let input = (*search_input).clone();
            if input.trim().is_empty() {
                return;
            }

            let search_result = search_result.clone();
            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            let api_url = (*api_url).clone();
            let chat_session = chat_session.clone();
            let chat_messages = chat_messages.clone();
            let is_chat_loading = is_chat_loading.clone();
            let chat_error = chat_error.clone();
            let wallet_account = wallet_account.clone();

            spawn_local(async move {
                is_loading.set(true);
                error_message.set(None);

                // Determine if input is FID (numeric) or username (text)
                let trimmed_input = input.trim();
                let is_fid = trimmed_input.parse::<u64>().is_ok();
                
                // Handle username with @ prefix
                let search_query = if is_fid {
                    trimmed_input.to_string()
                } else {
                    // Remove @ prefix if present for username search
                    trimmed_input.trim_start_matches('@').to_string()
                };

                // Create endpoint info for profile request
                let profile_endpoint = api::EndpointInfo {
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
                };

                // Make profile request with payment support
                match api::make_request(&api_url, &profile_endpoint, None, None).await {
                    Ok(resp) => {
                        // Check if payment is required (402)
                        if resp.status == 402 {
                            // Try to handle payment automatically
                            if let Some(account) = (*wallet_account).clone() {
                                if account.is_connected {
                                    // Parse payment requirements
                                    if let Ok(payment_resp) = serde_json::from_str::<payment::PaymentRequirementsResponse>(&resp.body) {
                                        if let Some(requirements) = payment_resp.accepts.first() {
                                            // Attempt payment
                                            match handle_payment(requirements, &account, &api_url, &profile_endpoint, None).await {
                                                Ok(paid_resp) => {
                                                    // Parse successful response
                                                    if let Ok(api_response) = serde_json::from_str::<ApiResponse<ProfileData>>(&paid_resp.body) {
                                                        if api_response.success && api_response.data.is_some() {
                                                            let profile = api_response.data.unwrap();
                                                            
                                                            // Now get social data
                                                            let social_endpoint = api::EndpointInfo {
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
                                                            };

                                                            let social_data = match api::make_request(&api_url, &social_endpoint, None, None).await {
                                                                Ok(social_resp) => {
                                                                    if social_resp.status == 402 {
                                                                        // Try payment for social data too
                                                                        if let Ok(social_payment_resp) = serde_json::from_str::<payment::PaymentRequirementsResponse>(&social_resp.body) {
                                                                            if let Some(social_requirements) = social_payment_resp.accepts.first() {
                                                                                match handle_payment(social_requirements, &account, &api_url, &social_endpoint, None).await {
                                                                                    Ok(social_paid_resp) => {
                                                                                        if let Ok(social_api_response) = serde_json::from_str::<ApiResponse<SocialData>>(&social_paid_resp.body) {
                                                                                            social_api_response.data
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                                    } else {
                                        None
                                    }
                                                                    } else if let Ok(social_api_response) = serde_json::from_str::<ApiResponse<SocialData>>(&social_resp.body) {
                                                                        social_api_response.data
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                };

                    search_result.set(Some(SearchResult {
                        profile,
                        social: social_data,
                    }));
                    error_message.set(None);

                    // Auto-create chat session after successful search
                    let chat_session = chat_session.clone();
                    let chat_messages = chat_messages.clone();
                    let is_chat_loading = is_chat_loading.clone();
                    let chat_error = chat_error.clone();
                    let api_url = api_url.clone();
                                                            let wallet_account = wallet_account.clone();

                    spawn_local(async move {
                        is_chat_loading.set(true);
                        chat_error.set(None);

                        // Create chat session
                        let request = CreateChatRequest {
                            user: if is_fid {
                                search_query.clone()
                            } else {
                                format!("@{}", search_query)
                            },
                            context_limit: 20,
                            temperature: 0.7,
                        };

                        let request_json = serde_json::to_string(&request).unwrap();
                                                                let chat_endpoint = api::EndpointInfo {
                                                                    path: "/api/chat/create".to_string(),
                                                                    method: "POST".to_string(),
                                                                    name: "Create Chat".to_string(),
                                                                    description: "Create chat session".to_string(),
                                                                    tier: "Premium".to_string(),
                                                                    requires_payment: false,
                                                                    default_body: None,
                                                                };

                                                                match api::make_request(&api_url, &chat_endpoint, Some(request_json), None).await {
                                                                    Ok(chat_resp) => {
                                                                        if chat_resp.status == 402 {
                                                                            // Try payment for chat creation
                                                                            if let Some(account) = (*wallet_account).clone() {
                                                                                if account.is_connected {
                                                                                    if let Ok(chat_payment_resp) = serde_json::from_str::<payment::PaymentRequirementsResponse>(&chat_resp.body) {
                                                                                        if let Some(chat_requirements) = chat_payment_resp.accepts.first() {
                                                                                            let request_json = serde_json::to_string(&request).unwrap();
                                                                                            match handle_payment(chat_requirements, &account, &api_url, &chat_endpoint, Some(request_json)).await {
                                                                                                Ok(chat_paid_resp) => {
                                                                                                    if let Ok(api_response) = serde_json::from_str::<ApiResponse<CreateChatResponse>>(&chat_paid_resp.body) {
                                            if api_response.success && api_response.data.is_some() {
                                                let chat_data = api_response.data.unwrap();
                                                let session = ChatSession {
                                                    session_id: chat_data.session_id,
                                                    fid: chat_data.fid,
                                                    username: chat_data.username,
                                                    display_name: chat_data.display_name,
                                                    conversation_history: Vec::new(),
                                                    created_at: 0,
                                                    last_activity: 0,
                                                };
                                                chat_session.set(Some(session));
                                                chat_messages.set(Vec::new());
                                                chat_error.set(None);
                } else {
                                                chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                                                                        }
                                                                                                    } else {
                                                                                                        chat_error.set(Some("Failed to parse chat response".to_string()));
                                            }
                                        }
                                        Err(e) => {
                                                                                                    chat_error.set(Some(format!("Chat payment failed: {}", e)));
                                        }
                                    }
                                } else {
                                                                                            chat_error.set(Some("No payment requirements for chat".to_string()));
                                                                                        }
                                                                                    } else {
                                                                                        chat_error.set(Some("Failed to parse chat payment requirements".to_string()));
                                                                                    }
                                                                                } else {
                                                                                    chat_error.set(Some("Wallet not connected for chat".to_string()));
                                                                                }
                                                                            } else {
                                                                                chat_error.set(Some("No wallet for chat".to_string()));
                                                                            }
                                                                        } else if let Ok(api_response) = serde_json::from_str::<ApiResponse<CreateChatResponse>>(&chat_resp.body) {
                                                                            if api_response.success && api_response.data.is_some() {
                                                                                let chat_data = api_response.data.unwrap();
                                                                                let session = ChatSession {
                                                                                    session_id: chat_data.session_id,
                                                                                    fid: chat_data.fid,
                                                                                    username: chat_data.username,
                                                                                    display_name: chat_data.display_name,
                                                                                    conversation_history: Vec::new(),
                                                                                    created_at: 0,
                                                                                    last_activity: 0,
                                                                                };
                                                                                chat_session.set(Some(session));
                                                                                chat_messages.set(Vec::new());
                                                                                chat_error.set(None);
                                                                            } else {
                                                                                chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                                            }
                                                                        } else {
                                                                            chat_error.set(Some("Failed to parse chat response".to_string()));
                        }
                    }
                    Err(e) => {
                                                                        chat_error.set(Some(format!("Chat request failed: {}", e)));
                            }
                    }

                        is_chat_loading.set(false);
                    });
                } else {
                                                            error_message.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                        }
                                                    } else {
                                                        error_message.set(Some("Failed to parse profile response".to_string()));
                                                    }
                                                }
                                                Err(e) => {
                                                    error_message.set(Some(format!("Payment failed: {}", e)));
                                                }
                                            }
                                        } else {
                                            error_message.set(Some("No payment requirements found".to_string()));
                                        }
                                    } else {
                                        error_message.set(Some("Failed to parse payment requirements".to_string()));
                                    }
                                } else {
                                    error_message.set(Some("Wallet not connected. Please connect your wallet to access paid features.".to_string()));
                                }
                            } else {
                                error_message.set(Some("No wallet connected. Please connect MetaMask to access paid features.".to_string()));
                            }
                        } else if let Ok(api_response) = serde_json::from_str::<ApiResponse<ProfileData>>(&resp.body) {
                            if api_response.success && api_response.data.is_some() {
                                let profile = api_response.data.unwrap();
                                
                                // Get social data (free)
                                let social_endpoint = api::EndpointInfo {
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
                                };

                                let social_data = match api::make_request(&api_url, &social_endpoint, None, None).await {
                                    Ok(social_resp) => {
                                        if social_resp.status == 402 {
                                            // Social data requires payment, skip for now
                                            None
                                        } else if let Ok(social_api_response) = serde_json::from_str::<ApiResponse<SocialData>>(&social_resp.body) {
                                            social_api_response.data
                                        } else {
                                            None
                                        }
                                    }
                                    Err(_) => None,
                                };

                                search_result.set(Some(SearchResult {
                                    profile,
                                    social: social_data,
                                }));
                                error_message.set(None);

                                // Auto-create chat session after successful search (non-payment branch)
                                let chat_session = chat_session.clone();
                                let chat_messages = chat_messages.clone();
                                let is_chat_loading = is_chat_loading.clone();
                                let chat_error = chat_error.clone();
                                let api_url = api_url.clone();
                                let wallet_account = wallet_account.clone();

                                spawn_local(async move {
                                    is_chat_loading.set(true);
                                    chat_error.set(None);

                                    // Create chat session
                                    let request = CreateChatRequest {
                                        user: if is_fid {
                                            search_query.clone()
                                        } else {
                                            format!("@{}", search_query)
                                        },
                                        context_limit: 20,
                                        temperature: 0.7,
                                    };

                                    let request_json = serde_json::to_string(&request).unwrap();
                                    let chat_endpoint = api::EndpointInfo {
                                        path: "/api/chat/create".to_string(),
                                        method: "POST".to_string(),
                                        name: "Create Chat".to_string(),
                                        description: "Create chat session".to_string(),
                                        tier: "Premium".to_string(),
                                        requires_payment: false,
                                        default_body: None,
                                    };

                                    match api::make_request(&api_url, &chat_endpoint, Some(request_json), None).await {
                                        Ok(chat_resp) => {
                                            if chat_resp.status == 402 {
                                                // Try payment for chat creation
                                                if let Some(account) = (*wallet_account).clone() {
                                                    if account.is_connected {
                                                        if let Ok(chat_payment_resp) = serde_json::from_str::<payment::PaymentRequirementsResponse>(&chat_resp.body) {
                                                            if let Some(chat_requirements) = chat_payment_resp.accepts.first() {
                                                                let request_json = serde_json::to_string(&request).unwrap();
                                                                match handle_payment(chat_requirements, &account, &api_url, &chat_endpoint, Some(request_json)).await {
                                                                    Ok(chat_paid_resp) => {
                                                                        if let Ok(api_response) = serde_json::from_str::<ApiResponse<CreateChatResponse>>(&chat_paid_resp.body) {
                                                                            if api_response.success && api_response.data.is_some() {
                                                                                let chat_data = api_response.data.unwrap();
                                                                                let session = ChatSession {
                                                                                    session_id: chat_data.session_id,
                                                                                    fid: chat_data.fid,
                                                                                    username: chat_data.username,
                                                                                    display_name: chat_data.display_name,
                                                                                    conversation_history: Vec::new(),
                                                                                    created_at: 0,
                                                                                    last_activity: 0,
                                                                                };
                                                                                chat_session.set(Some(session));
                                                                                chat_messages.set(Vec::new());
                                                                                chat_error.set(None);
                                                                            } else {
                                                                                chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                                            }
                                                                        } else {
                                                                            chat_error.set(Some("Failed to parse chat response".to_string()));
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        chat_error.set(Some(format!("Chat payment failed: {}", e)));
                                                                    }
                                                                }
                                                            } else {
                                                                chat_error.set(Some("No payment requirements for chat".to_string()));
                                                            }
                                                        } else {
                                                            chat_error.set(Some("Failed to parse chat payment requirements".to_string()));
                                                        }
                                                    } else {
                                                        chat_error.set(Some("Wallet not connected for chat".to_string()));
                                                    }
                                                } else {
                                                    chat_error.set(Some("No wallet for chat".to_string()));
                                                }
                                            } else if let Ok(api_response) = serde_json::from_str::<ApiResponse<CreateChatResponse>>(&chat_resp.body) {
                                                if api_response.success && api_response.data.is_some() {
                                                    let chat_data = api_response.data.unwrap();
                                                    let session = ChatSession {
                                                        session_id: chat_data.session_id,
                                                        fid: chat_data.fid,
                                                        username: chat_data.username,
                                                        display_name: chat_data.display_name,
                                                        conversation_history: Vec::new(),
                                                        created_at: 0,
                                                        last_activity: 0,
                                                    };
                                                    chat_session.set(Some(session));
                                                    chat_messages.set(Vec::new());
                                                    chat_error.set(None);
                                                } else {
                                                    chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                }
                                            } else {
                                                chat_error.set(Some("Failed to parse chat response".to_string()));
                                            }
                                        }
                                        Err(e) => {
                                            chat_error.set(Some(format!("Chat request failed: {}", e)));
                                        }
                                    }
                                    is_chat_loading.set(false);
                                });
                            } else {
                                error_message.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                            }
                        } else {
                            error_message.set(Some("Failed to parse profile response".to_string()));
                        }
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Request failed: {}", e)));
                    }
                }

                is_loading.set(false);
            });
        })
    };

    // Enter key handler for search
    let on_keypress = {
        let on_search = on_search.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_search.emit(());
            }
        })
    };


    // Popular FID handler
    let on_popular_fid = {
        let search_input = search_input.clone();
        let on_search = on_search.clone();
        Callback::from(move |fid: String| {
            search_input.set(fid.clone());
            on_search.emit(());
        })
    };

    // View switching handler
    let on_switch_to_chat = {
        let current_view = current_view.clone();
        Callback::from(move |_| {
            current_view.set("chat");
        })
    };


    // Smart back button handler
    let on_smart_back = {
        let current_view = current_view.clone();
        let search_result = search_result.clone();
        let search_input = search_input.clone();
        let error_message = error_message.clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        Callback::from(move |_| {
            match *current_view {
                "profile" => {
                    // From profile back to search
                    search_result.set(None);
                    search_input.set(String::new());
                    error_message.set(None);
                    chat_session.set(None);
                    chat_messages.set(Vec::new());
                },
                "chat" => {
                    // From chat back to profile
                    current_view.set("profile");
                },
                _ => {
                    // Default: back to search
                    search_result.set(None);
                    search_input.set(String::new());
                    error_message.set(None);
                    chat_session.set(None);
                    chat_messages.set(Vec::new());
                }
            }
        })
    };

    // Chat message handler
    let on_send_chat_message = {
        let chat_session = chat_session.clone();
        let chat_message = chat_message.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();
        let api_url = api_url.clone();
        let wallet_account = wallet_account.clone();

        Callback::from(move |_| {
            let message = (*chat_message).clone();
            if message.trim().is_empty() {
                return;
            }

            if let Some(session) = (*chat_session).clone() {
                let chat_message = chat_message.clone();
                let chat_messages = chat_messages.clone();
                let is_chat_loading = is_chat_loading.clone();
                let chat_error = chat_error.clone();
            let api_url = (*api_url).clone();
                let wallet_account = wallet_account.clone();

            spawn_local(async move {
                    is_chat_loading.set(true);
                    chat_error.set(None);

                    // Add user message to chat
                    let user_message = ChatMessage {
                        role: "user".to_string(),
                        content: message.clone(),
                        timestamp: 0,
                    };
                    let mut messages = (*chat_messages).clone();
                    messages.push(user_message);
                    chat_messages.set(messages);

                    // Send message to API
                    let request = ChatMessageRequest {
                        session_id: session.session_id.clone(),
                        message: message.clone(),
                    };

                    let request_json = serde_json::to_string(&request).unwrap();
                    let chat_endpoint = api::EndpointInfo {
                        path: "/api/chat/message".to_string(),
                        method: "POST".to_string(),
                        name: "Send Chat Message".to_string(),
                        description: "Send chat message".to_string(),
                        tier: "Premium".to_string(),
                        requires_payment: true,
                        default_body: None,
                    };

                    match api::make_request(&api_url, &chat_endpoint, Some(request_json), None).await {
                        Ok(resp) => {
                            if resp.status == 402 {
                                // Try payment for chat message
                                if let Some(account) = (*wallet_account).clone() {
                                    if account.is_connected {
                                        if let Ok(chat_payment_resp) = serde_json::from_str::<payment::PaymentRequirementsResponse>(&resp.body) {
                                            if let Some(chat_requirements) = chat_payment_resp.accepts.first() {
                                                let request_json = serde_json::to_string(&request).unwrap();
                                                match handle_payment(chat_requirements, &account, &api_url, &chat_endpoint, Some(request_json)).await {
                                                    Ok(chat_paid_resp) => {
                                                        if let Ok(api_response) = serde_json::from_str::<ApiResponse<ChatMessageResponse>>(&chat_paid_resp.body) {
                                        if api_response.success && api_response.data.is_some() {
                                            let chat_data = api_response.data.unwrap();
                                            let assistant_message = ChatMessage {
                                                role: "assistant".to_string(),
                                                content: chat_data.message,
                                                timestamp: 0,
                                            };
                                            let mut messages = (*chat_messages).clone();
                                            messages.push(assistant_message);
                                            chat_messages.set(messages);
                                            chat_error.set(None);
                                        } else {
                                            chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                                            }
                                                        } else {
                                                            chat_error.set(Some("Failed to parse chat response".to_string()));
                                        }
                                    }
                                    Err(e) => {
                                                        chat_error.set(Some(format!("Chat payment failed: {}", e)));
                                    }
                                }
                            } else {
                                                chat_error.set(Some("No payment requirements for chat message".to_string()));
                                            }
                                        } else {
                                            chat_error.set(Some("Failed to parse chat payment requirements".to_string()));
                                        }
                                    } else {
                                        chat_error.set(Some("Wallet not connected for chat message".to_string()));
                                    }
                                } else {
                                    chat_error.set(Some("No wallet for chat message".to_string()));
                                }
                            } else if let Ok(api_response) = serde_json::from_str::<ApiResponse<ChatMessageResponse>>(&resp.body) {
                                if api_response.success && api_response.data.is_some() {
                                    let chat_data = api_response.data.unwrap();
                                    let assistant_message = ChatMessage {
                                        role: "assistant".to_string(),
                                        content: chat_data.message,
                                        timestamp: 0,
                                    };
                                    let mut messages = (*chat_messages).clone();
                                    messages.push(assistant_message);
                                    chat_messages.set(messages);
                                    chat_error.set(None);
                                } else {
                                    chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                }
                            } else {
                                chat_error.set(Some("Failed to parse chat response".to_string()));
                            }
                        }
                        Err(e) => {
                            chat_error.set(Some(format!("Chat request failed: {}", e)));
                        }
                    }

                    chat_message.set(String::new());
                    is_chat_loading.set(false);
                });
            }
        })
    };

    html! {
        <div class="app-container">
            // Search Page (only show if no search results)
            if (*search_result).is_none() {
                <div class="search-page">
                    <div class="search-header">
                        <div class="logo">
                            // Logo Image
                            <div class="logo-image">
                                <img src="/logo.png" alt="Polyjuice Logo" />
                            </div>
                            <h1>{"polyjuice"}</h1>
                            <p class="tagline">{"Discover & Chat with Farcaster Users"}</p>
                        </div>
                        
                        // Wallet Status - Only show if wallet is available
                        {
                            if *wallet_initialized && (*wallet_error).is_none() {
                                html! {
                                    <div class="wallet-section">
                                        {
                                            if let Some(account) = (*wallet_account).clone() {
                                                if account.is_connected {
                                                    html! {
                                                        <div class="wallet-status connected" onclick={on_disconnect_wallet.clone()} style="cursor: pointer;">
                                                            <span style="font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace; font-size: 14px;">
                                                                {format!("{}...{}", 
                                                                    account.address.as_ref().map(|a| &a[..4]).unwrap_or(""),
                                                                    account.address.as_ref().map(|a| &a[a.len()-4..]).unwrap_or("")
                                                                )}
                                                            </span>
                                                            <span style="margin-left: 8px; font-size: 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;">{"✕"}</span>
                                                        </div>
                                                    }
                                                } else {
                                                    html! {
                                                        <button onclick={on_connect_wallet} class="wallet-button">
                                                            {"Connect"}
                                                        </button>
                                                    }
                                                }
                                            } else {
                                                html! {
                                                    <button onclick={on_connect_wallet} class="wallet-button">
                                                        {"Connect"}
                                                    </button>
                                                }
                                            }
                                        }
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>

                    <div class="search-content">

                        <div class="search-box">
                            <input 
                                type="text" 
                                class="search-input"
                                placeholder="give me a fid/username"
                                value={(*search_input).clone()}
                                oninput={
                                    let search_input = search_input.clone();
                                    Callback::from(move |e: InputEvent| {
                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                            search_input.set(input.value());
                                        }
                                    })
                                }
                                onkeypress={on_keypress}
                            />
                            <button 
                                class="search-button"
                                onclick={on_search.reform(|_| ())}
                                disabled={*is_loading}
                            >
                                {
                                    if *is_loading {
                                        html! { "Searching..." }
                                    } else {
                                        html! { "⌕" }
                                    }
                                }
                            </button>
                        </div>

                        // Mobile-only search button
                        <div class="mobile-search-button">
                            <button 
                                class="mobile-search-btn"
                                onclick={on_search.reform(|_| ())}
                                disabled={*is_loading}
                            >
                                {
                                    if *is_loading {
                                        html! { "Searching..." }
                                    } else {
                                        html! { "Search" }
                                    }
                                }
                            </button>
                        </div>

                        <div class="search-suggestions">
                            <p class="suggestions-title">{"Popular:"}</p>
                            <div class="suggestion-tags">
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "vitalik.eth".to_string())}>{"@vitalik.eth"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "jesse.base.eth".to_string())}>{"@jesse.base.eth"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "ryankung.base.eth".to_string())}>{"@ryankung.base.eth"}</button>
                            </div>
                        </div>

                        if let Some(error) = (*error_message).clone() {
                            <div class="error-message">
                                <p>{error}</p>
                                        </div>
                        }
                    </div>
                </div>
            }

            // Results Page (Profile + Chat cards)
            if (*search_result).is_some() {
                <div class="results-page">
                    // Smart Back Button
                    <div class="back-to-search">
                        <button class="back-button" onclick={on_smart_back}>
                            {"←"}
                        </button>
                    </div>
                    
                    // Profile Card (only show if current_view is "profile")
                    if *current_view == "profile" {
                        <div class="card profile-card">
                            <div class="card-content">
                                if let Some(result) = (*search_result).clone() {
                                <div class="profile-info">
                                    <div class="profile-picture">
                                        if let Some(pfp_url) = &result.profile.pfp_url {
                                            <img src={pfp_url.clone()} alt="Profile" />
                                        } else {
                                            <div class="profile-picture-placeholder">
                                                {"👤"}
                                            </div>
                                        }
                                    </div>

                                    <div class="user-details">
                                        <h2>{result.profile.display_name.clone().unwrap_or_else(|| "Unknown".to_string())}</h2>
                                        if let Some(username) = &result.profile.username {
                                            <p class="username">{"@"}{username}</p>
                                        }
                                        <div class="fid-badge">{"FID: "}{result.profile.fid}</div>
                                        
                                        if let Some(bio) = &result.profile.bio {
                                            <p class="bio">{bio}</p>
                                        }
                                    </div>
                    </div>

                                if let Some(social) = &result.social {
                                    <div class="social-analysis">
                                        <div class="social-stats">
                                            <div class="stat-item">
                                                <div class="stat-label">{"Following"}</div>
                                                <div class="stat-value">{format!("{}", social.following_count)}</div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">{"Followers"}</div>
                                                <div class="stat-value">{format!("{}", social.followers_count)}</div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">{"Influence"}</div>
                                                <div class="stat-value">{format!("{:.1}", social.influence_score)}</div>
                                            </div>
                                        </div>

                                        <div class="social-circles">
                                            <h4>{"Social Circles"}</h4>
                                            <div class="circle-item">
                                                <span>{"Tech Builders"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.tech_builders * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Content Creators"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.content_creators * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Web3 Natives"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.web3_natives * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Casual Users"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.casual_users * 100.0)}></div>
                                                </div>
                                            </div>
                                        </div>

                                        // Interaction Style
                                        <div class="interaction-style">
                                            <h4>{"Interaction Style"}</h4>
                                            <div class="interaction-stats">
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Reply Frequency"}</span>
                                                    <div class="interaction-bar">
                                                        <div class="interaction-fill" style={format!("width: {}%", social.interaction_style.reply_frequency * 100.0)}></div>
                                                    </div>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Mention Frequency"}</span>
                                                    <div class="interaction-bar">
                                                        <div class="interaction-fill" style={format!("width: {}%", social.interaction_style.mention_frequency * 100.0)}></div>
                                                    </div>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Network Connector"}</span>
                                                    <span class="interaction-value">{if social.interaction_style.network_connector { "Yes" } else { "No" }}</span>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Community Role"}</span>
                                                    <span class="interaction-value">{&social.interaction_style.community_role}</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Most Mentioned Users
                                        if !social.most_mentioned_users.is_empty() {
                                            <div class="mentioned-users">
                                                <h4>{"Most Mentioned Users"}</h4>
                                                <div class="mentioned-list">
                                                    {for social.most_mentioned_users.iter().take(5).map(|user| {
                                                        html! {
                                                            <div class="mentioned-item">
                                                                <span class="mentioned-name">
                                                                    {user.display_name.clone().unwrap_or_else(|| user.username.clone().unwrap_or_else(|| format!("FID {}", user.fid)))}
                                                                </span>
                                                                <span class="mentioned-count">{format!("{} mentions", user.count)}</span>
                                                                <span class="mentioned-category">{&user.category}</span>
                                                            </div>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }

                                        // Word Cloud
                                        if !social.word_cloud.top_words.is_empty() {
                                            <div class="word-cloud">
                                                <h4>{"Top Words"}</h4>
                                                <div class="word-tags">
                                                    {for social.word_cloud.top_words.iter().take(10).map(|word| {
                                                        html! {
                                                            <span class="word-tag" style={format!("font-size: {}px", (word.percentage * 20.0 + 12.0).max(10.0).min(18.0))}>
                                                                {&word.word}
                                                            </span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }

                                        // Signature Words
                                        if !social.word_cloud.signature_words.is_empty() {
                                            <div class="signature-words">
                                                <h4>{"Signature Words"}</h4>
                                                <div class="signature-tags">
                                                    {for social.word_cloud.signature_words.iter().map(|word| {
                                                        html! {
                                                            <span class="signature-tag">{word}</span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }
                                    </div>
                                }
                            }
                            </div>
                        </div>
                    }

                    // Chat Card (only show if current_view is "chat")
                    if *current_view == "chat" {
                        <div class="card chat-card">
                            <div class="card-content">
                                if let Some(session) = (*chat_session).clone() {
                                    <div class="chat-user-info">
                                        <div class="chat-user-avatar">
                                            if let Some(result) = (*search_result).clone() {
                                                if let Some(pfp_url) = &result.profile.pfp_url {
                                                    <img src={pfp_url.clone()} alt="Profile" />
                                                } else {
                                                    <div class="chat-avatar-placeholder">
                                                        {session.display_name.clone().unwrap_or_else(|| "Unknown".to_string()).chars().next().unwrap_or('?').to_uppercase().collect::<String>()}
                                                    </div>
                                                }
                                            } else {
                                                <div class="chat-avatar-placeholder">
                                                    {session.display_name.clone().unwrap_or_else(|| "Unknown".to_string()).chars().next().unwrap_or('?').to_uppercase().collect::<String>()}
                                                </div>
                                            }
                                        </div>
                                        <div class="chat-user-details">
                                            <h3>{session.display_name.clone().unwrap_or_else(|| "Unknown".to_string())}</h3>
                                            <p>{"FID: "}{session.fid}</p>
                                        </div>
                                        <div class="chat-status">
                                            <div class="chat-status-dot"></div>
                                            {"Online"}
                                        </div>
                                    </div>

                                    <div class="chat-messages">
                                        {for (*chat_messages).iter().map(|message| {
                                                html! {
                                                <div class={if message.role == "user" { "user-message" } else { "assistant-message" }}>
                                                    <div class="message-content">
                                                        {&message.content}
                                                    </div>
                                                    <div class="message-time">
                                                        {"Now"}
                                                    </div>
                                                </div>
                                            }
                                        })}
                                        
                                        if *is_chat_loading {
                                            <div class="assistant-message">
                                                <div class="message-content loading">
                                                    <div class="typing-indicator">
                                                        <span></span>
                                                        <span></span>
                                                        <span></span>
                                                    </div>
                                                    {"AI is thinking..."}
                                                </div>
                                            </div>
                                        }
                                    </div>

                                    <div class="chat-input">
                                        <div class="chat-input-box">
                                            <input 
                                                type="text" 
                                                class="chat-input-field"
                                                placeholder="Ask me anything about this user..."
                                                value={(*chat_message).clone()}
                                                            oninput={
                                                    let chat_message = chat_message.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                            chat_message.set(input.value());
                                                        }
                                                    })
                                                }
                                                onkeypress={
                                                    let on_send_chat_message = on_send_chat_message.clone();
                                                    Callback::from(move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            on_send_chat_message.emit(());
                                                        }
                                                    })
                                                }
                                            />
                                            <button 
                                                class="chat-send-button"
                                                onclick={on_send_chat_message.reform(|_| ())}
                                                disabled={*is_chat_loading}
                                            >
                                                {"➤"}
                                            </button>
                                        </div>
                                    </div>

                                    if let Some(error) = (*chat_error).clone() {
                                        <div class="error-message">
                                            <p>{error}</p>
                                        </div>
                                    }
                                } else {
                                    <div class="no-chat-session">
                                        <p>{"No chat session available. Please try searching again."}</p>
                                    </div>
                                }
                            </div>
                        </div>
                    }
                </div>

                // Floating Chat Button (only show on results page when profile is visible)
                if (*search_result).is_some() && *current_view == "profile" {
                    <div class="floating-chat-button" onclick={on_switch_to_chat}>
                        {"💭"}
                    </div>
                }
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}