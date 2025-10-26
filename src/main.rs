use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod api;
mod components;
mod handlers;
mod models;
mod payment;
mod services;
mod views;
mod wallet;

use components::*;
use handlers::*;
use models::*;
use views::*;

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
    let chat_message = use_state(|| String::new());
    let chat_messages = use_state(|| Vec::<ChatMessage>::new());
    let is_chat_loading = use_state(|| false);
    let chat_error = use_state(|| None::<String>);
    let current_view = use_state(|| "profile".to_string()); // "profile" or "chat"

    // Endpoint state management
    let endpoint_data = use_state(|| None::<EndpointData>);
    let is_endpoint_loading = use_state(|| false);
    let endpoint_error = use_state(|| None::<String>);
    let show_endpoint = use_state(|| false);
    let ping_results = use_state(|| Vec::<(String, Option<f64>)>::new());
    let selected_endpoint = use_state(|| None::<String>); // Currently selected endpoint

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

    // Create handlers
    let on_connect_wallet =
        create_wallet_connect_handler(wallet_error.clone(), wallet_account.clone());
    let on_disconnect_wallet = create_wallet_disconnect_handler(wallet_account.clone());

    let on_search = create_search_handler(
        search_input.clone(),
        search_result.clone(),
        is_loading.clone(),
        error_message.clone(),
        api_url.clone(),
        chat_session.clone(),
        chat_messages.clone(),
        is_chat_loading.clone(),
        chat_error.clone(),
        wallet_account.clone(),
    );

    let on_keypress = create_search_keypress_handler(on_search.clone());
    let on_popular_fid = create_popular_fid_handler(search_input.clone(), on_search.clone());
    let on_switch_to_chat = create_view_switch_handler(current_view.clone());
    let on_smart_back = create_smart_back_handler(
        current_view.clone(),
        search_result.clone(),
        search_input.clone(),
        error_message.clone(),
        chat_session.clone(),
        chat_messages.clone(),
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

    let on_show_endpoint = {
        let show_endpoint = show_endpoint.clone();
        let endpoint_data = endpoint_data.clone();
        let is_endpoint_loading = is_endpoint_loading.clone();
        Callback::from(move |_| {
            show_endpoint.set(true);
            // Fetch endpoints if not already loaded
            if endpoint_data.is_none() && !*is_endpoint_loading {
                on_fetch_endpoints.emit(());
            }
        })
    };

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

    html! {
        <div class="app-container">
            // Link Button - Only visible in search page (top left)
            if !*show_endpoint && (*search_result).is_none() {
                <LinkButton on_click={on_show_endpoint} />
            }

            // Endpoint View (show when show_endpoint is true)
            if *show_endpoint {
                <div class="endpoint-page">
                    <div class="back-to-search">
                        <button class="back-button" onclick={on_back_from_endpoint}>
                            {"←"}
                        </button>
                    </div>
                    <EndpointView
                        endpoint_data={(*endpoint_data).clone()}
                        is_loading={*is_endpoint_loading}
                        error={(*endpoint_error).clone()}
                        ping_results={(*ping_results).clone()}
                        selected_endpoint={(*selected_endpoint).clone()}
                        on_select_endpoint={on_select_endpoint.clone()}
                    />
                </div>
            } else {
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

                        <WalletStatus
                            wallet_account={(*wallet_account).clone()}
                            wallet_initialized={*wallet_initialized}
                            wallet_error={(*wallet_error).clone()}
                            on_connect={on_connect_wallet}
                            on_disconnect={on_disconnect_wallet}
                        />
                    </div>

                    <div class="search-content">
                        <SearchBox
                            search_input={(*search_input).clone()}
                            is_loading={*is_loading}
                            on_input_change={on_search_input_change}
                            on_keypress={on_keypress}
                            on_search={on_search.clone()}
                        />

                        <MobileSearchButton
                            is_loading={*is_loading}
                            on_search={on_search.clone()}
                        />

                        <SearchSuggestions on_popular_fid={on_popular_fid} />

                        <ErrorMessage error={(*error_message).clone()} />

                        <LoadingOverlay is_loading={*is_loading} text={"Searching...".to_string()} />
                    </div>
                </div>
            }

            // Results Page (Profile + Chat cards)
            if (*search_result).is_some() {
                <div class="results-page">
                    <BackButton on_back={on_smart_back} />

                    // Profile Card (only show if current_view is "profile")
                    if (*current_view).as_str() == "profile" {
                        <ProfileView search_result={(*search_result).clone()} />
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
                </div>

                // Floating Chat Button (only show on results page when profile is visible)
                if (*search_result).is_some() && (*current_view).as_str() == "profile" {
                    <FloatingChatButton on_switch_to_chat={on_switch_to_chat} />
                }
            }
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
