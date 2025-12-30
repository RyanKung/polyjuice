use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod analysis_loaders;
mod api;
mod chat;
mod components;
mod dashboard;
mod farcaster;
mod handlers;
mod headers;
mod icons;
mod models;
mod pages;
mod payment;
mod services;
mod share;
mod views;
mod wallet;

use analysis_loaders::*;
use chat::*;
use components::*;
use handlers::*;
use headers::*;
use models::*;
use pages::*;
use views::*;

#[function_component]
fn App() -> Html {
    // Wallet state
    let wallet_account = use_state(|| None::<wallet::WalletAccount>);
    let wallet_initialized = use_state(|| false);
    let wallet_error = use_state(|| None::<String>);
    let show_wallet_list = use_state(|| false);
    let discovered_wallets = use_state(Vec::<wallet::DiscoveredWallet>::new);

    // Track if we're in Farcaster Mini App environment
    let is_farcaster_env = use_state(|| false);
    let farcaster_context = use_state(|| None::<farcaster::MiniAppContext>);

    // Tab navigation state
    let active_tab = use_state(|| "search".to_string()); // "profile", "search", or "about"

    // State management
    let search_input = use_state(String::new);
    let search_query = use_state(|| None::<String>); // Current search query
    let is_fid_search = use_state(|| false); // Whether current search is by FID
    let search_result = use_state(|| None::<SearchResult>); // Keep for backward compatibility with chat
    let loading_tasks = use_state(std::collections::HashSet::<String>::new); // Multiple loading tasks
    let error_message = use_state(|| None::<String>);
    let api_url = use_state(|| {
        // Get API URL from build-time environment variable, fallback to default
        let url = option_env!("SNAPRAG_API_URL")
            .unwrap_or("https://snaprag.0xbase.ai")
            .trim_end_matches('/')
            .to_string();

        web_sys::console::log_1(&format!("🌐 Using API Server: {}", url).into());
        url
    });

    // Chat state management
    let chat_session = use_state(|| None::<ChatSession>);
    let chat_message = use_state(String::new);
    let chat_messages = use_state(Vec::<ChatMessage>::new);
    let is_chat_loading = use_state(|| false);
    let chat_error = use_state(|| None::<String>);
    let current_view = use_state(|| "profile".to_string()); // "profile" or "chat"
    let show_annual_report = use_state(|| false); // Whether to show annual report
    let annual_report_fid = use_state(|| None::<i64>); // FID for annual report
    let show_annual_report_modal = use_state(|| false); // Whether to show annual report modal

    // Endpoint state management
    let endpoint_data = use_state(|| None::<EndpointData>);
    let is_endpoint_loading = use_state(|| false);
    let endpoint_error = use_state(|| None::<String>);
    let show_endpoint = use_state(|| false);
    let ping_results = use_state(Vec::<(String, Option<f64>)>::new);
    let selected_endpoint = use_state(|| None::<String>); // Currently selected endpoint
    let custom_endpoints = use_state(Vec::<String>::new); // Custom endpoints added by user
    let custom_url_input = use_state(String::new); // Input for custom URL
    let custom_endpoint_error = use_state(|| None::<String>); // Error message for custom endpoint
    let is_adding_endpoint = use_state(|| false); // Whether we're currently adding an endpoint

    // Initialize Farcaster Mini App SDK on mount
    // According to Farcaster docs: call sdk.actions.ready() when app is fully loaded
    {
        let is_farcaster_env = is_farcaster_env.clone();
        let farcaster_context = farcaster_context.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // Wait a bit for app to fully render
                gloo_timers::future::TimeoutFuture::new(100).await;

                // Check if we're in a Mini App environment using official method
                match farcaster::is_in_mini_app().await {
                    Ok(true) => {
                        web_sys::console::log_1(&"📱 Running in Farcaster Mini App".into());
                        is_farcaster_env.set(true);
                        // Call sdk.actions.ready() to hide loading screen and show content
                        // This must be called when app is fully loaded
                        if let Err(e) = farcaster::initialize().await {
                            web_sys::console::warn_1(
                                &format!("⚠️ Failed to call sdk.actions.ready(): {}", e).into(),
                            );
                        } else {
                            web_sys::console::log_1(
                                &"✅ sdk.actions.ready() called successfully".into(),
                            );
                            // Get context after ready() and store it
                            match farcaster::get_context().await {
                                Ok(context) => {
                                    // Validate: In Farcaster environment, user.fid must exist
                                    if let Some(user) = &context.user {
                                        if user.fid.is_none() {
                                            let error_msg = format!(
                                                "❌ CRITICAL: Farcaster user missing FID! username={:?}, display_name={:?}",
                                                user.username, user.display_name
                                            );
                                            web_sys::console::error_1(&error_msg.clone().into());
                                            // Still set context but log error
                                            farcaster_context.set(Some(context.clone()));
                                        } else {
                                            web_sys::console::log_1(
                                                &format!(
                                                    "👤 Farcaster user: {} (FID: {})",
                                                    user.username.as_deref().unwrap_or("unknown"),
                                                    user.fid.unwrap()
                                                )
                                                .into(),
                                            );
                                            farcaster_context.set(Some(context.clone()));
                                        }
                                    } else {
                                        web_sys::console::warn_1(
                                            &"⚠️ Farcaster context has no user".into(),
                                        );
                                        farcaster_context.set(Some(context.clone()));
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("❌ Failed to get Farcaster context: {}", e)
                                            .into(),
                                    );
                                }
                            }
                        }
                    }
                    Ok(false) => {
                        web_sys::console::log_1(
                            &"🌐 Running in regular browser (not a Mini App)".into(),
                        );
                        is_farcaster_env.set(false);
                        // In regular browser, we don't need to call sdk.ready()
                    }
                    Err(e) => {
                        web_sys::console::warn_1(
                            &format!("⚠️ Failed to check Mini App status: {}", e).into(),
                        );
                        is_farcaster_env.set(false);
                        // If SDK is not available, assume regular browser
                    }
                }
            });
            || ()
        });
    }

    // Initialize wallet on mount (only if not in Farcaster environment)
    {
        let wallet_initialized = wallet_initialized.clone();
        let wallet_account = wallet_account.clone();
        let wallet_error = wallet_error.clone();
        let is_farcaster_env = is_farcaster_env.clone();
        let api_url = api_url.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                // Don't initialize wallet discovery in Farcaster environment
                if *is_farcaster_env {
                    web_sys::console::log_1(
                        &"📱 Skipping wallet initialization in Farcaster environment".into(),
                    );
                    wallet_initialized.set(true); // Mark as initialized to avoid UI issues
                    return;
                }

                match wallet::initialize().await {
                    Ok(_) => {
                        wallet_initialized.set(true);

                        // Wait a bit for wallets to be discovered via EIP-6963
                        gloo_timers::future::TimeoutFuture::new(1000).await;

                        // Try to restore wallet connection from localStorage
                        let api_url_clone = (*api_url).clone();
                        let wallet_account_clone = wallet_account.clone();
                        if let Ok(Some((saved_uuid, saved_address))) =
                            wallet::load_wallet_from_storage()
                        {
                            web_sys::console::log_1(
                                &format!(
                                    "🔄 Attempting to restore wallet connection: {}",
                                    saved_uuid
                                )
                                .into(),
                            );

                            // Try to reconnect to the saved wallet
                            match wallet::connect_to_wallet(&saved_uuid).await {
                                Ok(_) => {
                                    // Wait a bit for connection to establish
                                    gloo_timers::future::TimeoutFuture::new(500).await;

                                    if let Ok(account) = wallet::get_account().await {
                                        if account.is_connected {
                                            // Verify the address matches
                                            if account.address.as_ref() == Some(&saved_address) {
                                                web_sys::console::log_1(&"✅ Wallet connection restored from localStorage".into());

                                                // Get FID for the connected address
                                                let api_url_for_fid = api_url_clone.clone();
                                                let wallet_account_for_fid =
                                                    wallet_account_clone.clone();
                                                let account_for_fid = account.clone();
                                                spawn_local(async move {
                                                    match wallet::get_fid_for_address(
                                                        &api_url_for_fid,
                                                        &saved_address,
                                                    )
                                                    .await
                                                    {
                                                        Ok(fid) => {
                                                            let mut updated_account =
                                                                account_for_fid;
                                                            updated_account.fid = fid;
                                                            wallet_account_for_fid
                                                                .set(Some(updated_account));
                                                        }
                                                        Err(_) => {
                                                            wallet_account_for_fid
                                                                .set(Some(account_for_fid));
                                                        }
                                                    }
                                                });
                                            } else {
                                                web_sys::console::log_1(&"⚠️ Wallet address mismatch, clearing saved connection".into());
                                                let _ = wallet::clear_wallet_from_storage();
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::log_1(
                                        &format!("⚠️ Failed to restore wallet connection: {}", e)
                                            .into(),
                                    );
                                    // Clear invalid saved connection
                                    let _ = wallet::clear_wallet_from_storage();
                                }
                            }
                        } else {
                            // No saved wallet, check if there's already a connected account
                            if let Ok(account) = wallet::get_account().await {
                                if account.is_connected {
                                    // Get FID for the connected address
                                    let api_url_clone = (*api_url).clone();
                                    let wallet_account_clone = wallet_account.clone();
                                    if let Some(address) = &account.address {
                                        let address_clone = address.clone();
                                        let account_clone = account.clone();
                                        spawn_local(async move {
                                            match wallet::get_fid_for_address(
                                                &api_url_clone,
                                                &address_clone,
                                            )
                                            .await
                                            {
                                                Ok(fid) => {
                                                    let mut updated_account = account_clone;
                                                    updated_account.fid = fid;
                                                    wallet_account_clone.set(Some(updated_account));
                                                }
                                                Err(_) => {
                                                    wallet_account_clone.set(Some(account_clone));
                                                }
                                            }
                                        });
                                    } else {
                                        wallet_account.set(Some(account));
                                    }
                                }
                            }
                        }
                        // Don't discover wallets here - wait for user to click Connect button
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

    // Restore state from URL path on mount and handle browser navigation
    // Clone annual_report_fid and show_annual_report early so they can be used both in use_effect_with and later
    let annual_report_fid_for_effect = annual_report_fid.clone();
    let show_annual_report_for_effect = show_annual_report.clone();

    {
        let search_input = search_input.clone();
        let search_query_state = search_query.clone();
        let is_fid_state = is_fid_search.clone();
        let loading_tasks = loading_tasks.clone();
        let error_message = error_message.clone();
        let api_url = api_url.clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();
        let wallet_account = wallet_account.clone();
        let current_view = current_view.clone();
        let annual_report_fid_for_restore = annual_report_fid_for_effect.clone();
        let show_annual_report_for_restore = show_annual_report_for_effect.clone();

        // Function to restore state from URL path
        let restore_from_path = {
            let search_input = search_input.clone();
            let search_query_state = search_query_state.clone();
            let is_fid_state = is_fid_state.clone();
            let loading_tasks = loading_tasks.clone();
            let error_message = error_message.clone();
            let api_url = api_url.clone();
            let chat_session = chat_session.clone();
            let chat_messages = chat_messages.clone();
            let is_chat_loading = is_chat_loading.clone();
            let chat_error = chat_error.clone();
            let wallet_account = wallet_account.clone();
            let current_view = current_view.clone();

            move |query: String, view: String| {
                web_sys::console::log_1(
                    &format!("📍 Restoring from URL path: {} (view: {})", query, view).into(),
                );

                // Set the search input
                let query_for_input = query.trim_start_matches('@').to_string();
                search_input.set(query_for_input.clone());

                // Determine if it's a FID or username
                let is_fid = query_for_input.parse::<u64>().is_ok();

                // Set the view (will be updated by perform_search, but set it here for immediate feedback)
                current_view.set(view.clone());

                // Use the shared perform_search function to restore state
                let search_query_state_clone = search_query_state.clone();
                let is_fid_state_clone = is_fid_state.clone();
                let loading_tasks_clone = loading_tasks.clone();
                let error_message_clone = error_message.clone();
                let api_url_clone = (*api_url).clone();
                let _chat_session_clone = chat_session.clone();
                let _chat_messages_clone = chat_messages.clone();
                let _is_chat_loading_clone = is_chat_loading.clone();
                let _chat_error_clone = chat_error.clone();
                let _wallet_account_clone = wallet_account.clone();
                let current_view_clone = current_view.clone();

                spawn_local(async move {
                    crate::handlers::perform_search(
                        query_for_input,
                        is_fid,
                        search_query_state_clone,
                        is_fid_state_clone,
                        loading_tasks_clone,
                        error_message_clone,
                        api_url_clone,
                        current_view_clone,
                    )
                    .await;
                });
            }
        };

        use_effect_with((), move |_| {
            // Check if there's a URL path to restore from on initial load
            if let Some((query, view)) = crate::services::get_url_path() {
                // Handle annual-report URL separately
                if view == "annual-report" {
                    if let Ok(fid) = query.parse::<i64>() {
                        annual_report_fid_for_restore.set(Some(fid));
                        show_annual_report_for_restore.set(true);
                    }
                } else {
                    restore_from_path(query, view);
                }
            }

            // Set up popstate listener for browser back/forward navigation
            let loading_tasks_for_popstate = loading_tasks.clone();
            let search_query_state = search_query_state.clone();
            let annual_report_fid_for_popstate = annual_report_fid_for_restore.clone();
            let show_annual_report_for_popstate = show_annual_report_for_restore.clone();
            crate::services::setup_popstate_listener(move |path| {
                if let Some((query, view)) = path {
                    // Handle annual-report URL separately
                    if view == "annual-report" {
                        if let Ok(fid) = query.parse::<i64>() {
                            annual_report_fid_for_popstate.set(Some(fid));
                            show_annual_report_for_popstate.set(true);
                        }
                    } else {
                        restore_from_path(query, view);
                    }
                } else {
                    // Returned to home page - clear all state
                    search_query_state.set(None);
                    search_input.set(String::new());
                    error_message.set(None);
                    chat_session.set(None);
                    chat_messages.set(Vec::new());
                    loading_tasks_for_popstate.set(std::collections::HashSet::new()); // Important: reset loading state
                    current_view.set("profile".to_string());
                    show_annual_report_for_popstate.set(false);
                    annual_report_fid_for_popstate.set(None);
                }
            });

            || ()
        });
    }

    // Create handlers
    let on_disconnect_wallet = create_wallet_disconnect_handler(wallet_account.clone());

    // Handler for showing wallet list
    let on_connect_wallet = {
        let show_wallet_list = show_wallet_list.clone();
        let discovered_wallets = discovered_wallets.clone();
        Callback::from(move |_| {
            let show_wallet_list = show_wallet_list.clone();
            let discovered_wallets = discovered_wallets.clone();
            spawn_local(async move {
                // Discover wallets
                match wallet::discover_wallets().await {
                    Ok(wallets) => {
                        web_sys::console::log_1(
                            &format!("✅ Discovered {} wallets", wallets.len()).into(),
                        );
                        discovered_wallets.set(wallets.clone());
                        show_wallet_list.set(true);
                    }
                    Err(e) => {
                        web_sys::console::log_1(
                            &format!("⚠️ Failed to discover wallets: {}", e).into(),
                        );
                        // Still show the modal even if discovery fails (might have cached wallets)
                        show_wallet_list.set(true);
                    }
                }
            });
        })
    };

    // Handler for closing wallet list
    let on_close_wallet_list = {
        let show_wallet_list = show_wallet_list.clone();
        Callback::from(move |_| {
            show_wallet_list.set(false);
        })
    };

    // Handler for selecting a wallet
    let on_select_wallet = create_wallet_select_handler(
        wallet_error.clone(),
        wallet_account.clone(),
        api_url.clone(),
    );

    let on_search = create_search_handler(
        search_input.clone(),
        search_query.clone(),
        is_fid_search.clone(),
        loading_tasks.clone(),
        error_message.clone(),
        api_url.clone(),
        current_view.clone(),
    );

    let on_keypress = create_search_keypress_handler(on_search.clone());
    let on_popular_fid = create_popular_fid_handler(search_input.clone(), on_search.clone());
    let on_switch_to_chat = create_view_switch_handler(
        current_view.clone(),
        search_query.clone(),
        is_fid_search.clone(),
        api_url.clone(),
        chat_session.clone(),
        chat_messages.clone(),
        is_chat_loading.clone(),
        chat_error.clone(),
        wallet_account.clone(),
    );
    let on_smart_back = create_smart_back_handler(
        current_view.clone(),
        search_query.clone(),
        is_fid_search.clone(),
        search_input.clone(),
        error_message.clone(),
        chat_session.clone(),
        chat_messages.clone(),
        loading_tasks.clone(),
    );

    let on_send_chat_message = create_chat_message_handler(
        chat_session.clone(),
        chat_message.clone(),
        chat_messages.clone(),
        is_chat_loading.clone(),
        chat_error.clone(),
        api_url.clone(),
        wallet_account.clone(),
    );

    let on_chat_keypress = create_chat_keypress_handler(on_send_chat_message.clone());
    let on_search_input_change = create_input_change_handler(search_input.clone());
    let on_chat_input_change = create_input_change_handler(chat_message.clone());

    let on_fetch_endpoints = create_endpoint_fetch_handler(
        endpoint_data.clone(),
        is_endpoint_loading.clone(),
        endpoint_error.clone(),
        ping_results.clone(),
    );

    let on_back_from_endpoint = {
        let show_endpoint = show_endpoint.clone();
        Callback::from(move |_| {
            show_endpoint.set(false);
        })
    };

    // Handler for selecting an endpoint
    let on_select_endpoint = {
        let api_url = api_url.clone();
        let show_endpoint = show_endpoint.clone();
        let selected_endpoint = selected_endpoint.clone();
        Callback::from(move |endpoint: String| {
            let endpoint_clone = endpoint.clone();
            api_url.set(endpoint_clone.clone().trim_end_matches('/').to_string());
            selected_endpoint.set(Some(endpoint_clone.clone()));
            show_endpoint.set(false);
            web_sys::console::log_1(&format!("✅ Selected endpoint: {}", &endpoint).into());
        })
    };

    // Handler for adding custom endpoint
    let on_add_custom_endpoint = {
        let custom_endpoints = custom_endpoints.clone();
        let custom_url_input = custom_url_input.clone();
        let ping_results = ping_results.clone();
        let custom_endpoint_error = custom_endpoint_error.clone();
        let is_adding_endpoint = is_adding_endpoint.clone();
        Callback::from(move |_| {
            let url = (*custom_url_input).clone().trim().to_string();
            if url.is_empty() {
                return;
            }

            // Validate URL format
            if !url.starts_with("http://") && !url.starts_with("https://") {
                custom_endpoint_error.set(Some(
                    "Invalid URL format. Must start with http:// or https://".to_string(),
                ));
                return;
            }

            let normalized_url = url.trim_end_matches('/').to_string();

            // Check if endpoint already exists
            let endpoints = (*custom_endpoints).clone();
            if endpoints.contains(&normalized_url) {
                custom_endpoint_error.set(Some("Endpoint already exists".to_string()));
                return;
            }

            // Clear previous error
            custom_endpoint_error.set(None);
            is_adding_endpoint.set(true);

            // Ping the endpoint first before adding
            let custom_endpoints_clone = custom_endpoints.clone();
            let custom_url_input_clone = custom_url_input.clone();
            let ping_results_clone = ping_results.clone();
            let custom_endpoint_error_clone = custom_endpoint_error.clone();
            let is_adding_endpoint_clone = is_adding_endpoint.clone();
            let normalized_url_for_ping = normalized_url.clone();
            let normalized_url_for_log = normalized_url.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Try to ping the endpoint
                match wallet::ping_endpoint_service(&normalized_url_for_ping).await {
                    Ok(latency) => {
                        // Ping successful, add the endpoint
                        let mut endpoints = (*custom_endpoints_clone).clone();
                        endpoints.push(normalized_url_for_ping.clone());
                        custom_endpoints_clone.set(endpoints);
                        custom_url_input_clone.set(String::new());

                        // Add ping result
                        let mut current_results = (*ping_results_clone).clone();
                        current_results.push((normalized_url_for_ping, Some(latency)));
                        ping_results_clone.set(current_results);

                        custom_endpoint_error_clone.set(None);
                        web_sys::console::log_1(
                            &format!("✅ Added custom endpoint: {}", &normalized_url_for_log)
                                .into(),
                        );
                    }
                    Err(e) => {
                        // Ping failed (likely CORS), don't add the endpoint
                        let error_msg = if e.contains("CORS") || e.contains("cors") {
                            "Cannot add endpoint: CORS policy blocked the request. The server must allow cross-origin requests from your origin.".to_string()
                        } else {
                            format!("Cannot add endpoint: Ping failed ({})", e)
                        };
                        custom_endpoint_error_clone.set(Some(error_msg));
                        web_sys::console::log_1(
                            &format!("❌ Failed to add endpoint: {}", &normalized_url_for_log)
                                .into(),
                        );
                    }
                }
                is_adding_endpoint_clone.set(false);
            });
        })
    };

    // Handler for custom URL input change
    let on_custom_url_input_change = create_input_change_handler(custom_url_input.clone());

    // Handler for tab change
    let on_tab_change = {
        let active_tab = active_tab.clone();
        Callback::from(move |tab: String| {
            active_tab.set(tab);
        })
    };

    // Show annual report modal when FID is available and user is on home page (search tab)
    // Only show on home page, close when user navigates away
    {
        let show_annual_report_modal = show_annual_report_modal.clone();
        let farcaster_context = farcaster_context.clone();
        let wallet_account = wallet_account.clone();
        let is_farcaster_env = is_farcaster_env.clone();
        let active_tab = active_tab.clone();
        let search_query = search_query.clone();
        let show_annual_report = show_annual_report.clone();
        let show_endpoint = show_endpoint.clone();
        
        use_effect_with(
            (
                (*farcaster_context).clone(),
                (*wallet_account).clone(),
                *is_farcaster_env,
                (*active_tab).clone(),
                (*search_query).clone(),
                *show_annual_report,
                *show_endpoint,
            ),
            move |(farcaster_context, wallet_account, is_farcaster_env, active_tab, search_query, show_annual_report, show_endpoint)| {
                // Check if we're on the home page (search tab, no search query, no annual report, no endpoint)
                let is_home_page = active_tab.as_str() == "search" 
                    && search_query.is_none() 
                    && !show_annual_report 
                    && !show_endpoint;
                
                if is_home_page {
                    // Check if we have a FID
                    let fid = if *is_farcaster_env {
                        farcaster_context
                            .as_ref()
                            .and_then(|ctx| ctx.user.as_ref())
                            .and_then(|user| user.fid)
                    } else {
                        wallet_account
                            .as_ref()
                            .and_then(|acc| acc.fid)
                    };

                    if fid.is_some() {
                        {
                            let show_modal = show_annual_report_modal.clone();
                            if !*show_modal {
                                // Show modal after a short delay to ensure page is loaded
                                spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(500).await;
                                    show_modal.set(true);
                                });
                            }
                        }
                    }
                } else {
                    // Not on home page, close modal if it's open
                    if *show_annual_report_modal {
                        show_annual_report_modal.set(false);
                    }
                }
            },
        );
    }

    // Handler for closing annual report modal
    let on_close_annual_report_modal = {
        let show_annual_report_modal = show_annual_report_modal.clone();
        Callback::from(move |_| {
            show_annual_report_modal.set(false);
        })
    };

    // Handler for claiming annual report (navigate to annual report page)
    let on_claim_annual_report = {
        let show_annual_report_modal = show_annual_report_modal.clone();
        let show_annual_report = show_annual_report.clone();
        let annual_report_fid = annual_report_fid.clone();
        let farcaster_context = farcaster_context.clone();
        let wallet_account = wallet_account.clone();
        let is_farcaster_env = is_farcaster_env.clone();
        Callback::from(move |_| {
            // Get FID from farcaster context or wallet account
            let is_farcaster = *is_farcaster_env;
            let fid = if is_farcaster {
                (*farcaster_context)
                    .as_ref()
                    .and_then(|ctx| ctx.user.as_ref())
                    .and_then(|user| user.fid)
            } else {
                (*wallet_account)
                    .as_ref()
                    .and_then(|acc| acc.fid)
            };

            if let Some(fid) = fid {
                show_annual_report_modal.set(false);
                annual_report_fid.set(Some(fid));
                show_annual_report.set(true);
                // Update URL to /annual-report/{fid}
                crate::services::update_annual_report_url(fid);
            }
        })
    };

    // Determine left action button based on current page state
    let left_action = if *show_annual_report {
        // Annual report page - show share button
        let current_url = web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default();
        let share_text = "Check out my Farcaster Annual Report on Polyjuice!".to_string();
        Some(html! {
            <share::ShareButton
                url={Some(current_url)}
                text={Some(share_text)}
                is_farcaster_env={*is_farcaster_env}
            />
        })
    } else if (*search_query).is_some() {
        // Results page - show back button and share button together
        let current_url = web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default();
        Some(html! {
            <div style="display: flex; align-items: center; gap: 8px;">
                <button class="back-button" onclick={on_smart_back.clone().reform(|_| ())} style="background: none; border: none; font-size: 24px; cursor: pointer; padding: 4px 8px; color: white;">
                    {icons::back_arrow()}
                </button>
                <share::ShareButton
                    url={Some(current_url)}
                    is_farcaster_env={*is_farcaster_env}
                />
            </div>
        })
    } else if (*active_tab).as_str() == "search" {
        // Search page - show share button
        Some(html! {
            <share::ShareButton
                is_farcaster_env={*is_farcaster_env}
            />
        })
    } else if (*active_tab).as_str() == "profile" {
        // Profile page - show share button
        let current_url = web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default();
        Some(html! {
            <share::ShareButton
                url={Some(current_url)}
                is_farcaster_env={*is_farcaster_env}
            />
        })
    } else if (*active_tab).as_str() == "about" {
        // About page - show share button
        Some(html! {
            <share::ShareButton
                is_farcaster_env={*is_farcaster_env}
            />
        })
    } else {
        // Other pages - show share button as default
        Some(html! {
            <share::ShareButton
                is_farcaster_env={*is_farcaster_env}
            />
        })
    };

    html! {
        <div class="app-container">
            <div class="content">
                // Global Header (inside content, inherits background)
                <Header
                    wallet_account={(*wallet_account).clone()}
                    wallet_initialized={*wallet_initialized}
                    wallet_error={(*wallet_error).clone()}
                    on_disconnect={on_disconnect_wallet.clone()}
                    on_connect={on_connect_wallet.clone()}
                    api_url={(*api_url).clone()}
                    left_action={left_action}
                    is_farcaster_env={*is_farcaster_env}
                    farcaster_context={(*farcaster_context).clone()}
                />
                // Main content
                <div>
                    // Endpoint View (show when show_endpoint is true, hides tabs)
                    if *show_endpoint {
                        <div class="endpoint-page">
                            <div class="back-to-search">
                                <button class="back-button" onclick={on_back_from_endpoint}>
                                    {icons::back_arrow()}
                                </button>
                            </div>
                            <EndpointView
                                endpoint_data={(*endpoint_data).clone()}
                                is_loading={*is_endpoint_loading}
                                error={(*endpoint_error).clone()}
                                ping_results={(*ping_results).clone()}
                                selected_endpoint={(*selected_endpoint).clone()}
                                on_select_endpoint={on_select_endpoint.clone()}
                                custom_endpoints={(*custom_endpoints).clone()}
                                custom_url_input={(*custom_url_input).clone()}
                                on_custom_url_input_change={on_custom_url_input_change.clone()}
                                on_add_custom_endpoint={on_add_custom_endpoint.clone()}
                                custom_endpoint_error={(*custom_endpoint_error).clone()}
                                is_adding_endpoint={*is_adding_endpoint}
                            />
                        </div>
                    } else {
                        // Main content area with tabs
                        <div class="main-content">
                            // Results Page (Profile + Chat cards) - shown when search_query exists
                            if let Some(query) = (*search_query).as_ref() {
                                <div class="results-page">

                                    // Profile Card (only show if current_view is "profile")
                                    if (*current_view).as_str() == "profile" {
                                        <ProfileLoader
                                            search_query={query.clone()}
                                            is_fid={*is_fid_search}
                                            api_url={(*api_url).clone()}
                                            wallet_account={(*wallet_account).clone()}
                                            on_profile_loaded={Callback::from({
                                                let search_result = search_result.clone();
                                                move |profile: ProfileData| {
                                                    // Update search_result when profile loads (for ChatView compatibility)
                                                    search_result.set(Some(SearchResult {
                                                        profile,
                                                        social: None,
                                                        mbti: None,
                                                        pending_jobs: None,
                                                    }));
                                                }
                                            })}
                                        />
                                    }

                                    // Chat Card (only show if current_view is "chat")
                                    if (*current_view).as_str() == "chat" {
                                        <ChatView
                                            chat_session={(*chat_session).clone()}
                                            chat_messages={(*chat_messages).clone()}
                                            chat_message={(*chat_message).clone()}
                                            is_chat_loading={*is_chat_loading}
                                            chat_error={(*chat_error).clone()}
                                            search_result={(*search_result).clone()}
                                            on_input_change={on_chat_input_change}
                                            on_keypress={on_chat_keypress}
                                            on_send_message={on_send_chat_message}
                                        />
                                    }

                                    // Floating Chat Button (only show on results page when profile is visible)
                                    if (*current_view).as_str() == "profile" {
                                        <FloatingChatButton on_switch_to_chat={on_switch_to_chat} />
                                    }
                                </div>
                            } else {
                                // Tab-based pages (only show when no search results)
                                {
                                    if *show_annual_report {
                                        if let Some(fid) = *annual_report_fid {
                                            // Generate share URL for annual report
                                            let share_url = web_sys::window()
                                                .and_then(|w| w.location().origin().ok())
                                                .map(|origin| format!("{}/annual-report/{}", origin, fid));

                                            // Get current user FID from farcaster context or wallet account
                                            let current_user_fid = if *is_farcaster_env {
                                                (*farcaster_context).as_ref()
                                                    .and_then(|ctx| ctx.user.as_ref())
                                                    .and_then(|user| user.fid)
                                            } else {
                                                (*wallet_account).as_ref()
                                                    .and_then(|acc| acc.fid)
                                            };

                                            html! {
                                                <div class="annual-report-container">
                                                    <AnnualReportPage
                                                        fid={fid}
                                                        api_url={(*api_url).clone()}
                                                        wallet_account={(*wallet_account).clone()}
                                                        is_farcaster_env={*is_farcaster_env}
                                                        share_url={share_url}
                                                        current_user_fid={current_user_fid}
                                                        farcaster_context={(*farcaster_context).clone()}
                                                    />
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <div class="error-container">
                                                    <p>{"No FID available for annual report"}</p>
                                                </div>
                                            }
                                        }
                                    } else if (*active_tab).as_str() == "profile" {
                                        html! {
                                            <ProfilePage
                                                wallet_account={(*wallet_account).clone()}
                                                api_url={(*api_url).clone()}
                                                is_farcaster_env={*is_farcaster_env}
                                                farcaster_context={(*farcaster_context).clone()}
                                                on_show_annual_report={Callback::from({
                                                    let show_annual_report_clone = show_annual_report.clone();
                                                    let annual_report_fid_clone = annual_report_fid.clone();
                                                    move |fid: i64| {
                                                        annual_report_fid_clone.set(Some(fid));
                                                        show_annual_report_clone.set(true);
                                                        // Update URL to /annual-report/{fid}
                                                        crate::services::update_annual_report_url(fid);
                                                    }
                                                })}
                                            />
                                        }
                                    } else if (*active_tab).as_str() == "search" {
                                        html! {
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

                                                </div>

                                                <div class="search-content">
                                                    <SearchBox
                                                        search_input={(*search_input).clone()}
                                                        is_loading={!(*loading_tasks).is_empty()}
                                                        on_input_change={on_search_input_change}
                                                        on_keypress={on_keypress}
                                                        on_search={on_search.clone()}
                                                    />

                                                    <MobileSearchButton
                                                        is_loading={!(*loading_tasks).is_empty()}
                                                        on_search={on_search.clone()}
                                                    />

                                                    <SearchSuggestions on_popular_fid={on_popular_fid} />

                                                    <ErrorMessage error={(*error_message).clone()} />
                                                </div>
                                            </div>
                                        }
                                    } else if (*active_tab).as_str() == "about" {
                                        html! {
                                            <AboutPage
                                                endpoint_data={(*endpoint_data).clone()}
                                                is_loading={*is_endpoint_loading}
                                                error={(*endpoint_error).clone()}
                                                ping_results={(*ping_results).clone()}
                                                selected_endpoint={(*selected_endpoint).clone()}
                                                on_select_endpoint={on_select_endpoint.clone()}
                                                custom_endpoints={(*custom_endpoints).clone()}
                                                custom_url_input={(*custom_url_input).clone()}
                                                on_custom_url_input_change={on_custom_url_input_change.clone()}
                                                on_add_custom_endpoint={on_add_custom_endpoint.clone()}
                                                custom_endpoint_error={(*custom_endpoint_error).clone()}
                                                is_adding_endpoint={*is_adding_endpoint}
                                                on_fetch_endpoints={on_fetch_endpoints.clone()}
                                            />
                                        }
                                    } else {
                                        html! {
                                            <div class="search-page">
                                                <p>{"Unknown tab"}</p>
                                            </div>
                                        }
                                    }
                                }
                            }
                        </div>

                        // Bottom Tab Navigation (only show when not in endpoint view, no search query, and not showing annual report)
                        if (*search_query).is_none() && !*show_annual_report {
                            <BottomTab active_tab={(*active_tab).clone()} on_tab_change={on_tab_change} />
                        }
                    }
                </div>
            </div>
            // Wallet List Modal
            if *show_wallet_list {
                <WalletList
                    wallets={discovered_wallets.to_vec()}
                    on_select_wallet={Callback::from({
                        let on_select_wallet = on_select_wallet.clone();
                        let on_close_wallet_list = on_close_wallet_list.clone();
                        move |uuid: String| {
                            on_close_wallet_list.emit(());
                            on_select_wallet.emit(uuid);
                        }
                    })}
                    on_close={on_close_wallet_list.clone()}
                />
            }
            // Annual Report Modal
            if *show_annual_report_modal {
                <AnnualReportModal
                    on_claim={on_claim_annual_report.clone()}
                    on_close={on_close_annual_report_modal.clone()}
                />
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
