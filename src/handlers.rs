use std::rc::Rc;

use futures::future::join;
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

/// Helper function to update pending_jobs in search_result
fn update_pending_job(
    search_result: &UseStateHandle<Option<SearchResult>>,
    job_type: &str,
    status: String,
    job_key: String,
    message: String,
) {
    if let Some(current_result) = search_result.as_ref() {
        let mut pending_jobs = current_result.pending_jobs.clone().unwrap_or_default();

        // Update or add job
        if let Some(existing_job) = pending_jobs.iter_mut().find(|j| j.job_type == job_type) {
            existing_job.status = Some(status.clone());
            existing_job.job_key = job_key.clone();
            existing_job.message = Some(message.clone());
        } else {
            pending_jobs.push(crate::models::PendingJob {
                job_key: job_key.clone(),
                job_type: job_type.to_string(),
                status: Some(status.clone()),
                started_at: Some(js_sys::Date::now() as u64),
                message: Some(message.clone()),
            });
        }

        let updated_result = SearchResult {
            pending_jobs: Some(pending_jobs),
            ..current_result.clone()
        };
        search_result.set(Some(updated_result));
    }
}

/// Create a status callback for updating UI during background polling
fn create_polling_status_callback(
    search_result: UseStateHandle<Option<SearchResult>>,
    job_type: &'static str,
) -> Option<crate::services::StatusCallback> {
    Some(Rc::new(Box::new(move |status, job_key, message| {
        update_pending_job(&search_result, job_type, status, job_key, message);
    })))
}

/// Parse JOB_STATUS error message format: "JOB_STATUS:{status}:JOB_KEY:{job_key}:MESSAGE:{message}"
fn parse_job_status_error(
    error: &str,
    default_job_key: String,
) -> Option<(String, String, String)> {
    if !error.starts_with("JOB_STATUS:") {
        return None;
    }

    let parts: Vec<&str> = error.split(":JOB_KEY:").collect();
    if parts.len() != 2 {
        return None;
    }

    let status_part = parts[0].strip_prefix("JOB_STATUS:").unwrap_or("");
    let rest = parts[1];
    let key_parts: Vec<&str> = rest.split(":MESSAGE:").collect();

    let status = if !status_part.is_empty() {
        status_part.to_string()
    } else {
        "pending".to_string()
    };

    let job_key = if !key_parts.is_empty() && !key_parts[0].is_empty() {
        key_parts[0].to_string()
    } else {
        default_job_key
    };

    let message = if key_parts.len() >= 2 {
        key_parts[1].to_string()
    } else {
        format!(
            "{} analysis is still processing. You can come back later to check the results.",
            if job_key.starts_with("social") {
                "Social"
            } else {
                "MBTI"
            }
        )
    };

    Some((status, job_key, message))
}

/// Process API result and extract pending job information
fn process_analysis_result<T>(
    result: Result<T, String>,
    job_type: &str,
    fid: i64,
    default_message: &str,
) -> (Option<T>, Option<PendingJob>) {
    match result {
        Ok(data) => {
            web_sys::console::log_1(&format!("✅ {} data received", job_type).into());
            (Some(data), None)
        }
        Err(e) => {
            // Check for JOB_STATUS error format
            if let Some((status, job_key, message)) =
                parse_job_status_error(&e, format!("{}:{}", job_type, fid))
            {
                web_sys::console::log_1(
                    &format!(
                        "⏳ {} analysis status: {} (job_key: {})",
                        job_type, status, job_key
                    )
                    .into(),
                );
                let pending_job = PendingJob {
                    job_key,
                    job_type: job_type.to_string(),
                    status: Some(status),
                    started_at: Some(js_sys::Date::now() as u64),
                    message: Some(message),
                };
                return (None, Some(pending_job));
            }

            // Check for timeout errors
            if e.contains("did not complete within") || e.contains("still be processing") {
                web_sys::console::log_1(
                    &format!("⏳ {} analysis is taking longer than expected. It will continue in the background.", job_type).into()
                );
                let status = if e.contains("processing") {
                    "processing"
                } else {
                    "pending"
                };
                let pending_job = PendingJob {
                    job_key: format!("{}:{}", job_type, fid),
                    job_type: job_type.to_string(),
                    status: Some(status.to_string()),
                    started_at: Some(js_sys::Date::now() as u64),
                    message: Some(default_message.to_string()),
                };
                return (None, Some(pending_job));
            }

            // Check for failed job
            if e.contains("Job failed") {
                web_sys::console::log_1(&format!("❌ {} analysis failed: {}", job_type, e).into());
                let pending_job = PendingJob {
                    job_key: format!("{}:{}", job_type, fid),
                    job_type: job_type.to_string(),
                    status: Some("failed".to_string()),
                    started_at: Some(js_sys::Date::now() as u64),
                    message: Some(e),
                };
                return (None, Some(pending_job));
            }

            // Other errors
            web_sys::console::log_1(&format!("⚠️ {} data error: {}", job_type, e).into());
            (None, None)
        }
    }
}

// ============================================================================
// Main Search Handler
// ============================================================================

/// Perform search with given query (shared logic for both search handler and URL restoration)
#[allow(clippy::too_many_arguments)]
pub async fn perform_search(
    search_query: String,
    is_fid: bool,
    search_result: UseStateHandle<Option<SearchResult>>,
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
    error_message: UseStateHandle<Option<String>>,
    api_url: String,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
    current_view: UseStateHandle<String>,
) {
    // Set loading state
    loading_tasks.set(std::collections::HashSet::from_iter([
        "Searching...".to_string()
    ]));
    error_message.set(None);

    // Clone values needed for futures
    let api_url_clone = api_url.clone();
    let wallet_account_clone = wallet_account.clone();

    let profile_endpoint = create_profile_endpoint(&search_query, is_fid);
    let social_endpoint = create_social_endpoint(&search_query, is_fid);
    let mbti_endpoint = create_mbti_endpoint(&search_query, is_fid);

    // First, wait for profile to load (needed for FID and to render the page)
    let profile_future = make_request_with_payment::<ProfileData>(
        &api_url_clone,
        &profile_endpoint,
        None,
        wallet_account_clone.as_ref(),
        None,
        None,
    );

    let profile_result = profile_future.await;

    match profile_result {
        Ok(profile) => {
            // Clear loading state and show profile
            loading_tasks.set(std::collections::HashSet::new());

            let initial_result = SearchResult {
                profile: profile.clone(),
                social: None,
                mbti: None,
                pending_jobs: None,
            };
            // CRITICAL: Set search_result BEFORE creating status callbacks
            // This ensures status callbacks can see the updated state
            // IMPORTANT: Use clone() to avoid moving initial_result
            let initial_result_clone = initial_result.clone();
            search_result.set(Some(initial_result_clone));
            current_view.set("profile".to_string());

            // Force a small delay to allow Yew to process the state update
            // This is necessary because UseStateHandle.set() is asynchronous in nature
            gloo_timers::future::TimeoutFuture::new(50).await;

            // Update URL path
            let query_for_url = if is_fid {
                search_query.clone()
            } else {
                format!("@{}", search_query)
            };
            crate::services::update_url_path(&query_for_url, "profile");

            // Create status callbacks for background polling updates
            let search_result_for_social = search_result.clone();
            let search_result_for_mbti = search_result.clone();
            let social_status_callback =
                create_polling_status_callback(search_result_for_social, "social");
            let mbti_status_callback =
                create_polling_status_callback(search_result_for_mbti, "mbti");

            // Start social and MBTI requests in parallel
            // Status callbacks will be used for background polling updates
            let social_future = make_request_with_payment::<SocialData>(
                &api_url_clone,
                &social_endpoint,
                None,
                wallet_account_clone.as_ref(),
                None,
                social_status_callback, // Pass callback for background polling updates
            );

            let mbti_future = make_request_with_payment::<MbtiProfile>(
                &api_url_clone,
                &mbti_endpoint,
                None,
                wallet_account_clone.as_ref(),
                None,
                mbti_status_callback, // Pass callback for background polling updates
            );

            // Wait for both results in parallel
            let (social_result, mbti_result) = join(social_future, mbti_future).await;

            // Immediately update pending_jobs if we got pending/processing status
            // This ensures UI shows status right away, even before polling completes
            if let Err(ref social_err) = social_result {
                if let Some((status, job_key, message)) =
                    parse_job_status_error(social_err, format!("social:{}", profile.fid))
                {
                    update_pending_job(&search_result, "social", status, job_key, message);
                }
            }

            if let Err(ref mbti_err) = mbti_result {
                if let Some((status, job_key, message)) =
                    parse_job_status_error(mbti_err, format!("mbti:{}", profile.fid))
                {
                    update_pending_job(&search_result, "mbti", status, job_key, message);
                }
            }

            // Process results and collect pending jobs
            let (social_data, social_pending) = process_analysis_result(
                social_result,
                "social",
                profile.fid,
                "Social analysis is still processing. You can come back later to check the results.",
            );

            let (mbti_data, mbti_pending) = process_analysis_result(
                mbti_result,
                "mbti",
                profile.fid,
                "MBTI analysis is still processing. You can come back later to check the results.",
            );

            // Get existing pending jobs (set above if we got pending/processing status) and merge with new ones
            let mut pending_jobs = search_result
                .as_ref()
                .and_then(|r| r.pending_jobs.clone())
                .unwrap_or_default();

            // If data was successfully loaded, remove the corresponding job
            if social_data.is_some() {
                pending_jobs.retain(|j| j.job_type != "social");
            } else if let Some(new_job) = social_pending {
                // Update or add social job
                if let Some(existing_job) = pending_jobs.iter_mut().find(|j| j.job_type == "social")
                {
                    // Update existing job with latest status from process_analysis_result
                    existing_job.status = new_job.status.clone();
                    existing_job.job_key = new_job.job_key.clone();
                    existing_job.message = new_job.message.clone();
                } else {
                    pending_jobs.push(new_job);
                }
            }

            if mbti_data.is_some() {
                pending_jobs.retain(|j| j.job_type != "mbti");
            } else if let Some(new_job) = mbti_pending {
                // Update or add mbti job
                if let Some(existing_job) = pending_jobs.iter_mut().find(|j| j.job_type == "mbti") {
                    // Update existing job with latest status from process_analysis_result
                    existing_job.status = new_job.status.clone();
                    existing_job.job_key = new_job.job_key.clone();
                    existing_job.message = new_job.message.clone();
                } else {
                    pending_jobs.push(new_job);
                }
            }

            // Update search result with all data, preserving pending jobs
            search_result.set(Some(SearchResult {
                profile,
                social: social_data,
                mbti: mbti_data,
                pending_jobs: if pending_jobs.is_empty() {
                    None
                } else {
                    Some(pending_jobs)
                },
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
            )
            .await;
        }
        Err(e) => {
            error_message.set(Some(e));
            loading_tasks.set(std::collections::HashSet::new());
        }
    }
}

// ============================================================================
// Handler Creators
// ============================================================================

/// Create wallet connect handler
pub fn create_wallet_connect_handler(
    wallet_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
) -> Callback<()> {
    Callback::from(move |_| {
        let wallet_error = wallet_error.clone();
        let wallet_account = wallet_account.clone();
        spawn_local(async move {
            web_sys::console::log_1(&"🔌 Connect button clicked".into());
            wallet_error.set(None); // Clear any previous errors

            match crate::wallet::connect().await {
                Ok(_) => {
                    web_sys::console::log_1(
                        &"✅ Wallet connect() succeeded, WalletConnect menu should be showing"
                            .into(),
                    );
                    wallet_error.set(None);

                    // Poll for account update (WalletConnect connection is async)
                    // The QRCodeModal will show wallet selection menu first
                    let mut attempts = 0;
                    while attempts < 30 {
                        if let Ok(account) = crate::wallet::get_account().await {
                            if account.is_connected {
                                web_sys::console::log_1(
                                    &format!("✅ Account connected: {:?}", account.address).into(),
                                );
                                wallet_account.set(Some(account));
                                return;
                            }
                        }
                        gloo_timers::future::TimeoutFuture::new(500).await;
                        attempts += 1;
                    }

                    // If still not connected, check one more time
                    if let Ok(account) = crate::wallet::get_account().await {
                        wallet_account.set(Some(account));
                    } else {
                        web_sys::console::log_1(
                            &"ℹ️ WalletConnect menu shown, waiting for user to select wallet..."
                                .into(),
                        );
                        // Don't set error - menu is showing, user needs to select wallet
                    }
                }
                Err(e) => {
                    web_sys::console::log_1(&format!("❌ Wallet connect() failed: {}", e).into());
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
#[allow(clippy::too_many_arguments)]
pub fn create_search_handler(
    search_input: UseStateHandle<String>,
    search_result: UseStateHandle<Option<SearchResult>>,
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
    error_message: UseStateHandle<Option<String>>,
    api_url: UseStateHandle<String>,
    chat_session: UseStateHandle<Option<ChatSession>>,
    chat_messages: UseStateHandle<Vec<ChatMessage>>,
    is_chat_loading: UseStateHandle<bool>,
    chat_error: UseStateHandle<Option<String>>,
    wallet_account: UseStateHandle<Option<WalletAccount>>,
    current_view: UseStateHandle<String>,
) -> Callback<()> {
    Callback::from(move |_| {
        let input = (*search_input).clone();
        if input.trim().is_empty() {
            return;
        }

        let search_result = search_result.clone();
        let loading_tasks = loading_tasks.clone();
        let error_message = error_message.clone();
        let api_url = (*api_url).clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();
        let wallet_account = wallet_account.clone();
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
                search_result,
                loading_tasks,
                error_message,
                api_url,
                chat_session,
                chat_messages,
                is_chat_loading,
                chat_error,
                wallet_account,
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

                match make_request_with_payment::<ChatMessageResponse>(
                    &api_url,
                    &chat_endpoint,
                    Some(request_json),
                    wallet_account.as_ref(),
                    None,
                    None,
                )
                .await
                {
                    Ok(chat_data) => {
                        let assistant_message = ChatMessage {
                            role: "assistant".to_string(),
                            content: chat_data.message,
                            timestamp: 0,
                        };
                        messages.push(assistant_message);
                        chat_messages.set(messages);
                        chat_error.set(None);
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

/// Create view switching handler
pub fn create_view_switch_handler(
    current_view: UseStateHandle<String>,
    search_result: UseStateHandle<Option<SearchResult>>,
) -> Callback<()> {
    Callback::from(move |_| {
        current_view.set("chat".to_string());
        if let Some(result) = (*search_result).as_ref() {
            let query = if let Some(username) = &result.profile.username {
                format!("@{}", username)
            } else {
                format!("{}", result.profile.fid)
            };
            crate::services::update_url_path(&query, "chat");
        }
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
    loading_tasks: UseStateHandle<std::collections::HashSet<String>>,
) -> Callback<()> {
    Callback::from(move |_| match (*current_view).as_str() {
        "profile" => {
            search_result.set(None);
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
            if let Some(result) = (*search_result).as_ref() {
                let query = if let Some(username) = &result.profile.username {
                    format!("@{}", username)
                } else {
                    format!("{}", result.profile.fid)
                };
                crate::services::update_url_path(&query, "profile");
            }
        }
        _ => {
            search_result.set(None);
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
