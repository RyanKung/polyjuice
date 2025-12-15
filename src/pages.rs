use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::dashboard::Dashboard;
use crate::farcaster;
use crate::models::ProfileData;
use crate::services::create_profile_endpoint;
use crate::services::make_request_with_payment;
use crate::wallet::WalletAccount;

#[derive(Properties, PartialEq, Clone)]
pub struct ProfilePageProps {
    pub wallet_account: Option<WalletAccount>,
    pub api_url: String,
}

/// Profile page component (shows current user's profile in Farcaster environment or from wallet)
#[function_component]
pub fn ProfilePage(props: &ProfilePageProps) -> Html {
    let user_context = use_state(|| None::<farcaster::MiniAppContext>);
    let wallet_profile = use_state(|| None::<ProfileData>);
    let is_loading = use_state(|| true);
    let is_loading_wallet_profile = use_state(|| false);

    // Try to get Farcaster Mini App context
    {
        let user_context = user_context.clone();
        let is_loading = is_loading.clone();
        use_effect_with((), move |_| {
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
                    }
                }
            }
            || ()
        });
    }

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
                                        <h2>{user.display_name.as_deref().unwrap_or("Unknown User")}</h2>
                                        if let Some(username) = &user.username {
                                            <p class="profile-username">{format!("@{}", username)}</p>
                                        }
                                        if let Some(fid) = user.fid {
                                            <p class="profile-fid">{format!("FID: {}", fid)}</p>
                                        }
                                    </div>
                                </div>
                            </div>

                            // Dashboard component
                            if let Some(fid) = user.fid {
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
                                    <h2>{profile.display_name.as_deref().unwrap_or("Unknown User")}</h2>
                                    if let Some(username) = &profile.username {
                                        <p class="profile-username">{format!("@{}", username)}</p>
                                    }
                                    <p class="profile-fid">{format!("FID: {}", profile.fid)}</p>
                                </div>
                            </div>
                        </div>

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

/// About page component
#[function_component]
pub fn AboutPage() -> Html {
    html! {
        <div class="about-page">
            <div class="about-page-content">
                <div class="about-header">
                    <div class="logo-image">
                        <img src="/logo.png" alt="Polyjuice Logo" />
                    </div>
                    <h1>{"polyjuice"}</h1>
                    <p class="tagline">{"Discover & Chat with Farcaster Users"}</p>
                </div>

                <div class="about-section">
                    <h2>{"About"}</h2>
                    <p>{"Polyjuice is a powerful tool for discovering and interacting with Farcaster users. Search for users by FID or username, view their profiles, and chat with them using AI-powered conversations."}</p>
                </div>

                <div class="about-section">
                    <h2>{"Features"}</h2>
                    <ul class="about-features">
                        <li>{"🔍 Search Farcaster users by FID or username"}</li>
                        <li>{"👤 View detailed user profiles and analytics"}</li>
                        <li>{"💬 Chat with users using AI"}</li>
                        <li>{"📊 MBTI personality analysis"}</li>
                        <li>{"🌐 Social network insights"}</li>
                    </ul>
                </div>

                <div class="about-section">
                    <h2>{"Built With"}</h2>
                    <ul class="about-tech">
                        <li>{"Rust + WebAssembly"}</li>
                        <li>{"Yew Framework"}</li>
                        <li>{"Farcaster Mini App SDK"}</li>
                        <li>{"EIP-1193 & EIP-6963"}</li>
                    </ul>
                </div>
            </div>
        </div>
    }
}
