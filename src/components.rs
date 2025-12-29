use web_sys::InputEvent;
use yew::prelude::*;

use crate::icons;
use crate::wallet::DiscoveredWallet;

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

#[derive(Properties, PartialEq, Clone)]
pub struct FloatingChatButtonProps {
    pub on_switch_to_chat: Callback<()>,
}

/// Floating chat button component
#[function_component]
pub fn FloatingChatButton(props: &FloatingChatButtonProps) -> Html {
    html! {
        <div class="floating-chat-button" onclick={props.on_switch_to_chat.clone().reform(|_| ())}>
            {icons::chat()}
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
                <span class="tab-icon">{icons::user()}</span>
                <span class="tab-label">{"Profile"}</span>
            </button>
            <button
                class={if active_tab == "search" { "tab-item active" } else { "tab-item" }}
                onclick={props.on_tab_change.clone().reform(|_| "search".to_string())}
            >
                <span class="tab-icon">{icons::search()}</span>
                <span class="tab-label">{"Search"}</span>
            </button>
            <button
                class={if active_tab == "about" { "tab-item active" } else { "tab-item" }}
                onclick={props.on_tab_change.clone().reform(|_| "about".to_string())}
            >
                <span class="tab-icon">{icons::info()}</span>
                <span class="tab-label">{"About"}</span>
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct AnnualReportModalProps {
    pub on_claim: Callback<()>,
    pub on_close: Callback<()>,
}

/// Annual Report Modal component
#[function_component]
pub fn AnnualReportModal(props: &AnnualReportModalProps) -> Html {
    html! {
        <div 
            class="annual-report-modal-overlay" 
            onclick={props.on_close.clone().reform(|_| ())} 
            style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0, 0, 0, 0.5); z-index: 10001; display: flex; align-items: center; justify-content: center;"
        >
            <div 
                class="annual-report-modal" 
                onclick={Callback::from(|e: web_sys::MouseEvent| e.stop_propagation())} 
                style="background: white; border-radius: 16px; padding: 0; max-width: 400px; width: 90%; position: relative; overflow: hidden;"
            >
                // Close button (X) in top-left
                <button 
                    onclick={props.on_close.clone().reform(|_| ())} 
                    style="position: absolute; top: 12px; left: 12px; background: rgba(0, 0, 0, 0.5); border: none; color: white; font-size: 20px; cursor: pointer; padding: 4px 10px; border-radius: 50%; width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; z-index: 10; transition: background-color 0.2s;"
                >
                    {"✕"}
                </button>
                
                // Preview image
                <div style="width: 100%;">
                    <img 
                        src="/imgs/preview.png" 
                        alt="Annual Report Preview" 
                        style="width: 100%; height: auto; display: block;"
                    />
                </div>
                
                // Purple button
                <div style="padding: 24px;">
                    <button 
                        onclick={props.on_claim.clone().reform(|_| ())} 
                        style="width: 100%; padding: 14px 24px; background: #8B5CF6; border: none; border-radius: 8px; color: white; font-size: 16px; font-weight: 600; cursor: pointer; transition: background-color 0.2s;"
                    >
                        {"Claim Your Annual Report"}
                    </button>
                </div>
            </div>
        </div>
    }
}
