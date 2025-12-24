use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::dashboard::Dashboard;
use crate::farcaster;
use crate::models::ProfileData;
use crate::services::{create_profile_endpoint, make_request_with_payment};
use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct ProfilePageProps {
    pub wallet_account: Option<WalletAccount>,
    pub api_url: String,
    pub on_show_annual_report: Option<Callback<i64>>,
    pub is_farcaster_env: bool,
    pub farcaster_context: Option<farcaster::MiniAppContext>,
}

/// Profile page component (shows current user's profile in Farcaster environment or from wallet)
#[function_component]
pub fn ProfilePage(props: &ProfilePageProps) -> Html {
    let user_context = use_state(|| props.farcaster_context.clone());
    let wallet_profile = use_state(|| None::<ProfileData>);
    let is_loading = use_state(|| false);
    let is_loading_wallet_profile = use_state(|| false);

    // Update user_context when farcaster_context prop changes
    {
        let user_context = user_context.clone();
        let farcaster_context = props.farcaster_context.clone();
        use_effect_with(farcaster_context.clone(), move |_| {
            user_context.set(farcaster_context.clone());
            || ()
        });
    }

    // Fallback: Try to get Farcaster Mini App context if not provided via props
    {
        let user_context = user_context.clone();
        let is_loading = is_loading.clone();
        let is_farcaster_env = props.is_farcaster_env;
        use_effect_with((), move |_| {
            if is_farcaster_env && user_context.is_none() {
                is_loading.set(true);
                let user_context = user_context.clone();
                let is_loading = is_loading.clone();
                spawn_local(async move {
                    match farcaster::get_context().await {
                        Ok(context) => {
                            user_context.set(Some(context));
                            is_loading.set(false);
                        }
                        Err(_) => {
                            // Not in Farcaster Mini App, that's okay - will check wallet
                            is_loading.set(false);
                        }
                    }
                });
            }
            || ()
        });
    }

    // If wallet is connected and has FID, fetch profile
    {
        let wallet_account = props.wallet_account.clone();
        let api_url = props.api_url.clone();
        let wallet_profile = wallet_profile.clone();
        let is_loading_wallet_profile = is_loading_wallet_profile.clone();

        use_effect_with(wallet_account.clone(), move |account| {
            if let Some(account) = account {
                if account.is_connected {
                    // Try to fetch profile if we have FID or address
                    if let Some(fid) = account.fid {
                        // Check if we already have this profile loaded
                        let should_fetch = wallet_profile
                            .as_ref()
                            .map(|p| p.fid != fid)
                            .unwrap_or(true);

                        if should_fetch && !*is_loading_wallet_profile {
                            is_loading_wallet_profile.set(true);
                            let api_url_clone = api_url.clone();
                            let wallet_profile_clone = wallet_profile.clone();
                            let is_loading_wallet_profile_clone = is_loading_wallet_profile.clone();
                            let wallet_account_clone = account.clone();

                            spawn_local(async move {
                                let fid_str = fid.to_string();
                                let endpoint = create_profile_endpoint(&fid_str, true);

                                match make_request_with_payment::<ProfileData>(
                                    &api_url_clone,
                                    &endpoint,
                                    None,
                                    Some(&wallet_account_clone),
                                    None,
                                    None,
                                )
                                .await
                                {
                                    Ok(profile) => {
                                        wallet_profile_clone.set(Some(profile));
                                        is_loading_wallet_profile_clone.set(false);
                                    }
                                    Err(e) => {
                                        web_sys::console::warn_1(
                                            &format!("⚠️ Failed to fetch wallet profile: {}", e)
                                                .into(),
                                        );
                                        is_loading_wallet_profile_clone.set(false);
                                    }
                                }
                            });
                        }
                    } else if let Some(address) = &account.address {
                        // No FID yet, try to fetch profile by address
                        let should_fetch = wallet_profile.is_none();

                        if should_fetch && !*is_loading_wallet_profile {
                            is_loading_wallet_profile.set(true);
                            let api_url_clone = api_url.clone();
                            let wallet_profile_clone = wallet_profile.clone();
                            let is_loading_wallet_profile_clone = is_loading_wallet_profile.clone();
                            let address_clone = address.clone();

                            spawn_local(async move {
                                match crate::wallet::get_profile_for_address(
                                    &api_url_clone,
                                    &address_clone,
                                )
                                .await
                                {
                                    Ok(profile) => {
                                        if let Some(profile) = profile {
                                            wallet_profile_clone.set(Some(profile));
                                        }
                                        is_loading_wallet_profile_clone.set(false);
                                    }
                                    Err(e) => {
                                        web_sys::console::warn_1(
                                            &format!("⚠️ Failed to fetch profile by address: {}", e)
                                                .into(),
                                        );
                                        is_loading_wallet_profile_clone.set(false);
                                    }
                                }
                            });
                        }
                    }
                }
            }
            || ()
        });
    }

    // Calculate user_fid for Farcaster context if available
    let farcaster_user_fid = if let Some(context) = &*user_context {
        if let Some(user) = &context.user {
            user.fid.or_else(|| {
                props.wallet_account.as_ref().and_then(|acc| acc.fid)
            })
        } else {
            None
        }
    } else {
        None
    };

    html! {
        <div class="profile-page">
            <div class="profile-page-content">
                if *is_loading || *is_loading_wallet_profile {
                    <div class="loading-container">
                        <div class="skeleton-spinner"></div>
                        <p>{"Loading profile..."}</p>
                    </div>
                } else if let Some(context) = &*user_context {
                    // Farcaster Mini App context
                    if let Some(user) = &context.user {
                        <>
                            <div class="profile-card">
                                <div class="profile-header">
                                    if let Some(pfp_url) = &user.pfp_url {
                                        <img src={pfp_url.clone()} alt="Profile" class="profile-avatar" />
                                    } else {
                                        <div class="profile-avatar-placeholder">{"👤"}</div>
                                    }
                                    <div class="profile-info">
                                        <h2>{user.get_display_name()}</h2>
                                        if let Some(username) = &user.username {
                                            <p class="profile-username">{format!("@{}", username)}</p>
                                        }
                                        if let Some(fid) = farcaster_user_fid {
                                            <p class="profile-fid">{format!("FID: {}", fid)}</p>
                                        } else {
                                            <p class="profile-fid" style="color: #ff3b30;">{"⚠️ FID not available"}</p>
                                        }
                                    </div>
                                </div>
                            </div>

                            // Annual Report Button
                            if let Some(fid) = farcaster_user_fid {
                                if let Some(on_show) = &props.on_show_annual_report {
                                    <div class="annual-report-button-container">
                                        <button
                                            class="annual-report-button"
                                            onclick={on_show.clone().reform(move |_| fid)}
                                        >
                                            {"🎉 View 2025 Annual Report"}
                                        </button>
                                    </div>
                                }
                            } else {
                                <div class="annual-report-button-container">
                                    <div style="padding: 12px; background: rgba(255, 59, 48, 0.1); border-radius: 8px; color: #ff3b30; text-align: center;">
                                        <p style="margin: 0; font-size: 14px;">{"⚠️ Unable to load annual report: FID not available"}</p>
                                    </div>
                                </div>
                            }

                            // Dashboard component
                            if let Some(fid) = farcaster_user_fid {
                                <Dashboard
                                    fid={fid}
                                    api_url={props.api_url.clone()}
                                />
                            }
                        </>
                    } else {
                        <div class="profile-empty">
                            <p>{"No user profile available"}</p>
                            <p class="profile-hint">{"This page shows your Farcaster profile when running in a Mini App"}</p>
                        </div>
                    }
                } else if let Some(profile) = &*wallet_profile {
                    // Wallet profile (connected wallet with FID)
                    <>
                        <div class="profile-card">
                            <div class="profile-header">
                                if let Some(pfp_url) = &profile.pfp_url {
                                    <img src={pfp_url.clone()} alt="Profile" class="profile-avatar" />
                                } else {
                                    <div class="profile-avatar-placeholder">{"👤"}</div>
                                }
                                <div class="profile-info">
                                    <h2>{profile.get_display_name()}</h2>
                                    if let Some(username) = &profile.username {
                                        <p class="profile-username">{format!("@{}", username)}</p>
                                    }
                                    <p class="profile-fid">{format!("FID: {}", profile.fid)}</p>
                                </div>
                            </div>
                        </div>

                        // Annual Report Button
                        {if let Some(on_show) = &props.on_show_annual_report {
                            let profile_fid = profile.fid;
                            html! {
                                <div class="annual-report-button-container">
                                    <button
                                        class="annual-report-button"
                                        onclick={on_show.clone().reform(move |_| profile_fid)}
                                    >
                                        {"🎉 View 2025 Annual Report"}
                                    </button>
                                </div>
                            }
                        } else {
                            html! {}
                        }}

                        // Dashboard component
                        <Dashboard
                            fid={profile.fid}
                            api_url={props.api_url.clone()}
                        />
                    </>
                } else {
                    // No profile available
                    <div class="profile-empty">
                        <p>{"Not running in Farcaster Mini App"}</p>
                        <p class="profile-hint">{"Please open with Farcaster or connect a Farcaster-linked wallet"}</p>
                    </div>
                }
            </div>
        </div>
    }
}

