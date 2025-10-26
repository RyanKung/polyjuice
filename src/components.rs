use yew::prelude::*;
use web_sys::InputEvent;

use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct WalletStatusProps {
    pub wallet_account: Option<WalletAccount>,
    pub wallet_initialized: bool,
    pub wallet_error: Option<String>,
    pub on_connect: Callback<()>,
    pub on_disconnect: Callback<()>,
}

/// Wallet status component
#[function_component]
pub fn WalletStatus(props: &WalletStatusProps) -> Html {
    if !props.wallet_initialized || props.wallet_error.is_some() {
        return html! {};
    }

    html! {
        <div class="wallet-section">
            {
                if let Some(account) = &props.wallet_account {
                    if account.is_connected {
                        html! {
                            <div class="wallet-status connected" onclick={props.on_disconnect.clone().reform(|_| ())} style="cursor: pointer;">
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
                            <button onclick={props.on_connect.clone().reform(|_| ())} class="wallet-button">
                                {"Connect"}
                            </button>
                        }
                    }
                } else {
                    html! {
                        <button onclick={props.on_connect.clone().reform(|_| ())} class="wallet-button">
                            {"Connect"}
                        </button>
                    }
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SearchBoxProps {
    pub search_input: String,
    pub is_loading: bool,
    pub on_input_change: Callback<InputEvent>,
    pub on_keypress: Callback<web_sys::KeyboardEvent>,
    pub on_search: Callback<()>,
}

/// Search box component
#[function_component]
pub fn SearchBox(props: &SearchBoxProps) -> Html {
    html! {
        <div class="search-box">
            <input 
                type="text" 
                class="search-input"
                placeholder="give me a fid/username"
                value={props.search_input.clone()}
                oninput={props.on_input_change.clone()}
                onkeypress={props.on_keypress.clone()}
            />
            <button 
                class="search-button"
                onclick={props.on_search.clone().reform(|_| ())}
                disabled={props.is_loading}
            >
                {"⌕"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct MobileSearchButtonProps {
    pub is_loading: bool,
    pub on_search: Callback<()>,
}

/// Mobile search button component
#[function_component]
pub fn MobileSearchButton(props: &MobileSearchButtonProps) -> Html {
    html! {
        <div class="mobile-search-button">
            <button 
                class="mobile-search-btn"
                onclick={props.on_search.clone().reform(|_| ())}
                disabled={props.is_loading}
            >
                {"Search"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SearchSuggestionsProps {
    pub on_popular_fid: Callback<String>,
}

/// Search suggestions component
#[function_component]
pub fn SearchSuggestions(props: &SearchSuggestionsProps) -> Html {
    html! {
        <div class="search-suggestions">
            <p class="suggestions-title">{"Popular:"}</p>
            <div class="suggestion-tags">
                <button class="suggestion-tag" onclick={props.on_popular_fid.clone().reform(|_| "vitalik.eth".to_string())}>{"@vitalik.eth"}</button>
                <button class="suggestion-tag" onclick={props.on_popular_fid.clone().reform(|_| "jesse.base.eth".to_string())}>{"@jesse.base.eth"}</button>
                <button class="suggestion-tag" onclick={props.on_popular_fid.clone().reform(|_| "ryankung.base.eth".to_string())}>{"@ryankung.base.eth"}</button>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ErrorMessageProps {
    pub error: Option<String>,
}

/// Error message component
#[function_component]
pub fn ErrorMessage(props: &ErrorMessageProps) -> Html {
    if let Some(error) = &props.error {
        html! {
            <div class="error-message">
                <p>{error}</p>
            </div>
        }
    } else {
        html! {}
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct LoadingOverlayProps {
    pub is_loading: bool,
    pub text: String,
}

/// Loading overlay component
#[function_component]
pub fn LoadingOverlay(props: &LoadingOverlayProps) -> Html {
    if props.is_loading {
        html! {
            <div class="loading-overlay">
                <div class="loading-content">
                    <div class="loading-spinner"></div>
                    <div class="loading-text">{&props.text}</div>
                </div>
            </div>
        }
    } else {
        html! {}
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct BackButtonProps {
    pub on_back: Callback<()>,
}

/// Back button component
#[function_component]
pub fn BackButton(props: &BackButtonProps) -> Html {
    html! {
        <div class="back-to-search">
            <button class="back-button" onclick={props.on_back.clone().reform(|_| ())}>
                {"←"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct FloatingChatButtonProps {
    pub on_switch_to_chat: Callback<()>,
}

/// Floating chat button component
#[function_component]
pub fn FloatingChatButton(props: &FloatingChatButtonProps) -> Html {
    html! {
        <div class="floating-chat-button" onclick={props.on_switch_to_chat.clone().reform(|_| ())}>
            {"💭"}
        </div>
    }
}