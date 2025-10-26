use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;
use web_sys::InputEvent;

use crate::models::*;
use crate::services::*;
use crate::wallet::WalletAccount;

/// Create wallet connect handler
pub fn create_wallet_connect_handler(
    wallet_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
    Callback::from(move |_| {
        let wallet_error = wallet_error.clone();
        let wallet_account = wallet_account.clone();
        spawn_local(async move {
            match crate::wallet::connect().await {
                Ok(_) => {
                    wallet_error.set(None);
                    if let Ok(account) = crate::wallet::get_account().await {
                        wallet_account.set(Some(account));
                    }
                }
                Err(e) => {
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
            wallet_account.set(None);
        });
    })
}

/// Create search handler
pub fn create_search_handler(
    search_input: UseStateHandle<String>,
    search_result: UseStateHandle<Option<SearchResult>>,
    is_loading: UseStateHandle<bool>,
    error_message: UseStateHandle<Option<String>>,
    api_url: UseStateHandle<String>,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
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

            // Get profile data
            let profile_endpoint = create_profile_endpoint(&search_query, is_fid);
            match make_request_with_payment::<ProfileData>(&api_url, &profile_endpoint, None, wallet_account.as_ref()).await {
                Ok(profile) => {
                    // Get social data
                    let social_endpoint = create_social_endpoint(&search_query, is_fid);
                    let social_data = match make_request_with_payment::<SocialData>(&api_url, &social_endpoint, None, wallet_account.as_ref()).await {
                        Ok(social) => Some(social),
                        Err(_) => None, // Social data is optional
                    };

                    search_result.set(Some(SearchResult {
                        profile,
                        social: social_data,
                    }));
                    error_message.set(None);

                    // Auto-create chat session after successful search
                    create_chat_session_after_search(
                        &api_url,
                        &search_query,
                        is_fid,
                        chat_session.clone(),
                        chat_messages.clone(),
                        is_chat_loading.clone(),
                        chat_error.clone(),
                        wallet_account.clone(),
                    ).await;
                }
                Err(e) => {
                    error_message.set(Some(e));
                }
            }

            is_loading.set(false);
        });
    })
}

/// Create chat session after successful search
async fn create_chat_session_after_search(
    api_url: &str,
    search_query: &str,
    is_fid: bool,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) {
    is_chat_loading.set(true);
    chat_error.set(None);

    // Create chat session
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

    match make_request_with_payment::<CreateChatResponse>(api_url, &chat_endpoint, Some(request_json), wallet_account.as_ref()).await {
        Ok(chat_data) => {
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
        }
        Err(e) => {
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

                match make_request_with_payment::<ChatMessageResponse>(&api_url, &chat_endpoint, Some(request_json), wallet_account.as_ref()).await {
                    Ok(chat_data) => {
                        let assistant_message = ChatMessage {
                            role: "assistant".to_string(),
                            content: chat_data.message,
                            timestamp: 0,
                        };
                        // Append assistant message to the existing messages
                        messages.push(assistant_message);
                        chat_messages.set(messages);
                        chat_error.set(None);
                    }
                    Err(e) => {
                        // On error, keep the user message in the chat
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

/// Create view switching handler
pub fn create_view_switch_handler(current_view: UseStateHandle<String>) -> Callback<()> {
    Callback::from(move |_| {
        current_view.set("chat".to_string());
    })
}

/// Create smart back button handler
pub fn create_smart_back_handler(
    current_view: UseStateHandle<String>,
    search_result: UseStateHandle<Option<SearchResult>>,
    search_input: UseStateHandle<String>,
    error_message: UseStateHandle<Option<String>>,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
) -> Callback<()> {
    Callback::from(move |_| {
        match (*current_view).as_str() {
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
                current_view.set("profile".to_string());
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
}

/// Create input change handler
pub fn create_input_change_handler(input_state: UseStateHandle<String>) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
            input_state.set(input.value());
        }
    })
}
