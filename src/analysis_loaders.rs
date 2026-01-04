use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::dashboard::Dashboard;
use crate::models::*;
use crate::services::*;
use crate::wallet::WalletAccount;

// ============================================================================
// Profile Loader Component
// ============================================================================

#[derive(Properties, PartialEq, Clone)]
pub struct ProfileLoaderProps {
    pub search_query: String,
    pub is_fid: bool,
    pub api_url: String,
    pub wallet_account: Option<WalletAccount>,
    pub on_profile_loaded: Option<Callback<ProfileData>>, // Optional callback to notify when profile is loaded
}

/// Independent Profile Loader Component
/// Manages its own state for loading and displaying profile, social, and MBTI data
#[function_component]
pub fn ProfileLoader(props: &ProfileLoaderProps) -> Html {
    let profile_data = use_state(|| None::<ProfileData>);
    let loading = use_state(|| true);
    let error = use_state(|| None::<String>);
    let loaded_query = use_state(|| None::<String>); // Track which query we've loaded

    // Clone values for use in both effect and render
    let api_url_for_render = props.api_url.clone();
    let wallet_account_for_render = props.wallet_account.clone();
    let on_profile_loaded_callback = props.on_profile_loaded.clone();

    // Load profile data when component mounts or props change
    {
        let profile_data = profile_data.clone();
        let loading = loading.clone();
        let error = error.clone();
        let loaded_query_clone = loaded_query.clone();
        let search_query_for_effect = props.search_query.clone();
        let is_fid_for_effect = props.is_fid;
        let api_url_for_effect = props.api_url.clone();
        let wallet_account_for_effect = props.wallet_account.clone();

        use_effect_with(
            (
                props.search_query.clone(),
                props.is_fid,
                props.api_url.clone(),
            ),
            move |_| {
                // Check if we need to load - only load if query changed or we don't have data
                let current_query = search_query_for_effect.clone();
                let query_changed = (*loaded_query_clone).as_ref() != Some(&current_query);
                let needs_load = (*profile_data).is_none() || query_changed;

                if needs_load {
                    loading.set(true);
                    error.set(None);

                    let profile_data_clone = profile_data.clone();
                    let loading_clone = loading.clone();
                    let error_clone = error.clone();
                    let loaded_query_for_spawn = loaded_query_clone.clone();
                    let search_query_clone = search_query_for_effect.clone();
                    let api_url_clone = api_url_for_effect.clone();
                    let wallet_account_clone = wallet_account_for_effect.clone();
                    let on_profile_loaded = on_profile_loaded_callback.clone();

                    spawn_local(async move {
                        let endpoint =
                            create_profile_endpoint(&search_query_clone, is_fid_for_effect);

                        let result = make_request_with_payment::<ProfileData>(
                            &api_url_clone,
                            &endpoint,
                            None,
                            wallet_account_clone.as_ref(),
                            None,
                            None,
                        )
                        .await;

                        match result {
                            Ok(data) => {
                                web_sys::console::log_1(
                                    &format!(
                                        "✅ Profile data loaded for query: {}",
                                        search_query_clone
                                    )
                                    .into(),
                                );
                                // Update loaded query first to prevent reloading
                                loaded_query_for_spawn.set(Some(search_query_clone.clone()));
                                // Set profile data first, then set loading to false
                                // This ensures data is available when component re-renders
                                profile_data_clone.set(Some(data.clone()));
                                loading_clone.set(false);
                                // Notify parent component if callback is provided
                                if let Some(callback) = on_profile_loaded {
                                    callback.emit(data);
                                }
                            }
                            Err(e) => {
                                web_sys::console::log_1(
                                    &format!("❌ Profile data error: {}", e).into(),
                                );
                                error_clone.set(Some(e));
                                loading_clone.set(false);
                            }
                        }
                    });
                } else {
                    // We already have data, make sure loading is false
                    loading.set(false);
                }
            },
        );
    }

    // Render based on state
    if let Some(profile) = (*profile_data).as_ref() {
        html! {
            <div class="card profile-card">
                <div class="card-content">
                    <div class="profile-info">
                        <div class="profile-picture">
                            if let Some(pfp_url) = &profile.pfp_url {
                                <img src={pfp_url.clone()} alt="Profile" />
                            } else {
                                <div class="profile-picture-placeholder">
                                    {"👤"}
                                </div>
                            }
                        </div>

                        <div class="user-details">
                            <h2>{profile.get_display_name()}</h2>
                            if let Some(username) = &profile.username {
                                <p class="username">{"@"}{username}</p>
                            }
                            <div class="fid-badge">{"FID: "}{profile.fid}</div>

                            if let Some(bio) = &profile.bio {
                                <p class="bio">{bio}</p>
                            }
                        </div>
                    </div>

                    // MBTI Analysis Loader - manages its own state
                    <MbtiAnalysisLoader
                        fid={profile.fid}
                        username={profile.username.clone()}
                        api_url={api_url_for_render.clone()}
                        wallet_account={wallet_account_for_render.clone()}
                    />

                    // Social Analysis Loader - manages its own state
                    <SocialAnalysisLoader
                        fid={profile.fid}
                        username={profile.username.clone()}
                        api_url={api_url_for_render.clone()}
                        wallet_account={wallet_account_for_render.clone()}
                    />
                </div>
            </div>
        }
    } else if let Some(err) = (*error).as_ref() {
        html! {
            <div class="card profile-card">
                <div class="card-content">
                    <div class="error-message">
                        <p>{"Error loading profile: "}{err}</p>
                    </div>
                </div>
            </div>
        }
    } else if *loading {
        html! {
            <div class="card profile-card">
                <div class="card-content">
                    <div class="loading-container">
                        <div class="skeleton-spinner"></div>
                        <p>{"Loading profile..."}</p>
                    </div>
                </div>
            </div>
        }
    } else {
        html! {
            <div class="card profile-card">
                <div class="card-content">
                    <div class="error-message">
                        <p>{"Profile not found"}</p>
                    </div>
                </div>
            </div>
        }
    }
}

// ============================================================================
// Social Analysis Loader Component
// ============================================================================

#[derive(Properties, PartialEq, Clone)]
pub struct SocialAnalysisLoaderProps {
    pub fid: i64,
    pub username: Option<String>,
    pub api_url: String,
    pub wallet_account: Option<WalletAccount>,
}

/// Independent Social Analysis Loader Component
/// Manages its own state for loading and displaying social data
#[function_component]
pub fn SocialAnalysisLoader(props: &SocialAnalysisLoaderProps) -> Html {
    let social_data = use_state(|| None::<SocialData>);
    let pending_job = use_state(|| None::<PendingJob>);
    let loading = use_state(|| true);

    let fid = props.fid;
    let username = props.username.clone();
    let api_url = props.api_url.clone();
    let wallet_account = props.wallet_account.clone();

    // Load social data when component mounts or props change
    {
        let social_data = social_data.clone();
        let pending_job = pending_job.clone();
        let loading = loading.clone();

        use_effect_with((fid, username.clone(), api_url.clone()), move |_| {
            // Check if we already have data for this fid - only reload if fid changed
            let needs_load = social_data.as_ref().map(|d| d.fid != fid).unwrap_or(true);
            
            if !needs_load {
                // We already have data for this fid, make sure loading is false
                loading.set(false);
                return;
            }
            
            loading.set(true);

            let social_data_clone = social_data.clone();
            let pending_job_clone = pending_job.clone();
            let loading_clone = loading.clone();
            let fid_clone = fid;
            let username_clone = username.clone();
            let api_url_clone = api_url.clone();
            let wallet_account_clone = wallet_account.clone();

            spawn_local(async move {
                // Determine if we should use fid or username
                // Use fid if username is None or if username is a pure number (should be treated as fid)
                let is_fid = username_clone.is_none() || 
                    username_clone.as_ref().map(|u| u.trim().parse::<u64>().is_ok()).unwrap_or(false);
                let query = if is_fid {
                    fid_clone.to_string()
                } else {
                    username_clone.unwrap_or_else(|| fid_clone.to_string())
                };
                let endpoint = create_social_endpoint(&query, is_fid);

                let result = make_request_with_payment::<SocialData>(
                    &api_url_clone,
                    &endpoint,
                    None,
                    wallet_account_clone.as_ref(),
                    None,
                    None,
                )
                .await;

                match result {
                    Ok(data) => {
                        web_sys::console::log_1(
                            &format!("✅ Social data loaded for FID={}", fid_clone).into(),
                        );
                        social_data_clone.set(Some(data));
                        pending_job_clone.set(None);
                        loading_clone.set(false);
                    }
                    Err(e) => {
                        // Check for JOB_STATUS error format
                        if let Some((status, job_key, message)) =
                            parse_job_status_error(&e, format!("social:{}", fid_clone))
                        {
                            web_sys::console::log_1(
                                &format!(
                                    "⏳ Social analysis status: {} (job_key: {})",
                                    status, job_key
                                )
                                .into(),
                            );
                            let new_job = PendingJob {
                                job_key: job_key.clone(),
                                job_type: "social".to_string(),
                                status: Some(status.clone()),
                                started_at: Some(js_sys::Date::now() as u64),
                                message: Some(message.clone()),
                            };
                            pending_job_clone.set(Some(new_job));
                            loading_clone.set(false);
                            
                            // Start polling to check if data is ready
                            // Clone values for polling task
                            let social_data_for_poll = social_data_clone.clone();
                            let pending_job_for_poll = pending_job_clone.clone();
                            let loading_for_poll = loading_clone.clone();
                            let api_url_for_poll = api_url_clone.clone();
                            let endpoint_for_poll = endpoint.clone();
                            let wallet_account_for_poll = wallet_account_clone.clone();
                            let job_key_for_poll = job_key.clone();
                            
                            spawn_local(async move {
                                // Poll every 4 seconds to check if data is ready
                                let mut attempt = 0;
                                let max_attempts = 200; // Max ~13 minutes
                                
                                loop {
                                    if attempt >= max_attempts {
                                        break;
                                    }
                                    
                                    // Wait before polling (except first attempt)
                                    if attempt > 0 {
                                        let wait_time = 4000u64; // 4 seconds
                                        let promise = js_sys::Promise::new(&mut |resolve, _| {
                                            let window = web_sys::window().unwrap();
                                            window
                                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &resolve,
                                                    wait_time as i32,
                                                )
                                                .unwrap();
                                        });
                                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                    }
                                    
                                    attempt += 1;
                                    
                                    // Try to fetch data
                                    match make_request_with_payment::<SocialData>(
                                        &api_url_for_poll,
                                        &endpoint_for_poll,
                                        None,
                                        wallet_account_for_poll.as_ref(),
                                        None,
                                        None,
                                    )
                                    .await
                                    {
                                        Ok(data) => {
                                            web_sys::console::log_1(
                                                &format!("✅ Social data loaded via polling for FID={}", fid_clone).into(),
                                            );
                                            social_data_for_poll.set(Some(data));
                                            pending_job_for_poll.set(None);
                                            loading_for_poll.set(false);
                                            break;
                                        }
                                        Err(e) => {
                                            // Check if still pending/processing
                                            if let Some((new_status, _, _)) =
                                                parse_job_status_error(&e, job_key_for_poll.clone())
                                            {
                                                if new_status == "pending" || new_status == "processing" {
                                                    // Still processing, continue polling
                                                    continue;
                                                } else {
                                                    // Status changed but not completed, stop polling
                                                    break;
                                                }
                                            } else {
                                                // Not a JOB_STATUS error, might be a real error
                                                // Continue polling in case it's temporary
                                                continue;
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            web_sys::console::log_1(&format!("❌ Social data error: {}", e).into());
                            loading_clone.set(false);
                        }
                    }
                }
            });
        });
    }

    // Render based on state
    if let Some(data) = (*social_data).as_ref() {
        html! {
            <SocialAnalysisView social={data.clone()} />
        }
    } else if let Some(job) = (*pending_job).as_ref() {
        let message = job
            .message
            .as_deref()
            .unwrap_or("Loading social analysis...");
        html! {
            <SocialSkeletonView message={message.to_string()} />
        }
    } else if *loading {
        html! {
            <SocialSkeletonView message={"Loading social analysis...".to_string()} />
        }
    } else {
        html! {
            <SocialSkeletonView message={"Social analysis not available".to_string()} />
        }
    }
}

// ============================================================================
// MBTI Analysis Loader Component
// ============================================================================

#[derive(Properties, PartialEq, Clone)]
pub struct MbtiAnalysisLoaderProps {
    pub fid: i64,
    pub username: Option<String>,
    pub api_url: String,
    pub wallet_account: Option<WalletAccount>,
}

/// Independent MBTI Analysis Loader Component
/// Manages its own state for loading and displaying MBTI data
#[function_component]
pub fn MbtiAnalysisLoader(props: &MbtiAnalysisLoaderProps) -> Html {
    let mbti_data = use_state(|| None::<MbtiProfile>);
    let pending_job = use_state(|| None::<PendingJob>);
    let loading = use_state(|| true);

    let fid = props.fid;
    let username = props.username.clone();
    let api_url_for_effect = props.api_url.clone();
    let api_url = props.api_url.clone();
    let wallet_account = props.wallet_account.clone();

    // Load MBTI data when component mounts or props change
    {
        let mbti_data = mbti_data.clone();
        let pending_job = pending_job.clone();
        let loading = loading.clone();

        use_effect_with((fid, username.clone(), api_url_for_effect.clone()), move |_| {
            // Check if we already have data for this fid - only reload if fid changed
            let needs_load = mbti_data.as_ref().map(|d| d.fid != fid).unwrap_or(true);
            
            if !needs_load {
                // We already have data for this fid, make sure loading is false
                loading.set(false);
                return;
            }
            
            loading.set(true);

            let mbti_data_clone = mbti_data.clone();
            let pending_job_clone = pending_job.clone();
            let loading_clone = loading.clone();
            let fid_clone = fid;
            let username_clone = username.clone();
            let api_url_clone = api_url_for_effect.clone();
            let wallet_account_clone = wallet_account.clone();

            spawn_local(async move {
                // Determine if we should use fid or username
                // Use fid if username is None or if username is a pure number (should be treated as fid)
                let is_fid = username_clone.is_none() || 
                    username_clone.as_ref().map(|u| u.trim().parse::<u64>().is_ok()).unwrap_or(false);
                let query = if is_fid {
                    fid_clone.to_string()
                } else {
                    username_clone.unwrap_or_else(|| fid_clone.to_string())
                };
                let endpoint = create_mbti_endpoint(&query, is_fid);

                let result = make_request_with_payment::<MbtiProfile>(
                    &api_url_clone,
                    &endpoint,
                    None,
                    wallet_account_clone.as_ref(),
                    None,
                    None,
                )
                .await;

                match result {
                    Ok(data) => {
                        web_sys::console::log_1(
                            &format!("✅ MBTI data loaded for FID={}", fid_clone).into(),
                        );
                        mbti_data_clone.set(Some(data));
                        pending_job_clone.set(None);
                        loading_clone.set(false);
                    }
                    Err(e) => {
                        // Check for JOB_STATUS error format
                        if let Some((status, job_key, message)) =
                            parse_job_status_error(&e, format!("mbti:{}", fid_clone))
                        {
                            web_sys::console::log_1(
                                &format!(
                                    "⏳ MBTI analysis status: {} (job_key: {})",
                                    status, job_key
                                )
                                .into(),
                            );
                            let new_job = PendingJob {
                                job_key: job_key.clone(),
                                job_type: "mbti".to_string(),
                                status: Some(status.clone()),
                                started_at: Some(js_sys::Date::now() as u64),
                                message: Some(message.clone()),
                            };
                            pending_job_clone.set(Some(new_job));
                            loading_clone.set(false);
                            
                            // Start polling to check if data is ready
                            // Clone values for polling task
                            let mbti_data_for_poll = mbti_data_clone.clone();
                            let pending_job_for_poll = pending_job_clone.clone();
                            let loading_for_poll = loading_clone.clone();
                            let api_url_for_poll = api_url_for_effect.clone();
                            let endpoint_for_poll = endpoint.clone();
                            let wallet_account_for_poll = wallet_account_clone.clone();
                            let job_key_for_poll = job_key.clone();
                            
                            spawn_local(async move {
                                // Poll every 4 seconds to check if data is ready
                                let mut attempt = 0;
                                let max_attempts = 200; // Max ~13 minutes
                                
                                loop {
                                    if attempt >= max_attempts {
                                        break;
                                    }
                                    
                                    // Wait before polling (except first attempt)
                                    if attempt > 0 {
                                        let wait_time = 4000u64; // 4 seconds
                                        let promise = js_sys::Promise::new(&mut |resolve, _| {
                                            let window = web_sys::window().unwrap();
                                            window
                                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                                    &resolve,
                                                    wait_time as i32,
                                                )
                                                .unwrap();
                                        });
                                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                                    }
                                    
                                    attempt += 1;
                                    
                                    // Try to fetch data
                                    match make_request_with_payment::<MbtiProfile>(
                                        &api_url_for_poll,
                                        &endpoint_for_poll,
                                        None,
                                        wallet_account_for_poll.as_ref(),
                                        None,
                                        None,
                                    )
                                    .await
                                    {
                                        Ok(data) => {
                                            web_sys::console::log_1(
                                                &format!("✅ MBTI data loaded via polling for FID={}", fid_clone).into(),
                                            );
                                            mbti_data_for_poll.set(Some(data));
                                            pending_job_for_poll.set(None);
                                            loading_for_poll.set(false);
                                            break;
                                        }
                                        Err(e) => {
                                            // Check if still pending/processing
                                            if let Some((new_status, _, _)) =
                                                parse_job_status_error(&e, job_key_for_poll.clone())
                                            {
                                                if new_status == "pending" || new_status == "processing" {
                                                    // Still processing, continue polling
                                                    continue;
                                                } else {
                                                    // Status changed but not completed, stop polling
                                                    break;
                                                }
                                            } else {
                                                // Not a JOB_STATUS error, might be a real error
                                                // Continue polling in case it's temporary
                                                continue;
                                            }
                                        }
                                    }
                                }
                            });
                        } else {
                            web_sys::console::log_1(&format!("❌ MBTI data error: {}", e).into());
                            loading_clone.set(false);
                        }
                    }
                }
            });
        });
    }

    // Render based on state
    if let Some(data) = (*mbti_data).as_ref() {
        html! {
            <>
                <MbtiAnalysisView 
                    mbti={data.clone()}
                    fid={fid}
                    api_url={api_url.clone()}
                />
                <Dashboard
                    fid={fid}
                    api_url={api_url.clone()}
                />
            </>
        }
    } else if let Some(job) = (*pending_job).as_ref() {
        let message = job.message.as_deref().unwrap_or("Loading MBTI analysis...");
        html! {
            <MbtiSkeletonView message={message.to_string()} />
        }
    } else if *loading {
        html! {
            <MbtiSkeletonView message={"Loading MBTI analysis...".to_string()} />
        }
    } else {
        html! {
            <MbtiSkeletonView message={"MBTI analysis not available".to_string()} />
        }
    }
}

// ============================================================================
// Helper function to parse JOB_STATUS error format
// ============================================================================

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

// Import view components from views module
use crate::views::MbtiAnalysis;
use crate::views::MbtiSkeleton;
use crate::views::SocialAnalysis;
use crate::views::SocialSkeleton;

// Wrapper components for the views
#[derive(Properties, PartialEq, Clone)]
struct SocialAnalysisViewProps {
    social: SocialData,
}

#[function_component]
fn SocialAnalysisView(props: &SocialAnalysisViewProps) -> Html {
    html! {
        <SocialAnalysis social={props.social.clone()} />
    }
}

#[derive(Properties, PartialEq, Clone)]
struct MbtiAnalysisViewProps {
    mbti: MbtiProfile,
    fid: i64,
    api_url: String,
}

#[function_component]
fn MbtiAnalysisView(props: &MbtiAnalysisViewProps) -> Html {
    html! {
        <MbtiAnalysis mbti={props.mbti.clone()} />
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SocialSkeletonViewProps {
    message: String,
}

#[function_component]
fn SocialSkeletonView(props: &SocialSkeletonViewProps) -> Html {
    html! {
        <SocialSkeleton message={props.message.clone()} />
    }
}

#[derive(Properties, PartialEq, Clone)]
struct MbtiSkeletonViewProps {
    message: String,
}

#[function_component]
fn MbtiSkeletonView(props: &MbtiSkeletonViewProps) -> Html {
    html! {
        <MbtiSkeleton message={props.message.clone()} />
    }
}
