use web_sys::InputEvent;
use yew::prelude::*;

use crate::wallet::DiscoveredWallet;
use crate::wallet::WalletAccount;

#[allow(dead_code)]
#[derive(Properties, PartialEq, Clone)]
pub struct WalletStatusProps {
    pub wallet_account: Option<WalletAccount>,
    pub wallet_initialized: bool,
    pub wallet_error: Option<String>,
    pub on_connect: Callback<()>,
    pub on_disconnect: Callback<()>,
}

#[derive(Properties, Clone)]
pub struct WalletListProps {
    pub wallets: Vec<DiscoveredWallet>,
    pub on_select_wallet: Callback<String>,
    pub on_close: Callback<()>,
}

impl PartialEq for WalletListProps {
    fn eq(&self, other: &Self) -> bool {
        // Compare wallets by their info (UUID), since provider (JsValue) cannot be compared
        self.wallets.len() == other.wallets.len()
            && self.wallets.iter().zip(other.wallets.iter()).all(|(a, b)| {
                a.info.uuid == b.info.uuid
                    && a.info.name == b.info.name
                    && a.info.icon == b.info.icon
            })
    }
}

/// Wallet status component
#[function_component]
pub fn WalletStatus(props: &WalletStatusProps) -> Html {
    if !props.wallet_initialized {
        return html! {};
    }

    // Show error message if there's an error
    let error_html = if let Some(error) = &props.wallet_error {
        html! {
            <div class="wallet-status disconnected" style="margin-bottom: 8px;">
                <span style="font-size: 12px;">{error}</span>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class="wallet-section">
            {error_html}
            {
                if let Some(account) = &props.wallet_account {
                    if account.is_connected {
                        // Display FID if available, otherwise display address
                        let display_text = if let Some(fid) = account.fid {
                            format!("FID: {}", fid)
                        } else if let Some(ref address) = account.address {
                            format!("{}...{}",
                                &address[..4.min(address.len())],
                                &address[address.len().saturating_sub(4)..]
                            )
                        } else {
                            "Connected".to_string()
                        };

                        html! {
                            <div class="wallet-status connected" onclick={props.on_disconnect.clone().reform(|_| ())} style="cursor: pointer;">
                                <span style="font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace; font-size: 14px;">
                                    {display_text}
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

/// Wallet list component
#[function_component]
pub fn WalletList(props: &WalletListProps) -> Html {
    html! {
        <div class="wallet-list-overlay" onclick={props.on_close.clone().reform(|_| ())} style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); z-index: 10000; display: flex; align-items: center; justify-content: center;">
            <div class="wallet-list-modal" onclick={Callback::from(|e: web_sys::MouseEvent| e.stop_propagation())} style="background: white; border-radius: 16px; padding: 24px; max-width: 400px; width: 90%; max-height: 80vh; overflow-y: auto; color: #000;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                    <h2 style="margin: 0; font-size: 20px; font-weight: 600;">{"Connect Wallet"}</h2>
                    <button onclick={props.on_close.clone().reform(|_| ())} style="background: none; border: none; font-size: 24px; cursor: pointer; padding: 0; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center;">{"✕"}</button>
                </div>
                <p style="margin: 0 0 16px 0; color: #666; font-size: 14px;">{"Please select a wallet to connect"}</p>
                <div class="wallet-list">
                    {
                        if props.wallets.is_empty() {
                            html! {
                                <p style="color: #999; text-align: center; padding: 20px;">{"No wallets found. Please install a wallet extension like MetaMask."}</p>
                            }
                        } else {
                            html! {
                                <>
                                    {
                                        for props.wallets.iter().map(|wallet| {
                                            let uuid = wallet.info.uuid.clone();
                                            let name = wallet.info.name.clone();
                                            let icon = wallet.info.icon.clone();

                                            html! {
                                                <button
                                                    class="wallet-list-item"
                                                    onclick={props.on_select_wallet.clone().reform(move |_| uuid.clone())}
                                                    style="width: 100%; padding: 12px 16px; margin-bottom: 8px; border: 1px solid #e0e0e0; border-radius: 8px; background: white; cursor: pointer; display: flex; align-items: center; gap: 12px; transition: background-color 0.2s;"
                                                >
                                                    {
                                                        if !icon.is_empty() {
                                                            html! {
                                                                <img src={icon.clone()} alt={name.clone()} style="width: 32px; height: 32px; border-radius: 4px;" />
                                                            }
                                                        } else {
                                                            html! {
                                                                <div style="width: 32px; height: 32px; border-radius: 4px; background: #f0f0f0; display: flex; align-items: center; justify-content: center; font-size: 18px;">{"🔷"}</div>
                                                            }
                                                        }
                                                    }
                                                    <span style="font-size: 16px; font-weight: 500; flex: 1; text-align: left;">{name}</span>
                                                </button>
                                            }
                                        })
                                    }
                                </>
                            }
                        }
                    }
                </div>
            </div>
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Properties, PartialEq, Clone)]
pub struct LinkButtonProps {
    pub on_click: Callback<()>,
}

/// Link button component (top left)
#[function_component]
pub fn LinkButton(props: &LinkButtonProps) -> Html {
    html! {
        <div class="link-button-container">
            <button class="link-button" onclick={props.on_click.clone().reform(|_| ())}>
                {"🔗"}
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct BottomTabProps {
    pub active_tab: String,
    pub on_tab_change: Callback<String>,
}

/// Bottom tab navigation component
#[function_component]
pub fn BottomTab(props: &BottomTabProps) -> Html {
    let active_tab = props.active_tab.clone();

    html! {
        <div class="bottom-tab-bar">
            <button
                class={if active_tab == "profile" { "tab-item active" } else { "tab-item" }}
                onclick={props.on_tab_change.clone().reform(|_| "profile".to_string())}
            >
                <span class="tab-icon">{"👤"}</span>
                <span class="tab-label">{"Profile"}</span>
            </button>
            <button
                class={if active_tab == "search" { "tab-item active" } else { "tab-item" }}
                onclick={props.on_tab_change.clone().reform(|_| "search".to_string())}
            >
                <span class="tab-icon">{"🔍"}</span>
                <span class="tab-label">{"Search"}</span>
            </button>
            <button
                class={if active_tab == "about" { "tab-item active" } else { "tab-item" }}
                onclick={props.on_tab_change.clone().reform(|_| "about".to_string())}
            >
                <span class="tab-icon">{"ℹ️"}</span>
                <span class="tab-label">{"About"}</span>
            </button>
        </div>
    }
}
