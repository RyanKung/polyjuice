use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::farcaster;
use crate::models::ProfileData;
use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderProps {
    pub wallet_account: Option<WalletAccount>,
    pub wallet_initialized: bool,
    pub wallet_error: Option<String>,
    pub on_disconnect: Callback<()>,
    pub on_connect: Callback<()>,
    pub api_url: String,
    #[prop_or_default]
    pub left_action: Option<Html>,
    pub is_farcaster_env: bool,
    pub farcaster_context: Option<farcaster::MiniAppContext>,
}

/// Global header component
/// Shows Connect button when not connected, or user info (avatar, username, FID) when connected
#[function_component]
pub fn Header(props: &HeaderProps) -> Html {
    let user_profile = use_state(|| None::<ProfileData>);
    let is_loading_profile = use_state(|| false);

    // Fetch user profile when FID is available
    {
        let wallet_account = props.wallet_account.clone();
        let api_url = props.api_url.clone();
        let user_profile = user_profile.clone();
        let is_loading_profile = is_loading_profile.clone();

        use_effect_with(wallet_account.clone(), move |account| {
            if let Some(account) = account {
                if account.is_connected {
                    // Try to fetch profile if we have an address
                    // We can fetch by address even if FID is not yet available
                    if let Some(address) = &account.address {
                        // Check if we should fetch:
                        // 1. If we have FID, check if profile FID matches
                        // 2. If we don't have FID yet, check if we already have a profile for this address
                        let should_fetch = if let Some(fid) = account.fid {
                            // If we have FID, only fetch if profile doesn't match
                            user_profile.as_ref().map(|p| p.fid != fid).unwrap_or(true)
                        } else {
                            // If no FID yet, check if we have any profile loaded
                            // If we have a profile, keep it; otherwise try to fetch
                            user_profile.is_none()
                        };

                        if should_fetch && !*is_loading_profile {
                            is_loading_profile.set(true);
                            let api_url_clone = api_url.clone();
                            let user_profile_clone = user_profile.clone();
                            let is_loading_profile_clone = is_loading_profile.clone();
                                let address_clone = address.clone();
                            let current_fid = account.fid;

                                spawn_local(async move {
                                    match crate::wallet::get_profile_for_address(
                                        &api_url_clone,
                                        &address_clone,
                                    )
                                    .await
                                    {
                                        Ok(profile) => {
                                        if let Some(profile) = profile {
                                            // Only set profile if it matches current FID (if available)
                                            // or if we don't have FID yet (profile might have FID)
                                            if let Some(fid) = current_fid {
                                                if profile.fid == fid {
                                                    user_profile_clone.set(Some(profile));
                                                } else {
                                                    web_sys::console::log_1(
                                                        &"⚠️ Profile FID mismatch, skipping".into(),
                                                    );
                                                }
                                            } else {
                                                // No FID yet, accept any profile we get
                                                user_profile_clone.set(Some(profile));
                                            }
                                        } else {
                                            web_sys::console::log_1(
                                                &"ℹ️ No profile found for this address".into(),
                                            );
                                        }
                                        is_loading_profile_clone.set(false);
                                        }
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("⚠️ Failed to fetch profile: {}", e)
                                                    .into(),
                                            );
                                            is_loading_profile_clone.set(false);
                                        }
                                    }
                                });
                        }
                    } else {
                        // No address, clear profile
                        user_profile.set(None);
                    }
                } else {
                    // Not connected, clear profile
                    user_profile.set(None);
                }
            } else {
                // No account, clear profile
                user_profile.set(None);
            }
            || ()
        });
    }

    html! {
        <header class="global-header" style="position: sticky; top: 0; z-index: 1000; background: rgba(255, 255, 255, 0.1); backdrop-filter: blur(10px); -webkit-backdrop-filter: blur(10px); border-bottom: 1px solid rgba(255, 255, 255, 0.2); padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; min-height: 60px; box-sizing: border-box;">
            <div class="header-left" style="display: flex; align-items: center; gap: 12px;">
                {
                    if let Some(action) = &props.left_action {
                        action.clone()
                    } else {
                        html! {}
                    }
                }
            </div>
            <div class="header-right" style="display: flex; flex-direction: column; align-items: flex-end; gap: 4px;">
                {
                    // Show error message if there's an error
                    if let Some(error) = &props.wallet_error {
                        html! {
                            <div style="font-size: 12px; color: #ff3b30; margin-bottom: 4px; max-width: 300px; text-align: right;">
                                {error}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
                {
                    // If in Farcaster environment, show Farcaster user info directly
                    if props.is_farcaster_env {
                        if let Some(context) = &props.farcaster_context {
                            if let Some(user) = &context.user {
                                html! {
                                    <div class="user-info" style="display: flex; align-items: center; gap: 12px;">
                                        // Avatar with circular border
                                        <div class="avatar-container" style="width: 40px; height: 40px; border-radius: 50%; border: 2px solid #007AFF; padding: 2px; display: flex; align-items: center; justify-content: center; background: white;">
                                            {
                                                if let Some(pfp_url) = &user.pfp_url {
                                                    html! {
                                                        <img
                                                            src={pfp_url.clone()}
                                                            alt="Avatar"
                                                            style="width: 100%; height: 100%; border-radius: 50%; object-fit: cover;"
                                                        />
                                                    }
                                                } else {
                                                    html! {
                                                        <div style="width: 100%; height: 100%; border-radius: 50%; background: #f0f0f0; display: flex; align-items: center; justify-content: center; font-size: 20px;">
                                                            {"👤"}
                                                        </div>
                                                    }
                                                }
                                            }
                                        </div>
                                        // Username and FID
                                        <div class="user-details" style="display: flex; flex-direction: column; align-items: flex-start;">
                                            {
                                                if let Some(username) = &user.username {
                                                    html! {
                                                        <span style="font-size: 14px; font-weight: 500; color: #333;">
                                                            {format!("@{}", username)}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(fid) = user.fid {
                                                    html! {
                                                        <span style="font-size: 12px; color: #666;">
                                                            {format!("FID: {}", fid)}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        } else {
                            html! {}
                        }
                    } else if !props.wallet_initialized {
                        html! {}
                    } else if let Some(account) = &props.wallet_account {
                        if account.is_connected {
                            // Show user info if we have profile data
                            if let Some(profile) = (*user_profile).as_ref() {
                                html! {
                                    <div class="user-info" style="display: flex; align-items: center; gap: 12px;">
                                        // Avatar with circular border
                                        <div class="avatar-container" style="width: 40px; height: 40px; border-radius: 50%; border: 2px solid #007AFF; padding: 2px; display: flex; align-items: center; justify-content: center; background: white;">
                                            {
                                                if let Some(pfp_url) = &profile.pfp_url {
                                                    html! {
                                                        <img
                                                            src={pfp_url.clone()}
                                                            alt="Avatar"
                                                            style="width: 100%; height: 100%; border-radius: 50%; object-fit: cover;"
                                                        />
                                                    }
                                                } else {
                                                    html! {
                                                        <div style="width: 100%; height: 100%; border-radius: 50%; background: #f0f0f0; display: flex; align-items: center; justify-content: center; font-size: 20px;">
                                                            {"👤"}
                                                        </div>
                                                    }
                                                }
                                            }
                                        </div>
                                        // Username and FID
                                        <div class="user-details" style="display: flex; flex-direction: column; align-items: flex-start;">
                                            {
                                                if let Some(username) = &profile.username {
                                                    html! {
                                                        <span style="font-size: 14px; font-weight: 500; color: #333;">
                                                            {format!("@{}", username)}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            <span style="font-size: 12px; color: #666;">
                                                {format!("FID: {}", profile.fid)}
                                            </span>
                                        </div>
                                        // Disconnect button
                                        <button
                                            class="disconnect-btn"
                                            style="background: none; border: none; font-size: 18px; cursor: pointer; padding: 4px 8px; color: #666;"
                                            onclick={props.on_disconnect.clone().reform(|_| ())}
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            } else if *is_loading_profile {
                                // Loading state - show address with loading indicator
                                html! {
                                    <div class="user-info" style="display: flex; align-items: center; gap: 12px;">
                                        <div class="avatar-container" style="width: 40px; height: 40px; border-radius: 50%; border: 2px solid #007AFF; padding: 2px; display: flex; align-items: center; justify-content: center; background: white;">
                                            <div style="width: 100%; height: 100%; border-radius: 50%; background: #f0f0f0; display: flex; align-items: center; justify-content: center; font-size: 20px;">
                                                {"👤"}
                                            </div>
                                        </div>
                                        <div class="user-details" style="display: flex; flex-direction: column; align-items: flex-start; gap: 4px;">
                                            {
                                                if let Some(address) = &account.address {
                                                    html! {
                                                        <span style="font-size: 14px; font-weight: 500; color: #333; font-family: 'SF Mono', Monaco, monospace;">
                                                            {format!("{}...{}", &address[..4.min(address.len())], &address[address.len().saturating_sub(4)..])}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            <div style="display: flex; align-items: center; gap: 6px;">
                                                <div style="width: 12px; height: 12px; border: 2px solid #f3f3f3; border-top: 2px solid #007AFF; border-radius: 50%; animation: spin 1s linear infinite;"></div>
                                                <span style="font-size: 12px; color: #666;">{"Loading profile..."}</span>
                                            </div>
                                        </div>
                                        <button
                                            class="disconnect-btn"
                                            style="background: none; border: none; font-size: 18px; cursor: pointer; padding: 4px 8px; color: #666;"
                                            onclick={props.on_disconnect.clone().reform(|_| ())}
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            } else {
                                // Connected but no profile yet (fallback to address)
                                html! {
                                    <div class="user-info" style="display: flex; align-items: center; gap: 12px;">
                                        <div class="avatar-container" style="width: 40px; height: 40px; border-radius: 50%; border: 2px solid #007AFF; padding: 2px; display: flex; align-items: center; justify-content: center; background: white;">
                                            <div style="width: 100%; height: 100%; border-radius: 50%; background: #f0f0f0; display: flex; align-items: center; justify-content: center; font-size: 20px;">
                                                {"👤"}
                                            </div>
                                        </div>
                                        <div class="user-details" style="display: flex; flex-direction: column; align-items: flex-start;">
                                            {
                                                if let Some(address) = &account.address {
                                                    html! {
                                                        <span style="font-size: 14px; font-weight: 500; color: #333; font-family: 'SF Mono', Monaco, monospace;">
                                                            {format!("{}...{}", &address[..4.min(address.len())], &address[address.len().saturating_sub(4)..])}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                            {
                                                if let Some(fid) = account.fid {
                                                    html! {
                                                        <span style="font-size: 12px; color: #666;">
                                                            {format!("FID: {}", fid)}
                                                        </span>
                                                    }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </div>
                                        <button
                                            class="disconnect-btn"
                                            style="background: none; border: none; font-size: 18px; cursor: pointer; padding: 4px 8px; color: #666;"
                                            onclick={props.on_disconnect.clone().reform(|_| ())}
                                        >
                                            {"✕"}
                                        </button>
                                    </div>
                                }
                            }
                        } else {
                            // Not connected - show connect button
                            html! {
                                <button
                                    class="wallet-button"
                                    onclick={props.on_connect.clone().reform(|_| ())}
                                    style="background: rgba(102, 126, 234, 0.8); border: 1px solid rgba(102, 126, 234, 0.9); border-radius: 12px; color: #ffffff; font-size: 13px; font-weight: 600; padding: 8px 16px; cursor: pointer; backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3), inset 0 1px 0 rgba(255,255,255,0.2); text-shadow: 0 1px 2px rgba(0,0,0,0.2); white-space: nowrap;"
                                >
                                    {"Connect"}
                                </button>
                            }
                        }
                    } else {
                        // No account - show connect button
                        html! {
                            <button
                                class="wallet-button"
                                onclick={props.on_connect.clone().reform(|_| ())}
                                style="background: rgba(102, 126, 234, 0.8); border: 1px solid rgba(102, 126, 234, 0.9); border-radius: 12px; color: #ffffff; font-size: 13px; font-weight: 600; padding: 8px 16px; cursor: pointer; backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); box-shadow: 0 4px 15px rgba(102, 126, 234, 0.3), inset 0 1px 0 rgba(255,255,255,0.2); text-shadow: 0 1px 2px rgba(0,0,0,0.2); white-space: nowrap;"
                            >
                                {"Connect"}
                            </button>
                        }
                    }
                }
            </div>
        </header>
    }
}
