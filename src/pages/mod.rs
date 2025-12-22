pub mod about;
pub mod profile;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys;
use yew::prelude::*;

use crate::models::{
    AnnualReportResponse, CastsStatsResponse, ContentStyleResponse, DomainStatusResponse,
    EngagementResponse, FollowerGrowthResponse,
    ProfileWithRegistration, TemporalActivityResponse,
};
use crate::services::{
    create_annual_report_endpoint, create_casts_stats_endpoint, create_profile_endpoint,
    get_2025_timestamps, make_request_with_payment,
};
use crate::wallet::WalletAccount;

// Re-export pages
pub use about::AboutPage;
pub use profile::ProfilePage;
pub use annual_report::AnnualReportPageProps;

pub mod annual_report;

use annual_report::utils::{convert_annual_report_response, farcaster_to_unix};
use annual_report::{ReportCard, ReportCardContent};

/// Annual Report page component
#[function_component]
pub fn AnnualReportPage(props: &annual_report::AnnualReportPageProps) -> Html {
    let annual_report = use_state(|| None::<AnnualReportResponse>);
    let profile = use_state(|| None::<ProfileWithRegistration>);
    let casts_stats = use_state(|| None::<CastsStatsResponse>);
    let engagement_2024 = use_state(|| None::<EngagementResponse>);
    let is_loading = use_state(|| false); // Don't show loading initially
    let show_intro = use_state(|| true); // Show intro screen initially
    let show_loading = use_state(|| false); // Only show loading after clicking begin
    let _error = use_state(|| None::<String>);
    let loading_status = use_state(|| "Loading annual report...".to_string());
    let current_page = use_state(|| 0);
    let scroll_container_ref = use_node_ref();

    let fid = props.fid;
    let api_url = props.api_url.clone();
    let wallet_account = props.wallet_account.clone();

    // Load annual report data in background
    {
        let annual_report = annual_report.clone();
        let profile = profile.clone();
        let casts_stats = casts_stats.clone();
        let engagement_2024 = engagement_2024.clone();
        let is_loading = is_loading.clone();
        let show_intro = show_intro.clone();
        let loading_status = loading_status.clone();
        let api_url_clone = api_url.clone();
        let wallet_account_clone = wallet_account.clone();
        let scroll_container_ref_for_loading = scroll_container_ref.clone();
        let current_page_for_loading = current_page.clone();

        use_effect_with((), move |_| {
            let annual_report = annual_report.clone();
            let profile = profile.clone();
            let casts_stats = casts_stats.clone();
            let _engagement_2024 = engagement_2024.clone();
            let is_loading = is_loading.clone();
            let show_intro = show_intro.clone();
            let loading_status = loading_status.clone();
            let api_url_clone = api_url_clone.clone();
            let wallet_account_clone = wallet_account_clone.clone();
            let scroll_container_ref = scroll_container_ref_for_loading.clone();
            let current_page = current_page_for_loading.clone();

            // Start loading data in background (don't show loading UI yet)
            web_sys::console::log_1(&"🚀 Starting annual report data loading in background...".into());

            spawn_local(async move {
                // Load annual report using unified endpoint
                loading_status.set("Loading annual report...".to_string());
                web_sys::console::log_1(&"🚀 Loading annual report from unified endpoint...".into());
                
                let annual_report_endpoint = create_annual_report_endpoint(fid, 2025);
                web_sys::console::log_1(&format!("🌐 Requesting annual report from: {}", annual_report_endpoint.path).into());
                
                match make_request_with_payment::<serde_json::Value>(
                    &api_url_clone,
                    &annual_report_endpoint,
                    None,
                    wallet_account_clone.as_ref(),
                    None,
                    None,
                )
                .await
                {
                    Ok(json) => {
                        web_sys::console::log_1(&"✅ Received response from annual report API".into());
                        web_sys::console::log_1(
                            &format!("📦 Response structure: {}", 
                                if json.get("data").is_some() { "has 'data' field" } else { "no 'data' field" }
                            ).into(),
                        );
                        // Extract the data field first
                        let api_data = if let Some(data) = json.get("data") {
                            data.clone()
                        } else {
                            json.clone()
                        };
                        // Clone for error logging
                        let api_data_for_error = api_data.clone();
                        match convert_annual_report_response(api_data) {
                            Ok(report) => {
                                // Successfully loaded from unified endpoint
                                web_sys::console::log_1(&"✅ Successfully parsed annual report".into());
                                web_sys::console::log_1(
                                    &format!("📊 Annual report data: FID={}, Year={}, Engagement={}", 
                                        report.fid, 
                                        report.year,
                                        report.engagement.total_engagement
                                    ).into(),
                                );
                                annual_report.set(Some(report));
                
                                // Load profile for display purposes
                                loading_status.set("Loading profile...".to_string());
                                let profile_endpoint = create_profile_endpoint(&fid.to_string(), true);
                                if let Ok(p) = make_request_with_payment::<ProfileWithRegistration>(
                    &api_url_clone,
                                    &profile_endpoint,
                    None,
                    wallet_account_clone.as_ref(),
                    None,
                    None,
                )
                .await
                                {
                                    profile.set(Some(p));
                                }

                // Load casts stats for additional data
                loading_status.set("Loading cast statistics...".to_string());
                                let (start_2025, end_2025) = get_2025_timestamps();
                let casts_endpoint = create_casts_stats_endpoint(fid, Some(start_2025), Some(end_2025));
                                if let Ok(json_data) = make_request_with_payment::<serde_json::Value>(
                    &api_url_clone,
                    &casts_endpoint,
                    None,
                    wallet_account_clone.as_ref(),
                    None,
                    None,
                )
                .await
                {
                                    if let Some(outer_data) = json_data.get("data") {
                            let actual_data = outer_data.get("data").unwrap_or(outer_data);
                                        if let Ok(stats) = serde_json::from_value::<CastsStatsResponse>(actual_data.clone()) {
                                casts_stats.set(Some(stats));
                                        }
                                    }
                                }
                                
                                web_sys::console::log_1(&"✅ All data loading completed".into());
                                is_loading.set(false);
                                show_intro.set(false); // Hide intro if data is ready
                                loading_status.set("Complete!".to_string());
                                
                                // Setup scroll listener after data is loaded
                                let scroll_container_ref_clone = scroll_container_ref.clone();
                                let current_page_clone = current_page.clone();
                                let window = web_sys::window().unwrap();
                                let timeout_closure = Closure::<dyn FnMut()>::new(move || {
                                    let scroll_handler = {
                                        let scroll_container_ref = scroll_container_ref_clone.clone();
                                        let current_page = current_page_clone.clone();
                                        Closure::<dyn FnMut(_)>::new(move |_e: web_sys::Event| {
                                            if let Some(element) = scroll_container_ref.cast::<web_sys::HtmlElement>() {
                                                let scroll_left = element.scroll_left();
                                                let client_width = element.client_width();
                                                let card_width = client_width as f64;
                                                let page = if card_width > 0.0 {
                                                    (scroll_left as f64 / card_width).round() as usize
                                                } else {
                                                    0
                                                };
                                                current_page.set(page);
                                            }
                                        })
                                    };
                                    
                                    if let Some(element) = scroll_container_ref_clone.cast::<web_sys::HtmlElement>() {
                                        if let Err(e) = element.add_event_listener_with_callback("scroll", scroll_handler.as_ref().unchecked_ref()) {
                                            web_sys::console::error_1(&format!("Failed to add scroll listener: {:?}", e).into());
                        } else {
                                            // Store handler to prevent it from being dropped
                                            scroll_handler.forget();
                                        }
                                    }
                                });
                                
                                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                    timeout_closure.as_ref().unchecked_ref(),
                                    200
                                );
                                timeout_closure.forget();
                            }
                            Err(parse_err) => {
                           let error_msg = format!("❌ Failed to parse annual report: {}", parse_err);
                           web_sys::console::error_1(&error_msg.clone().into());
                           web_sys::console::error_1(&format!("📦 Raw API response: {}", serde_json::to_string(&api_data_for_error).unwrap_or_else(|_| "Failed to serialize".to_string())).into());
                           is_loading.set(false);
                           show_intro.set(false); // Hide intro on error
                           loading_status.set(format!("Failed to parse annual report: {}", parse_err));
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("❌ Failed to load annual report: {}", e).into(),
                        );
                        is_loading.set(false);
                        show_intro.set(false); // Hide intro on error
                        loading_status.set("Failed to load annual report".to_string());
                    }
                }
            });
            || ()
        });
    }


    // Calculate total number of cards
    let total_cards = if annual_report.is_some() && profile.is_some() {
        12 // Cover + 11 sections (Identity, Voice Frequency, Engagement, Engagement Quality, Activity Distribution, Top Interactive Users, Growth Trend, Style, Content Themes, Highlights, CTA)
    } else {
        0
    };

    html! {
        <div class="annual-report-page" style="
            width: 100vw;
            height: 100vh;
            position: fixed;
            top: 0;
            left: 0;
            overflow: hidden;
            margin: 0;
            padding: 0;
            border: none;
            touch-action: pan-x;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncontextmenu={Callback::from(|e: web_sys::MouseEvent| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
                if *is_loading {
                <div style="
                    position: fixed;
                    top: 0;
                    left: 0;
                    width: 100%;
                    height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    z-index: 1000;
                ">
                    <div style="
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        gap: 24px;
                        text-align: center;
                        padding: 40px;
                    ">
                        // Animated spinner
                        <div style="
                            width: 60px;
                            height: 60px;
                            border: 4px solid rgba(255, 255, 255, 0.2);
                            border-top: 4px solid white;
                            border-radius: 50%;
                            animation: spin 1s linear infinite;
                        "></div>
                        // Loading text
                        <div style="
                            display: flex;
                            flex-direction: column;
                            gap: 8px;
                        ">
                            <p style="
                                font-size: 24px;
                                font-weight: 600;
                                color: white;
                                margin: 0;
                                text-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
                            ">{"Loading Annual Report"}</p>
                            <p style="
                                font-size: 16px;
                                font-weight: 400;
                                color: rgba(255, 255, 255, 0.9);
                                margin: 0;
                            ">{(*loading_status).clone()}</p>
                        </div>
                        // Progress dots animation
                        <div style="
                            display: flex;
                            gap: 8px;
                            margin-top: 8px;
                        ">
                            <div style="
                                width: 8px;
                                height: 8px;
                                border-radius: 50%;
                                background: white;
                                animation: pulse 1.4s ease-in-out infinite;
                                animation-delay: 0s;
                            "></div>
                            <div style="
                                width: 8px;
                                height: 8px;
                                border-radius: 50%;
                                background: white;
                                animation: pulse 1.4s ease-in-out infinite;
                                animation-delay: 0.2s;
                            "></div>
                            <div style="
                                width: 8px;
                                height: 8px;
                                border-radius: 50%;
                                background: white;
                                animation: pulse 1.4s ease-in-out infinite;
                                animation-delay: 0.4s;
                            "></div>
                        </div>
                    </div>
                    // CSS animations
                    <style>{"
                        @keyframes spin {
                            0% { transform: rotate(0deg); }
                            100% { transform: rotate(360deg); }
                        }
                        @keyframes pulse {
                            0%, 100% {
                                opacity: 0.4;
                                transform: scale(0.8);
                            }
                            50% {
                                opacity: 1;
                                transform: scale(1.2);
                            }
                        }
                    "}</style>
                </div>
            } else {
                <>
                    // Show error if annual report failed to load
                    {if annual_report.is_none() {
                        html! {
                            <div class="error-container" style="padding: 40px; text-align: center;">
                                <h2>{"Failed to load annual report"}</h2>
                        <p>{(*loading_status).clone()}</p>
                    </div>
                        }
                } else {
                        html! {
                            <>
                                // Fixed background image at bottom
                                <img 
                                    src="/imgs/report-background.png"
                                    alt=""
                                    style="
                                        position: fixed;
                                        bottom: 0;
                                        left: 0;
                                        width: 100vw;
                                        height: auto;
                                        z-index: 0;
                                        pointer-events: none;
                                        object-fit: contain;
                                        object-position: bottom center;
                                    "
                                />
                                // Horizontal scrolling container
                                <div 
                                    ref={scroll_container_ref.clone()}
                                    class="annual-report-scroll-container"
                                    style="
                                        display: flex;
                                        overflow-x: auto;
                                        overflow-y: hidden;
                                        scroll-snap-type: x mandatory;
                                        scroll-behavior: smooth;
                                        -webkit-overflow-scrolling: touch;
                                        width: 100vw;
                                        height: 100vh;
                                        position: relative;
                                        margin: 0;
                                        padding: 0;
                                        border: none;
                                        z-index: 1;
                                        touch-action: pan-x;
                                        user-select: none;
                                        -webkit-user-select: none;
                                        -moz-user-select: none;
                                        -ms-user-select: none;
                                        -webkit-user-drag: none;
                                        -khtml-user-drag: none;
                                        -moz-user-drag: none;
                                        -o-user-drag: none;
                                    "
                                    oncopy={Callback::from(|e: web_sys::Event| {
                                        e.prevent_default();
                                    })}
                                    oncut={Callback::from(|e: web_sys::Event| {
                                        e.prevent_default();
                                    })}
                                    onpaste={Callback::from(|e: web_sys::Event| {
                                        e.prevent_default();
                                    })}
                                    oncontextmenu={Callback::from(|e: web_sys::MouseEvent| {
                                        e.prevent_default();
                                    })}
                                    ondragstart={Callback::from(|e: web_sys::DragEvent| {
                                        e.prevent_default();
                                    })}
                                    onwheel={Callback::from({
                                        let scroll_container_ref = scroll_container_ref.clone();
                                        move |e: web_sys::WheelEvent| {
                                            // Only allow horizontal scrolling with wheel
                                            if e.delta_y().abs() > e.delta_x().abs() {
                                                e.prevent_default();
                                            }
                                        }
                                    })}
                                >
                                    // Cover Page Card
                                    {if let Some(p) = &*profile {
                                        html! {
                                            <ReportCard>
                                                <AnnualReportCover profile={p.clone()} />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 1: Your Farcaster Identity Card
                                    {if let (Some(p), Some(temporal), Some(followers)) = (
                            &*profile,
                            annual_report.as_ref().map(|r| &r.temporal_activity),
                            annual_report.as_ref().map(|r| &r.follower_growth),
                        ) {
                                        html! {
                                            <ReportCard>
                                                <IdentitySection
                                                    profile={p.clone()}
                                                    temporal={temporal.clone()}
                                                    followers={followers.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 2: Your Voice Frequency Card
                        {if let Some(temporal) = annual_report.as_ref().map(|r| &r.temporal_activity) {
                            let casts = casts_stats.as_ref().cloned().unwrap_or_else(|| CastsStatsResponse {
                                total_casts: 0,
                                date_distribution: Vec::new(),
                                date_range: None,
                                language_distribution: std::collections::HashMap::new(),
                                top_nouns: Vec::new(),
                                top_verbs: Vec::new(),
                            });
                            let comparison = annual_report.as_ref().and_then(|r| r.network_comparison.as_ref());
                            html! {
                                <ReportCard>
                                    <VoiceFrequencySection
                                        temporal={temporal.clone()}
                                        casts_stats={casts}
                                        network_comparison={comparison.cloned()}
                                    />
                                </ReportCard>
                            }
                        } else {
                            html! {}
                        }}

                                    // Section 3: Your Engagement Impact Card
                                    {if let Some(engagement) = annual_report.as_ref().map(|r| &r.engagement) {
                                        html! {
                                            <ReportCard>
                                                <EngagementSection
                                                    engagement={engagement.clone()}
                                                    engagement_2024={(*engagement_2024).clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 3.5: Activity Distribution Card
                                    {if let Some(temporal) = annual_report.as_ref().map(|r| &r.temporal_activity) {
                                        html! {
                                            <ReportCard>
                                                <ActivityDistributionSection
                                                    temporal={temporal.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 3.6: Top Interactive Users Card
                                    {if let Some(engagement) = annual_report.as_ref().map(|r| &r.engagement) {
                                        html! {
                                            <ReportCard>
                                                <TopInteractiveUsersSection
                                                    engagement={engagement.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 3.7: Engagement Quality Card
                                    {if let (Some(engagement), Some(temporal)) = (
                            annual_report.as_ref().map(|r| &r.engagement),
                                        annual_report.as_ref().map(|r| &r.temporal_activity),
                                    ) {
                                        html! {
                                            <ReportCard>
                                                <EngagementQualitySection
                                                    engagement={engagement.clone()}
                                                    temporal={temporal.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 3.8: Growth Trend Card
                                    {if let Some(followers) = annual_report.as_ref().map(|r| &r.follower_growth) {
                                        html! {
                                            <ReportCard>
                                                <GrowthTrendSection
                                                    followers={followers.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 4: Your Unique Style Card
                        {if let Some(style) = annual_report.as_ref().map(|r| &r.content_style) {
                            let casts = casts_stats.as_ref().cloned().unwrap_or_else(|| CastsStatsResponse {
                                total_casts: 0,
                                date_distribution: Vec::new(),
                                date_range: None,
                                language_distribution: std::collections::HashMap::new(),
                                top_nouns: Vec::new(),
                                top_verbs: Vec::new(),
                            });
                            html! {
                                <ReportCard>
                                    <StyleSection
                                        style={style.clone()}
                                        casts_stats={casts}
                                    />
                                </ReportCard>
                            }
                        } else {
                            html! {}
                        }}

                                    // Section 4.1: Content Themes Card
                                    {if let Some(style) = annual_report.as_ref().map(|r| &r.content_style) {
                                        html! {
                                            <ReportCard>
                                                <ContentThemeSection
                                                    style={style.clone()}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 5: Highlights Card
                                    {if let Some(temporal) = annual_report.as_ref().map(|r| &r.temporal_activity) {
                                        html! {
                                            <ReportCard>
                                                <HighlightsSection temporal={temporal.clone()} />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 6: 2025 Call to Action Card
                                    <ReportCard with_padding_top={false}>
                                        <CallToActionSection
                                            profile={(*profile).clone()}
                                            annual_report={(*annual_report).clone()}
                                        />
                                    </ReportCard>
                                </div>

                                // Pagination indicators (glassmorphism dots)
                                {if total_cards > 0 {
                                    html! {
                                        <div class="pagination-indicators" style="
                                            position: fixed;
                                            bottom: 20px;
                                            left: 50%;
                                            transform: translateX(-50%);
                                            display: flex;
                                            gap: 8px;
                                            z-index: 1000;
                                            padding: 8px 16px;
                                            background: rgba(255, 255, 255, 0.1);
                                            backdrop-filter: blur(10px);
                                            -webkit-backdrop-filter: blur(10px);
                                            border-radius: 20px;
                                            border: 1px solid rgba(255, 255, 255, 0.2);
                                        ">
                                            {for (0..total_cards).map(|i| {
                                                let is_active = *current_page == i;
                                                html! {
                                                    <div
                                                        class={if is_active { "indicator-dot active" } else { "indicator-dot" }}
                                                        style={format!("
                                                            width: {};
                                                            height: {};
                                                            border-radius: 50%;
                                                            background: {};
                                                            transition: all 0.3s ease;
                                                            cursor: pointer;
                                                        ",
                                                            if is_active { "10px" } else { "8px" },
                                                            if is_active { "10px" } else { "8px" },
                                                            if is_active {
                                                                "rgba(255, 255, 255, 0.9)"
                                                            } else {
                                                                "rgba(255, 255, 255, 0.4)"
                                                            }
                                                        )}
                                                        onclick={Callback::from({
                                                            let scroll_container_ref = scroll_container_ref.clone();
                                                            move |_| {
                                                                if let Some(element) = scroll_container_ref.cast::<web_sys::HtmlElement>() {
                                                                    let card_width = element.client_width() as f64;
                                                                    let scroll_to = (i as f64) * card_width;
                                                                    element.set_scroll_left(scroll_to as i32);
                                                                }
                                                            }
                                                        })}
                                                    />
                                                }
                                            })}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </>
                        }
                    }}
                </>
            }
        </div>
    }
}

// Cover Page Component
#[derive(Properties, PartialEq, Clone)]
struct AnnualReportCoverProps {
    profile: ProfileWithRegistration,
}

#[function_component]
fn AnnualReportCover(props: &AnnualReportCoverProps) -> Html {
    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
        ">
            <div class="cover-header" style="
                text-align: center;
                max-width: 600px;
            ">
                if let Some(pfp_url) = &props.profile.pfp_url {
                    <img 
                        src={pfp_url.clone()} 
                        alt="Profile" 
                        style="
                            width: 120px;
                            height: 120px;
                            border-radius: 50%;
                            border: 4px solid rgba(255, 255, 255, 0.3);
                            margin-bottom: 24px;
                            object-fit: cover;
                        "
                    />
                } else {
                    <div style="
                        width: 120px;
                        height: 120px;
                        border-radius: 50%;
                        border: 4px solid rgba(255, 255, 255, 0.3);
                        margin: 0 auto 24px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 60px;
                        background: rgba(255, 255, 255, 0.1);
                    ">{"👤"}</div>
                }
                <div class="cover-info">
                    <h1 style="
                        font-size: 48px;
                        font-weight: 700;
                        margin: 0 0 16px 0;
                        color: white;
                        text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
                    ">{"Your Farcaster 2025"}</h1>
                    <p style="
                        font-size: 20px;
                        color: rgba(255, 255, 255, 0.9);
                        margin: 0 0 24px 0;
                        line-height: 1.5;
                    ">{"This year, you made your voice heard and built connections"}</p>
                    if let Some(username) = &props.profile.username {
                        <p style="
                            font-size: 24px;
                            font-weight: 600;
                            color: white;
                            margin: 0 0 8px 0;
                        ">{format!("@{}", username)}</p>
                    }
                    <p style="
                        font-size: 16px;
                        color: rgba(255, 255, 255, 0.7);
                        margin: 0;
                    ">{format!("FID: {}", props.profile.fid)}</p>
                </div>
            </div>
        </div>
    }
}

// Identity Section Component
#[derive(Properties, PartialEq, Clone)]
struct IdentitySectionProps {
    profile: ProfileWithRegistration,
    temporal: TemporalActivityResponse,
    followers: FollowerGrowthResponse,
}

#[function_component]
fn IdentitySection(props: &IdentitySectionProps) -> Html {
    // Calculate days since registration
    // registered_at is a Farcaster timestamp, need to convert to Unix timestamp
    let days_since_registration = props
        .profile
        .registered_at
        .map(|farcaster_timestamp| {
            let unix_timestamp = farcaster_to_unix(farcaster_timestamp);
            let now = js_sys::Date::now() / 1000.0; // Current time in seconds
            ((now - unix_timestamp as f64) / 86400.0) as i64
        })
        .unwrap_or(0);

    // Format first cast date
    // cast.timestamp is a Farcaster timestamp, need to convert to Unix timestamp
    let first_cast_date = props
        .temporal
        .first_cast
        .as_ref()
        .map(|cast| {
            let unix_timestamp = farcaster_to_unix(cast.timestamp);
            let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
                unix_timestamp as f64 * 1000.0,
            ));
            format!(
                "{}-{:02}-{:02}",
                date.get_full_year(),
                date.get_month() + 1,
                date.get_date()
            )
        })
        .unwrap_or_else(|| "N/A".to_string());

    let follower_change = props.followers.current_followers as i64 - props.followers.followers_at_start as i64;

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Your Farcaster Identity"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 600px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Registration Date"}</div>
                    <div style="font-size: 24px; font-weight: 600; color: white;">{format!("Day {} on Farcaster", days_since_registration)}</div>
                </div>
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"First Cast"}</div>
                    <div style="font-size: 24px; font-weight: 600; color: white;">{first_cast_date}</div>
                </div>
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Following / Followers"}</div>
                    <div style="font-size: 24px; font-weight: 600; color: white;">
                        {format!("{} / {}", props.followers.current_following, props.followers.current_followers)}
                        {if follower_change != 0 {
                            html! {
                                <span style={format!("
                                    margin-left: 8px;
                                    font-size: 18px;
                                    color: {};
                                ", if follower_change > 0 { "#4ade80" } else { "#f87171" })}>
                                    {format!("({}{})", if follower_change > 0 { "+" } else { "" }, follower_change)}
                                </span>
                            }
                        } else {
                            html! {}
                        }}
                </div>
                </div>
            </div>
        </div>
    }
}

// Voice Frequency Section Component
#[derive(Properties, PartialEq, Clone)]
struct VoiceFrequencySectionProps {
    temporal: TemporalActivityResponse,
    casts_stats: CastsStatsResponse,
    network_comparison: Option<crate::models::NetworkComparison>,
}

#[function_component]
fn VoiceFrequencySection(props: &VoiceFrequencySectionProps) -> Html {
    // Use total_casts from temporal_activity if available, otherwise fall back to casts_stats
    let total_casts = if props.temporal.total_casts > 0 {
        props.temporal.total_casts
    } else {
        props.casts_stats.total_casts
    };
    let avg_per_week = (total_casts as f32 / 52.0).round();
    let network_avg = props
        .network_comparison
        .as_ref()
        .map(|nc| nc.avg_casts_per_user)
        .unwrap_or(0.0);

    let most_active_month = props
        .temporal
        .monthly_distribution
        .iter()
        .max_by_key(|m| m.count)
        .map(|m| m.month.clone())
        .unwrap_or_else(|| "N/A".to_string());

    let most_active_hour = props
        .temporal
        .most_active_hour
        .map(|h| format!("{}:00", h))
        .unwrap_or_else(|| "N/A".to_string());

    let max_monthly_count = props
        .temporal
        .monthly_distribution
        .iter()
        .map(|m| m.count)
        .max()
        .unwrap_or(1);

    let max_hourly_count = props
        .temporal
        .hourly_distribution
        .iter()
        .map(|h| h.count)
        .max()
        .unwrap_or(1);

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Voice Frequency"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Total Casts Published"}</div>
                    <div style="font-size: 32px; font-weight: 700; color: white;">{format!("{} Casts", total_casts)}</div>
                </div>
                <div style="
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 16px;
                ">
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Most Active Month"}</div>
                        <div style="font-size: 20px; font-weight: 600; color: white;">{most_active_month}</div>
                </div>
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Most Active Time"}</div>
                        <div style="font-size: 20px; font-weight: 600; color: white;">{most_active_hour}</div>
                </div>
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Average Per Week"}</div>
                        <div style="font-size: 20px; font-weight: 600; color: white;">
                        {format!("{:.1} Casts", avg_per_week)}
                        {if network_avg > 0.0 {
                            html! {
                                    <div style="font-size: 12px; color: rgba(255, 255, 255, 0.6); margin-top: 4px;">
                                        {format!("(Network avg {:.1})", network_avg / 52.0)}
                                    </div>
                            }
                        } else {
                            html! {}
                        }}
                        </div>
                    </div>
                </div>

            </div>
        </div>
    }
}

// Activity Distribution Section Component
#[derive(Properties, PartialEq, Clone)]
struct ActivityDistributionSectionProps {
    temporal: TemporalActivityResponse,
}

#[function_component]
fn ActivityDistributionSection(props: &ActivityDistributionSectionProps) -> Html {
    let max_monthly_count = props
        .temporal
        .monthly_distribution
        .iter()
        .map(|m| m.count)
        .max()
        .unwrap_or(1);

    let max_hourly_count = props
        .temporal
        .hourly_distribution
        .iter()
        .map(|h| h.count)
        .max()
        .unwrap_or(1);

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Activity Distribution"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                // Monthly Distribution Bar Chart
                {if !props.temporal.monthly_distribution.is_empty() {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <h3 style="
                                font-size: 18px;
                                font-weight: 600;
                                color: white;
                                margin: 0 0 16px 0;
                            ">{"Monthly Activity Distribution"}</h3>
                            <div style="
                                display: flex;
                                gap: 8px;
                                align-items: flex-end;
                                height: 200px;
                            ">
                            {for props.temporal.monthly_distribution.iter().map(|month| {
                                    let height_percent = (month.count as f32 / max_monthly_count as f32 * 100.0) as f32;
                                html! {
                                        <div style="
                                            flex: 1;
                                            display: flex;
                                            flex-direction: column;
                                            align-items: center;
                                            height: 100%;
                                        ">
                                            <div style={format!("
                                                width: 100%;
                                                background: linear-gradient(to top, rgba(0, 122, 255, 0.8), rgba(0, 122, 255, 0.4));
                                                border-radius: 4px 4px 0 0;
                                                height: {}%;
                                                display: flex;
                                                align-items: flex-end;
                                                justify-content: center;
                                                padding-bottom: 4px;
                                                color: white;
                                                font-size: 12px;
                                                font-weight: 600;
                                            ", height_percent)}>
                                                {month.count}
                                            </div>
                                            <div style="
                                                font-size: 11px;
                                                color: rgba(255, 255, 255, 0.7);
                                                margin-top: 8px;
                                                text-align: center;
                                                writing-mode: horizontal-tb;
                                                transform: none;
                                            ">{month.month.clone()}</div>
                                    </div>
                                }
                            })}
                        </div>
                    </div>
                }
                } else {
                    html! {}
                }}

                // Hourly Distribution Chart
                {if !props.temporal.hourly_distribution.is_empty() {
                    // Rainbow colors for bars
                    let rainbow_colors = vec![
                        "rgba(255, 0, 0, 0.8)",      // Red
                        "rgba(255, 127, 0, 0.8)",   // Orange
                        "rgba(255, 255, 0, 0.8)",   // Yellow
                        "rgba(0, 255, 0, 0.8)",     // Green
                        "rgba(0, 0, 255, 0.8)",     // Blue
                        "rgba(75, 0, 130, 0.8)",    // Indigo
                        "rgba(148, 0, 211, 0.8)",   // Violet
                    ];
                    
                                html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <h3 style="
                                font-size: 18px;
                                font-weight: 600;
                                color: white;
                                margin: 0 0 16px 0;
                            ">{"Hourly Activity Distribution"}</h3>
                            <div style="
                                display: flex;
                                gap: 4px;
                                align-items: flex-end;
                                height: 150px;
                            ">
                            {for props.temporal.hourly_distribution.iter().enumerate().map(|(idx, hour)| {
                                    let height_percent = (hour.count as f32 / max_hourly_count as f32 * 100.0) as f32;
                                    let color = rainbow_colors[idx % rainbow_colors.len()];
                                html! {
                                        <div style="
                                            flex: 1;
                                            display: flex;
                                            flex-direction: column;
                                            align-items: center;
                                            height: 100%;
                                        " title={format!("{}:00 - {} casts", hour.hour, hour.count)}>
                                            <div style={format!("
                                                width: 100%;
                                                background: {};
                                                border-radius: 2px 2px 0 0;
                                                height: {}%;
                                            ", color, height_percent)} />
                                            <div style="
                                                font-size: 10px;
                                                color: rgba(255, 255, 255, 0.6);
                                                margin-top: 4px;
                                            ">{format!("{}", hour.hour)}</div>
                                        </div>
                                    }
                                })}
                            </div>
                                    </div>
                                }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

// Top Interactive Users Section Component
#[derive(Properties, PartialEq, Clone)]
struct TopInteractiveUsersSectionProps {
    engagement: EngagementResponse,
}

#[function_component]
fn TopInteractiveUsersSection(props: &TopInteractiveUsersSectionProps) -> Html {
    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Top Interactive Users"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                {if !props.engagement.top_reactors.is_empty() {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 16px;">{"Top 3 Most Interactive Users"}</div>
                            <div style="display: flex; flex-direction: column; gap: 12px;">
                                {for props.engagement.top_reactors.iter().take(3).map(|reactor| {
                                    html! {
                                        <div style="
                                            display: flex;
                                            justify-content: space-between;
                                            align-items: center;
                                            padding: 12px;
                                            background: rgba(255, 255, 255, 0.05);
                                            border-radius: 8px;
                                        ">
                                            <span style="
                                                font-size: 16px;
                                                font-weight: 500;
                                                color: white;
                                            ">
                                                {reactor.display_name.as_ref()
                                                    .or(reactor.username.as_ref())
                                                    .map(|n| n.clone())
                                                    .unwrap_or_else(|| format!("FID: {}", reactor.fid))}
                                            </span>
                                            <span style="
                                                font-size: 14px;
                                                color: rgba(255, 255, 255, 0.7);
                                            ">{format!("{} times", reactor.interaction_count)}</span>
                                        </div>
                                    }
                                })}
                        </div>
                    </div>
                }
                } else {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                            text-align: center;
                            color: rgba(255, 255, 255, 0.7);
                        ">
                            {"No interactive users data available"}
                        </div>
                    }
                }}
            </div>
        </div>
    }
}

// Engagement Section Component
#[derive(Properties, PartialEq, Clone)]
struct EngagementSectionProps {
    engagement: EngagementResponse,
    engagement_2024: Option<EngagementResponse>,
}

#[function_component]
fn EngagementSection(props: &EngagementSectionProps) -> Html {
    let year_over_year = props.engagement_2024.as_ref().map(|e2024| {
        if e2024.reactions_received > 0 {
            let change = ((props.engagement.reactions_received as f32
                - e2024.reactions_received as f32)
                / e2024.reactions_received as f32)
                * 100.0;
            format!("{:.1}%", change)
        } else {
            "N/A".to_string()
        }
    });

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Engagement Impact"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Total Likes Received"}</div>
                    <div style="font-size: 32px; font-weight: 700; color: white;">
                        {format!("{}", props.engagement.reactions_received)}
                        {if let Some(yoy) = year_over_year {
                            html! {
                                <span style="
                                    margin-left: 12px;
                                    font-size: 18px;
                                    color: rgba(255, 255, 255, 0.7);
                                ">
                                    {format!("(YoY {})", yoy)}
                                </span>
                            }
                        } else {
                            html! {}
                        }}
                </div>
                </div>
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Total Recasts Received"}</div>
                    <div style="font-size: 32px; font-weight: 700; color: white;">{format!("{}", props.engagement.recasts_received)}</div>
                </div>
                {if let Some(popular_cast) = &props.engagement.most_popular_cast {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 12px;">{"Most Popular Cast"}</div>
                            <p style="
                                font-size: 16px;
                                color: white;
                                line-height: 1.6;
                                margin: 0 0 16px 0;
                            ">{&popular_cast.text}</p>
                            <div style="
                                display: flex;
                                gap: 16px;
                                flex-wrap: wrap;
                            ">
                                <span style="color: rgba(255, 255, 255, 0.9);">{"Likes: "}{popular_cast.reactions}</span>
                                <span style="color: rgba(255, 255, 255, 0.9);">{"Recasts: "}{popular_cast.recasts}</span>
                                <span style="color: rgba(255, 255, 255, 0.9);">{"Replies: "}{popular_cast.replies}</span>
                                </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

// Engagement Quality Section Component
#[derive(Properties, PartialEq, Clone)]
struct EngagementQualitySectionProps {
    engagement: EngagementResponse,
    temporal: TemporalActivityResponse,
}

#[function_component]
fn EngagementQualitySection(props: &EngagementQualitySectionProps) -> Html {
    let total_casts = props.temporal.total_casts_in_year.unwrap_or(props.temporal.total_casts);
    let avg_engagement_per_cast = if total_casts > 0 {
        props.engagement.total_engagement as f32 / total_casts as f32
    } else {
        0.0
    };
    let reaction_rate = if total_casts > 0 {
        (props.engagement.reactions_received as f32 / total_casts as f32) * 100.0
    } else {
        0.0
    };
    let reply_rate = if total_casts > 0 {
        (props.engagement.replies_received as f32 / total_casts as f32) * 100.0
    } else {
        0.0
    };
    let recast_rate = if total_casts > 0 {
        (props.engagement.recasts_received as f32 / total_casts as f32) * 100.0
    } else {
        0.0
    };

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Engagement Quality"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Average Engagement per Cast"}</div>
                    <div style="font-size: 32px; font-weight: 700; color: white;">{format!("{:.2}", avg_engagement_per_cast)}</div>
                </div>
                <div style="
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                    gap: 16px;
                ">
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Reaction Rate"}</div>
                        <div style="font-size: 24px; font-weight: 600; color: white;">{format!("{:.1}%", reaction_rate)}</div>
                    </div>
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Reply Rate"}</div>
                        <div style="font-size: 24px; font-weight: 600; color: white;">{format!("{:.1}%", reply_rate)}</div>
                    </div>
                    <div style="
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 20px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <div style="font-size: 12px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Recast Rate"}</div>
                        <div style="font-size: 24px; font-weight: 600; color: white;">{format!("{:.1}%", recast_rate)}</div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// Growth Trend Section Component
#[derive(Properties, PartialEq, Clone)]
struct GrowthTrendSectionProps {
    followers: FollowerGrowthResponse,
}

#[function_component]
fn GrowthTrendSection(props: &GrowthTrendSectionProps) -> Html {
    let growth_rate = if props.followers.followers_at_start > 0 {
        ((props.followers.current_followers as f32 - props.followers.followers_at_start as f32) 
            / props.followers.followers_at_start as f32) * 100.0
    } else {
        0.0
    };
    
    // Find month with highest growth
    let max_growth_month = props.followers.monthly_snapshots
        .iter()
        .zip(props.followers.monthly_snapshots.iter().skip(1))
        .map(|(prev, curr)| {
            let growth = curr.followers as i32 - prev.followers as i32;
            (curr.month.clone(), growth)
        })
        .max_by_key(|(_, growth)| *growth)
        .map(|(month, growth)| (month, growth));

    let max_followers = props.followers.monthly_snapshots
        .iter()
        .map(|s| s.followers)
        .max()
        .unwrap_or(1);

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Growth Trend"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Follower Growth Rate"}</div>
                    <div style="font-size: 32px; font-weight: 700; color: white;">{format!("{:.1}%", growth_rate)}</div>
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-top: 8px;">
                        {format!("From {} to {} followers", props.followers.followers_at_start, props.followers.current_followers)}
                    </div>
                </div>
                {if let Some((month, growth)) = max_growth_month {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"Fastest Growth Month"}</div>
                            <div style="font-size: 24px; font-weight: 600; color: white;">{month}</div>
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-top: 8px;">
                                {format!("+{} followers", growth)}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
                {if !props.followers.monthly_snapshots.is_empty() {
                    // Helper function to extract month from "YYYY-MM" format
                    let extract_month = |month_str: &str| -> String {
                        if let Some(dash_pos) = month_str.rfind('-') {
                            let month_num = &month_str[dash_pos + 1..];
                            match month_num {
                                "01" => "Jan".to_string(),
                                "02" => "Feb".to_string(),
                                "03" => "Mar".to_string(),
                                "04" => "Apr".to_string(),
                                "05" => "May".to_string(),
                                "06" => "Jun".to_string(),
                                "07" => "Jul".to_string(),
                                "08" => "Aug".to_string(),
                                "09" => "Sep".to_string(),
                                "10" => "Oct".to_string(),
                                "11" => "Nov".to_string(),
                                "12" => "Dec".to_string(),
                                _ => month_num.to_string(),
                            }
                        } else {
                            month_str.to_string()
                        }
                    };
                    
                    // Rainbow colors for bars
                    let rainbow_colors = vec![
                        "rgba(255, 0, 0, 0.8)",      // Red
                        "rgba(255, 127, 0, 0.8)",   // Orange
                        "rgba(255, 255, 0, 0.8)",   // Yellow
                        "rgba(0, 255, 0, 0.8)",     // Green
                        "rgba(0, 0, 255, 0.8)",     // Blue
                        "rgba(75, 0, 130, 0.8)",    // Indigo
                        "rgba(148, 0, 211, 0.8)",   // Violet
                    ];
                    
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                            overflow-x: auto;
                        ">
                            <h3 style="
                                font-size: 18px;
                                font-weight: 600;
                                color: white;
                                margin: 0 0 16px 0;
                            ">{"Monthly Follower Growth"}</h3>
                            <div style="
                                display: flex;
                                gap: 2px;
                                align-items: flex-end;
                                height: 200px;
                                min-width: fit-content;
                            ">
                                {for props.followers.monthly_snapshots.iter().enumerate().map(|(idx, snapshot)| {
                                    let height_percent = (snapshot.followers as f32 / max_followers as f32 * 100.0) as f32;
                                    let color = rainbow_colors[idx % rainbow_colors.len()];
                                    let month_label = extract_month(&snapshot.month);
                                    html! {
                                        <div style="
                                            flex: 0 0 auto;
                                            width: 20px;
                                            display: flex;
                                            flex-direction: column;
                                            align-items: center;
                                            height: 100%;
                                        ">
                                            <div style={format!("
                                                width: 16px;
                                                background: {};
                                                border-radius: 2px 2px 0 0;
                                                height: {}%;
                                                display: flex;
                                                align-items: flex-end;
                                                justify-content: center;
                                                padding-bottom: 2px;
                                                color: white;
                                                font-size: 8px;
                                                font-weight: 600;
                                                min-height: 18px;
                                            ", color, height_percent)}>
                                                {snapshot.followers}
                                            </div>
                                            <div style="
                                                font-size: 9px;
                                                color: rgba(255, 255, 255, 0.7);
                                                margin-top: 6px;
                                                text-align: center;
                                                white-space: nowrap;
                                            ">{month_label}</div>
                                        </div>
                                    }
                                })}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

// Content Theme Section Component
#[derive(Properties, PartialEq, Clone)]
struct ContentThemeSectionProps {
    style: ContentStyleResponse,
}

#[function_component]
fn ContentThemeSection(props: &ContentThemeSectionProps) -> Html {
    // Analyze top words to identify themes
    let top_words = props.style.top_words.iter().take(10).collect::<Vec<_>>();
    
    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Content Themes"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                {if !top_words.is_empty() {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 16px;">{"Top Words"}</div>
                            <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                                {for top_words.iter().map(|word| {
                                    html! {
                                        <span style="
                                            background: rgba(255, 255, 255, 0.15);
                                            padding: 8px 16px;
                                            border-radius: 20px;
                                            font-size: 16px;
                                            font-weight: 500;
                                            color: white;
                                        ">{format!("{} ({})", word.word, word.count)}</span>
                                    }
                                })}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

// Style Section Component
#[derive(Properties, PartialEq, Clone)]
struct StyleSectionProps {
    style: ContentStyleResponse,
    casts_stats: CastsStatsResponse,
}

#[function_component]
fn StyleSection(props: &StyleSectionProps) -> Html {
    let top_emojis = props.style.top_emojis.iter().take(3).collect::<Vec<_>>();
    // Use top_words from content_style, not from casts_stats
    let top_words = props.style.top_words.clone();
    
    // Find max count for font size calculation
    let max_count = top_words.iter().map(|w| w.count).max().unwrap_or(1);

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Content Style"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 12px;">{"Most Used Emojis"}</div>
                    <div style="display: flex; gap: 12px; flex-wrap: wrap;">
                        {for top_emojis.iter().map(|e| {
                            html! {
                                <span style="
                                    background: rgba(255, 255, 255, 0.15);
                                    padding: 8px 16px;
                                    border-radius: 20px;
                                    font-size: 18px;
                                    color: white;
                                ">{format!("{} ({})", e.emoji, e.count)}</span>
                            }
                        })}
                </div>
                </div>
                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                ">
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 12px;">{"Most Used Words"}</div>
                    <div style="
                        display: flex;
                        gap: 8px 12px;
                        flex-wrap: wrap;
                        align-items: center;
                        justify-content: center;
                        min-height: 200px;
                    ">
                        {for top_words.iter().enumerate().map(|(idx, word)| {
                            // Calculate font size based on count (word cloud effect)
                            let size_ratio = word.count as f32 / max_count as f32;
                            let font_size = (12.0 + size_ratio * 24.0).clamp(12.0, 36.0);
                            
                            // Rainbow colors - randomly assign based on index
                            let rainbow_colors = vec![
                                "#FF0000",      // Red
                                "#FF7F00",      // Orange
                                "#FFFF00",      // Yellow
                                "#00FF00",      // Green
                                "#0000FF",      // Blue
                                "#4B0082",      // Indigo
                                "#9400D3",      // Violet
                            ];
                            // Use index to cycle through colors, but add some randomness
                            let color_idx = (idx + (word.word.len() % rainbow_colors.len())) % rainbow_colors.len();
                            let color = rainbow_colors[color_idx];
                            
                            html! {
                                <span style={format!("
                                    font-size: {}px;
                                    font-weight: {};
                                    color: {};
                                    display: inline-block;
                                    transition: transform 0.2s ease;
                                ",
                                    font_size,
                                    if size_ratio > 0.5 { "600" } else { "500" },
                                    color
                                )}>
                                    {word.word.clone()}
                                </span>
                            }
                        })}
                </div>
                </div>
            </div>
        </div>
    }
}

// Highlights Section Component
#[derive(Properties, PartialEq, Clone)]
struct HighlightsSectionProps {
    temporal: TemporalActivityResponse,
}

#[function_component]
fn HighlightsSection(props: &HighlightsSectionProps) -> Html {
    // Format date from Farcaster timestamp
    let format_date = |farcaster_timestamp: i64| {
        let unix_timestamp = farcaster_to_unix(farcaster_timestamp);
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
            unix_timestamp as f64 * 1000.0,
        ));
        format!(
            "{}-{:02}-{:02}",
            date.get_full_year(),
            date.get_month() + 1,
            date.get_date()
        )
    };

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        "
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 32px 0;
                color: white;
                text-align: center;
            ">{"Highlights"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                {if let Some(first_cast) = &props.temporal.first_cast {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 12px;">{"First Cast of the Year"}</div>
                            <p style="
                                font-size: 16px;
                                color: white;
                                line-height: 1.6;
                                margin: 0 0 12px 0;
                            ">{&first_cast.text}</p>
                            <p style="
                                font-size: 14px;
                                color: rgba(255, 255, 255, 0.6);
                                margin: 0;
                            ">{format_date(first_cast.timestamp)}</p>
                        </div>
                    }
                } else {
                    html! {}
                }}
                {if let Some(last_cast) = &props.temporal.last_cast {
                    html! {
                        <div style="
                            background: rgba(255, 255, 255, 0.1);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(255, 255, 255, 0.2);
                        ">
                            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 12px;">{"Last Cast of the Year"}</div>
                            <p style="
                                font-size: 16px;
                                color: white;
                                line-height: 1.6;
                                margin: 0 0 12px 0;
                            ">{&last_cast.text}</p>
                            <p style="
                                font-size: 14px;
                                color: rgba(255, 255, 255, 0.6);
                                margin: 0;
                            ">{format_date(last_cast.timestamp)}</p>
                        </div>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

// Call to Action Section Component
#[derive(Properties, PartialEq, Clone)]
struct CallToActionSectionProps {
    profile: Option<ProfileWithRegistration>,
    annual_report: Option<AnnualReportResponse>,
}

#[function_component]
fn CallToActionSection(props: &CallToActionSectionProps) -> Html {
    let share_text = use_state(|| String::new());
    let is_sharing = use_state(|| false);

    let on_share = {
        let share_text = share_text.clone();
        let is_sharing = is_sharing.clone();
        let profile = props.profile.clone();
        let report = props.annual_report.clone();

        Callback::from(move |_| {
            is_sharing.set(true);
            let mut text = String::from("🎉 Farcaster 2025 Annual Report\n\n");

            if let Some(p) = &profile {
                if let Some(username) = &p.username {
                    text.push_str(&format!("@{}'s 2025 Annual Report\n\n", username));
                }
            }

            if let Some(r) = &report {
                text.push_str(&format!("📊 Published {} Casts this year\n", r.engagement.total_engagement));
                text.push_str(&format!("❤️ Received {} likes\n", r.engagement.reactions_received));
                text.push_str(&format!("🔁 Received {} recasts\n", r.engagement.recasts_received));
                
                if let Some(most_active) = &r.temporal_activity.most_active_month {
                    text.push_str(&format!("📅 Most active month: {}\n", most_active));
                }

                if !r.content_style.top_emojis.is_empty() {
                    let top_emoji = &r.content_style.top_emojis[0];
                    text.push_str(&format!("😊 Most used emoji: {}\n", top_emoji.emoji));
                }
            }

            text.push_str("\n#MyFarcaster2025\n");
            text.push_str("The story of Web3 is written by you.");

            share_text.set(text.clone());

            // Copy to clipboard using js_sys
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let textarea = document.create_element("textarea").unwrap();
            let textarea_js: &wasm_bindgen::JsValue = textarea.as_ref();
            js_sys::Reflect::set(textarea_js, &"value".into(), &text.into()).unwrap();
            let style = js_sys::Reflect::get(textarea_js, &"style".into()).unwrap();
            js_sys::Reflect::set(&style, &"position".into(), &"fixed".into()).unwrap();
            js_sys::Reflect::set(&style, &"left".into(), &"-9999px".into()).unwrap();
            document.body().unwrap().append_child(&textarea).unwrap();
            js_sys::Reflect::get(textarea_js, &"select".into())
                .and_then(|f| js_sys::Function::from(f).call0(textarea_js))
                .ok();
            
            let exec_command_result = js_sys::Reflect::get(&document, &"execCommand".into())
                .and_then(|f| {
                    js_sys::Function::from(f)
                        .call2(&document, &"copy".into(), &wasm_bindgen::JsValue::FALSE)
                });
            
            if exec_command_result.is_ok() {
                web_sys::console::log_1(&"✅ Text copied to clipboard".into());
            } else {
                web_sys::console::warn_1(&"⚠️ Failed to copy to clipboard".into());
            }
            
            document.body().unwrap().remove_child(&textarea).unwrap();

            is_sharing.set(false);
        })
    };

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
        ">
            <h2 style="
                font-size: 36px;
                font-weight: 700;
                margin: 0 0 24px 0;
                color: white;
                text-align: center;
            ">{"2026"}</h2>
            <p style="
                font-size: 20px;
                color: rgba(255, 255, 255, 0.9);
                margin: 0 0 32px 0;
                text-align: center;
            ">{"The story of Web3 is written by you."}</p>
                <button
                    onclick={on_share}
                    disabled={*is_sharing}
                style="
                    background: rgba(0, 122, 255, 0.8);
                    color: white;
                    border: none;
                    border-radius: 12px;
                    padding: 16px 32px;
                    font-size: 18px;
                    font-weight: 600;
                    cursor: pointer;
                    transition: all 0.3s ease;
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                "
                >
                    {if *is_sharing {
                        "Generating..."
                    } else {
                    "Share Your Annual Report"
                    }}
                </button>
                {if !(*share_text).is_empty() {
                    html! {
                    <div style="
                        margin-top: 32px;
                        max-width: 600px;
                        width: 100%;
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 16px;
                        padding: 24px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    ">
                        <p style="
                            font-size: 14px;
                            color: rgba(255, 255, 255, 0.7);
                            margin: 0 0 12px 0;
                        ">{"Share text copied to clipboard:"}</p>
                        <div style="
                            font-size: 14px;
                            color: white;
                            line-height: 1.6;
                            white-space: pre-wrap;
                        ">
                                {(*share_text).clone()}
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
        </div>
    }
}
