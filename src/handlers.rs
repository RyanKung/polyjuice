use wasm_bindgen_futures::spawn_local;
use web_sys::InputEvent;
use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::models::*;
use crate::services::*;
use crate::wallet;
use crate::wallet::WalletAccount;

// ============================================================================
// Helper Functions for Job Status Management
// ============================================================================

// ============================================================================
// Main Search Handler
// ============================================================================

/// Perform search with given query (shared logic for both search handler and URL restoration)
/// Now it just sets the search_query state - ProfileLoader component will handle loading
#[allow(clippy::too_many_arguments)]
pub async fn perform_search(
    search_query: String,
    is_fid: bool,
    search_query_state: UseStateHandle<Option<String>>,
    is_fid_state: UseStateHandle<bool>,
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
    error_message: UseStateHandle<Option<String>>,
    _api_url: String, // Not used anymore, kept for URL path update
    current_view: UseStateHandle<String>,
) {
    // Set loading state
    loading_tasks.set(std::collections::HashSet::from_iter([
        "Searching...".to_string()
    ]));
    error_message.set(None);

    // Set search query - ProfileLoader will handle the loading
    search_query_state.set(Some(search_query.clone()));
    is_fid_state.set(is_fid);
    current_view.set("profile".to_string());

    // Scroll to top immediately when search starts
    if let Some(window) = web_sys::window() {
        window.scroll_to_with_x_and_y(0.0, 0.0);
        if let Some(document) = window.document() {
            if let Ok(Some(main_content)) = document.query_selector(".main-content") {
                use wasm_bindgen::JsCast;
                if let Ok(main_element) = main_content.dyn_into::<web_sys::HtmlElement>() {
                    main_element.scroll_to_with_x_and_y(0.0, 0.0);
                }
            }
        }
    }

    // Clear loading state - ProfileLoader will show its own loading state
    loading_tasks.set(std::collections::HashSet::new());

    // Update URL path
    let query_for_url = if is_fid {
        search_query.clone()
    } else {
        format!("@{}", search_query)
    };
    crate::services::update_url_path(&query_for_url, "profile");

    // Chat session will be created when user clicks the chat button
}

// ============================================================================
// Handler Creators
// ============================================================================

/// Create wallet select handler
pub fn create_wallet_select_handler(
    wallet_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
    api_url: UseStateHandle<String>,
) -> Callback<String> {
    Callback::from(move |uuid: String| {
        let wallet_error = wallet_error.clone();
        let wallet_account = wallet_account.clone();
        let api_url = (*api_url).clone();
        spawn_local(async move {
            web_sys::console::log_1(&format!("🔌 Connecting to wallet: {}", uuid).into());
            wallet_error.set(None);

            match crate::wallet::connect_to_wallet(&uuid).await {
                Ok(_) => {
                    // Poll for account update
                    let mut attempts = 0;
                    let mut connected_account: Option<WalletAccount> = None;
                    while attempts < 30 {
                        if let Ok(account) = crate::wallet::get_account().await {
                            if account.is_connected {
                                web_sys::console::log_1(
                                    &format!("✅ Account connected: {:?}", account.address).into(),
                                );
                                connected_account = Some(account);
                                break;
                            }
                        }
                        gloo_timers::future::TimeoutFuture::new(500).await;
                        attempts += 1;
                    }

                    // Final check if not found yet
                    if connected_account.is_none() {
                        if let Ok(account) = crate::wallet::get_account().await {
                            connected_account = Some(account);
                        } else {
                            wallet_error
                                .set(Some("Connection timeout. Please try again.".to_string()));
                            return;
                        }
                    }

                    // Get FID for the connected address
                    if let Some(ref account) = connected_account {
                        if let Some(ref address) = account.address {
                            // Save wallet connection to localStorage
                            if let Err(e) = crate::wallet::save_wallet_to_storage(&uuid, address) {
                                web_sys::console::warn_1(
                                    &format!("⚠️ Failed to save wallet to storage: {}", e).into(),
                                );
                            }

                            web_sys::console::log_1(
                                &"🔍 Fetching FID for connected address...".into(),
                            );
                            match crate::wallet::get_fid_for_address(&api_url, address).await {
                                Ok(fid) => {
                                    let mut updated_account = account.clone();
                                    updated_account.fid = fid;
                                    wallet_account.set(Some(updated_account));
                                    if let Some(fid_value) = fid {
                                        web_sys::console::log_1(
                                            &format!("✅ FID found: {}", fid_value).into(),
                                        );
                                    } else {
                                        web_sys::console::log_1(
                                            &"ℹ️ No FID found for this address".into(),
                                        );
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::log_1(
                                        &format!("⚠️ Failed to fetch FID: {}", e).into(),
                                    );
                                    // Still set the account even if FID fetch failed
                                    wallet_account.set(Some(account.clone()));
                                }
                            }
                        } else {
                            wallet_account.set(Some(account.clone()));
                        }
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("❌ Wallet connection failed: {}", e).into());
                    wallet_error.set(Some(e));
                }
            }
        });
    })
}

/// Create wallet disconnect handler
pub fn create_wallet_disconnect_handler(
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
    Callback::from(move |_| {
        let wallet_account = wallet_account.clone();
        spawn_local(async move {
            let _ = crate::wallet::disconnect().await;
            // Clear wallet from localStorage
            let _ = crate::wallet::clear_wallet_from_storage();
            wallet_account.set(None);
        });
    })
}

/// Create search handler
#[allow(clippy::too_many_arguments)]
pub fn create_search_handler(
    search_input: UseStateHandle<String>,
    search_query_state: UseStateHandle<Option<String>>,
    is_fid_state: UseStateHandle<bool>,
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
    error_message: UseStateHandle<Option<String>>,
    api_url: UseStateHandle<String>,
    current_view: UseStateHandle<String>,
) -> Callback<()> {
    Callback::from(move |_| {
        let input = (*search_input).clone();
        if input.trim().is_empty() {
            return;
        }

        let search_query_state = search_query_state.clone();
        let is_fid_state = is_fid_state.clone();
        let loading_tasks = loading_tasks.clone();
        let error_message = error_message.clone();
        let api_url_clone = (*api_url).clone();
        let current_view = current_view.clone();

        spawn_local(async move {
            // Determine if input is FID (numeric) or username (text)
            let trimmed_input = input.trim();
            let is_fid = trimmed_input.parse::<u64>().is_ok();

            // Handle username with @ prefix
            let search_query = if is_fid {
                trimmed_input.to_string()
            } else {
                trimmed_input.trim_start_matches('@').to_string()
            };

            perform_search(
                search_query,
                is_fid,
                search_query_state,
                is_fid_state,
                loading_tasks,
                error_message,
                api_url_clone,
                current_view,
            )
            .await;
        });
    })
}

/// Create chat session after successful search
#[allow(clippy::too_many_arguments)]
pub async fn create_chat_session_after_search(
    api_url: &str,
    search_query: &str,
    is_fid: bool,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) {
    web_sys::console::log_1(
        &format!(
            "💬 Creating chat session for query: {}, is_fid: {}",
            search_query, is_fid
        )
        .into(),
    );
    is_chat_loading.set(true);
    chat_error.set(None);

    let request = CreateChatRequest {
        user: if is_fid {
            search_query.to_string()
        } else {
            format!("@{}", search_query)
        },
        context_limit: 20,
        temperature: 0.7,
    };

    let request_json = serde_json::to_string(&request).unwrap();
    let chat_endpoint = create_chat_session_endpoint();

    match make_request_with_payment::<CreateChatResponse>(
        api_url,
        &chat_endpoint,
        Some(request_json),
        wallet_account.as_ref(),
        None,
        None,
    )
    .await
    {
        Ok(chat_data) => {
            web_sys::console::log_1(
                &format!(
                    "✅ Chat session created successfully: session_id={}, fid={}",
                    chat_data.session_id, chat_data.fid
                )
                .into(),
            );
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
            web_sys::console::log_1(&"✅ Chat session state updated".into());
        }
        Err(e) => {
            web_sys::console::log_1(&format!("❌ Chat session creation failed: {}", e).into());
            chat_error.set(Some(e));
        }
    }

    is_chat_loading.set(false);
}

/// Create chat message send handler
pub fn create_chat_message_handler(
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_message: UseStateHandle<String>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    api_url: UseStateHandle<String>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
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

                // Add user message to chat immediately
                let user_message = ChatMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                    timestamp: 0,
                };
                let mut messages = (*chat_messages).clone();
                messages.push(user_message.clone());
                chat_messages.set(messages.clone());

                // Send message to API
                let request = ChatMessageRequest {
                    session_id: session.session_id.clone(),
                    message: message.clone(),
                };

                let request_json = serde_json::to_string(&request).unwrap();
                let chat_endpoint = create_chat_message_endpoint();
                
                // Remember the initial message count before sending (includes the user message we just added)
                let initial_message_count = messages.len();

                match make_request_with_payment::<ChatMessageResponse>(
                    &api_url,
                    &chat_endpoint,
                    Some(request_json.clone()),
                    wallet_account.as_ref(),
                    None,
                    None,
                )
                .await
                {
                    Ok(chat_data) => {
                        // Check if response indicates pending status
                        // According to api.md, chat message API returns pending with message containing "Processing... Please check back later or poll for result"
                        if chat_data.message.contains("Processing... Please check back later") || 
                           chat_data.message.contains("Please check back later or poll for result") {
                            web_sys::console::log_1(
                                &"💬 Chat message API returned pending status, starting polling...".into(),
                            );
                            
                            // Clear input box
                            chat_message.set(String::new());
                            
                            // Start polling using GET /api/chat/session endpoint
                            let api_url_clone = api_url.clone();
                            let session_id = session.session_id.clone();
                            let wallet_account_clone = wallet_account.clone();
                            let chat_messages_clone = chat_messages.clone();
                            let is_chat_loading_clone = is_chat_loading.clone();
                            let chat_error_clone = chat_error.clone();
                            let initial_message_count_clone = initial_message_count;
                            
                            spawn_local(async move {
                                // Poll the chat session endpoint with exponential backoff
                                let max_attempts = 200;
                                let initial_interval_ms = 2000u64;
                                let max_interval_ms = 15000u64;
                                let mut current_interval = initial_interval_ms;
                                
                                for attempt in 0..max_attempts {
                                    // Wait before polling (except first attempt)
                                    if attempt > 0 {
                                        current_interval = (current_interval * 2).min(max_interval_ms);
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
                                    
                                    web_sys::console::log_1(
                                        &format!(
                                            "📊 Polling chat session attempt {}/{}",
                                            attempt + 1,
                                            max_attempts
                                        )
                                        .into(),
                                    );
                                    
                                    // Poll using GET /api/chat/session?session_id=xxx
                                    let session_endpoint = create_get_chat_session_endpoint(&session_id);
                                    match make_request_with_payment::<ChatSession>(
                                        &api_url_clone,
                                        &session_endpoint,
                                        None, // GET request, no body
                                        wallet_account_clone.as_ref(),
                                        None,
                                        None,
                                    )
                                    .await
                                    {
                                        Ok(session_data) => {
                                            // Check if there are new messages in conversation_history
                                            let current_messages_count = session_data.conversation_history.len();
                                            
                                            if current_messages_count > initial_message_count_clone {
                                                // Got new messages, update UI with the full conversation_history
                                                // This ensures we have the latest state from the server
                                                chat_messages_clone.set(session_data.conversation_history);
                                                chat_error_clone.set(None);
                                                is_chat_loading_clone.set(false);
                                                web_sys::console::log_1(&"✅ Chat message polling completed!".into());
                                                return;
                                            } else {
                                                // No new messages yet, continue polling
                                                continue;
                                            }
                                        }
                                        Err(e) => {
                                            // Check if it's a transient error or permanent error
                                            if attempt < max_attempts - 1 {
                                                // Continue polling on transient errors
                                                web_sys::console::log_1(
                                                    &format!("⚠️ Polling error (will retry): {}", e).into(),
                                                );
                                                continue;
                                            } else {
                                                // Final attempt failed
                                                chat_error_clone.set(Some(e));
                                                is_chat_loading_clone.set(false);
                                                return;
                                            }
                                        }
                                    }
                                }
                                
                                // Max attempts reached
                                chat_error_clone.set(Some("Chat message polling timed out. Please try again later.".to_string()));
                                is_chat_loading_clone.set(false);
                            });
                            // Return early - don't execute code after match (polling handles its own state)
                            return;
                        } else {
                            // Not pending, use the response directly
                            let assistant_message = ChatMessage {
                                role: "assistant".to_string(),
                                content: chat_data.message,
                                timestamp: 0,
                            };
                            messages.push(assistant_message);
                            chat_messages.set(messages);
                            chat_error.set(None);
                        }
                    }
                    Err(e) => {
                        chat_error.set(Some(e));
                    }
                }

                chat_message.set(String::new());
                is_chat_loading.set(false);
            });
        }
    })
}

/// Create enter key handler for search
pub fn create_search_keypress_handler(on_search: Callback<()>) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Enter" {
            on_search.emit(());
        }
    })
}

/// Create enter key handler for chat
pub fn create_chat_keypress_handler(on_send_message: Callback<()>) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Enter" {
            on_send_message.emit(());
        }
    })
}

/// Create popular FID handler
pub fn create_popular_fid_handler(
    search_input: UseStateHandle<String>,
    on_search: Callback<()>,
) -> Callback<String> {
    Callback::from(move |fid: String| {
        search_input.set(fid.clone());
        on_search.emit(());
    })
}

/// Create view switching handler - creates chat session when switching to chat
#[allow(clippy::too_many_arguments)]
pub fn create_view_switch_handler(
    current_view: UseStateHandle<String>,
    search_query: UseStateHandle<Option<String>>,
    is_fid: UseStateHandle<bool>,
    api_url: UseStateHandle<String>,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
    Callback::from(move |_| {
        // Check if we already have a chat session
        if (*chat_session).is_some() {
            // Session already exists, just switch view
            current_view.set("chat".to_string());
            if let Some(query) = (*search_query).as_ref() {
                let query_for_url = if *is_fid {
                    query.clone()
                } else {
                    format!("@{}", query)
                };
                crate::services::update_url_path(&query_for_url, "chat");
            }
            return;
        }

        // No session yet, need to create one
        if let Some(query) = (*search_query).as_ref() {
            web_sys::console::log_1(
                &format!(
                    "💬 Switching to chat view and creating session for query: {}",
                    query
                )
                .into(),
            );
            let query_clone = query.clone();
            let is_fid_clone = *is_fid;
            let api_url_clone = (*api_url).clone();
            let chat_session_clone = chat_session.clone();
            let chat_messages_clone = chat_messages.clone();
            let is_chat_loading_clone = is_chat_loading.clone();
            let chat_error_clone = chat_error.clone();
            let wallet_account_clone = wallet_account.clone();

            // Switch view first
            current_view.set("chat".to_string());
            let query_for_url = if is_fid_clone {
                query_clone.clone()
            } else {
                format!("@{}", query_clone)
            };
            crate::services::update_url_path(&query_for_url, "chat");

            // Create chat session
            spawn_local(async move {
                create_chat_session_after_search(
                    &api_url_clone,
                    &query_clone,
                    is_fid_clone,
                    chat_session_clone,
                    chat_messages_clone,
                    is_chat_loading_clone,
                    chat_error_clone,
                    wallet_account_clone,
                )
                .await;
            });
        }
    })
}

/// Create smart back button handler
#[allow(clippy::too_many_arguments)]
pub fn create_smart_back_handler(
    current_view: UseStateHandle<String>,
    search_query: UseStateHandle<Option<String>>,
    is_fid: UseStateHandle<bool>,
    search_input: UseStateHandle<String>,
    error_message: UseStateHandle<Option<String>>,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
) -> Callback<()> {
    Callback::from(move |_| match (*current_view).as_str() {
        "profile" => {
            search_query.set(None);
            search_input.set(String::new());
            error_message.set(None);
            chat_session.set(None);
            chat_messages.set(Vec::new());
            loading_tasks.set(std::collections::HashSet::new());
            current_view.set("profile".to_string());
            crate::services::clear_url_path();
        }
        "chat" => {
            current_view.set("profile".to_string());
            if let Some(query) = (*search_query).as_ref() {
                let query_for_url = if *is_fid {
                    query.clone()
                } else {
                    format!("@{}", query)
                };
                crate::services::update_url_path(&query_for_url, "profile");
            }
        }
        _ => {
            search_query.set(None);
            search_input.set(String::new());
            error_message.set(None);
            chat_session.set(None);
            chat_messages.set(Vec::new());
            loading_tasks.set(std::collections::HashSet::new());
            current_view.set("profile".to_string());
            crate::services::clear_url_path();
        }
    })
}

/// Create input change handler
pub fn create_input_change_handler(input_state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
            input_state.set(input.value());
        }
    })
}

/// Create endpoint fetch handler
pub fn create_endpoint_fetch_handler(
    endpoint_data: UseStateHandle<Option<EndpointData>>,
    is_loading: UseStateHandle<bool>,
    error: UseStateHandle<Option<String>>,
    ping_results: UseStateHandle<Vec<(String, Option<f64>)>>,
) -> Callback<()> {
    Callback::from(move |_| {
        let endpoint_data = endpoint_data.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();
        let ping_results = ping_results.clone();

        let contract_address = "0xf16e03526d1be6d120cfbf5a24e1ac78a8192663";
        let rpc_url = "https://sepolia.base.org";

        spawn_local(async move {
            is_loading.set(true);
            error.set(None);

            match wallet::get_endpoints(contract_address, rpc_url).await {
                Ok(data) => {
                    endpoint_data.set(Some(data.clone()));
                    error.set(None);
                    is_loading.set(false);

                    let endpoints = data.endpoints.clone();
                    let ping_results_handle = ping_results.clone();

                    spawn_local(async move {
                        let mut results = Vec::new();
                        for endpoint in &endpoints {
                            let result = wallet::ping_endpoint_service(endpoint).await.ok();
                            results.push((endpoint.clone(), result));
                        }
                        ping_results_handle.set(results);
                    });
                }
                Err(e) => {
                    error.set(Some(e));
                    endpoint_data.set(None);
                    is_loading.set(false);
                }
            }
        });
    })
}
