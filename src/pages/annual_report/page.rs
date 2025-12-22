use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys;
use yew::prelude::*;

use crate::models::{
    AnnualReportResponse, CastsStatsResponse, EngagementResponse,
    ProfileWithRegistration,
};
use crate::services::{
    create_annual_report_endpoint, create_casts_stats_endpoint, create_profile_endpoint,
    get_2025_timestamps, make_request_with_payment,
};

use super::utils::{convert_annual_report_response};
use super::{AnnualReportPageProps, ReportCard};
use super::sections::*;

/// Annual Report page component
#[function_component]
pub fn AnnualReportPage(props: &AnnualReportPageProps) -> Html {
    let annual_report = use_state(|| None::<AnnualReportResponse>);
    let profile = use_state(|| None::<ProfileWithRegistration>);
    let casts_stats = use_state(|| None::<CastsStatsResponse>);
    let engagement_2024 = use_state(|| None::<EngagementResponse>);
    let is_loading = use_state(|| false); // Track if data is still loading
    let show_intro = use_state(|| true); // Show intro screen initially
    let has_clicked_begin = use_state(|| false); // Track if user clicked begin
    let data_loading_complete = use_state(|| false); // Track if data loading is complete
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
        let data_loading_complete = data_loading_complete.clone();
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
            let data_loading_complete = data_loading_complete.clone();
            let loading_status = loading_status.clone();
            let api_url_clone = api_url_clone.clone();
            let wallet_account_clone = wallet_account_clone.clone();
            let scroll_container_ref = scroll_container_ref_for_loading.clone();
            let current_page = current_page_for_loading.clone();

            // Start loading data in background (don't show loading UI yet)
            web_sys::console::log_1(&"🚀 Starting annual report data loading in background...".into());
            is_loading.set(true); // Mark as loading

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
                                data_loading_complete.set(true); // Mark data loading as complete
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
                           data_loading_complete.set(true); // Mark as complete even on error
                           loading_status.set(format!("Failed to parse annual report: {}", parse_err));
                            }
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("❌ Failed to load annual report: {}", e).into(),
                        );
                        is_loading.set(false);
                        data_loading_complete.set(true); // Mark as complete even on error
                        loading_status.set("Failed to load annual report".to_string());
                    }
                }
            });
            || ()
        });
    }


    // Calculate total number of cards
    let total_cards = if annual_report.is_some() && profile.is_some() {
        13 // Cover + 12 sections (Identity, Voice Frequency, Engagement, Engagement Quality, Activity Distribution, Top Interactive Users, Growth Trend, Style, Content Themes, Highlights, CTA, Personality Tag)
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
                // Show intro screen first
                if *show_intro {
                    <>
                        // Fixed background image at bottom
                        <img 
                            src="/imgs/report-bg-0.png"
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
                        <div style="
                            position: fixed;
                            top: 0;
                            left: 0;
                            width: 100%;
                            height: 100vh;
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            justify-content: center;
                            z-index: 1000;
                        ">
                        <div style="
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            gap: 32px;
                            text-align: center;
                            padding: 40px;
                        ">
                            <h1 style="
                                font-size: 32px;
                                font-weight: 700;
                                color: white;
                                margin: 0;
                                text-shadow: 0 2px 20px rgba(0, 0, 0, 0.3);
                                white-space: nowrap;
                                overflow: hidden;
                                text-overflow: ellipsis;
                            ">{"Your year with Base"}</h1>
                            <button
                                onclick={Callback::from({
                                    let show_intro = show_intro.clone();
                                    let has_clicked_begin = has_clicked_begin.clone();
                                    move |_| {
                                        has_clicked_begin.set(true);
                                        // Hide intro - loading or content will be shown based on data state
                                        show_intro.set(false);
                                    }
                                })}
                                style="
                                    padding: 16px 48px;
                                    font-size: 18px;
                                    font-weight: 600;
                                    color: white;
                                    background: rgba(102, 126, 234, 0.3);
                                    backdrop-filter: blur(10px);
                                    -webkit-backdrop-filter: blur(10px);
                                    border: 2px solid rgba(118, 75, 162, 0.4);
                                    border-radius: 30px;
                                    cursor: pointer;
                                    transition: all 0.3s ease;
                                    text-transform: none;
                                    box-shadow: 0 4px 15px rgba(102, 126, 234, 0.2);
                                "
                                class="begin-button"
                            >
                                {"lets begin"}
                            </button>
                        </div>
                        <style>{"
                            .begin-button:hover {
                                background: rgba(102, 126, 234, 0.4) !important;
                                border-color: rgba(118, 75, 162, 0.6) !important;
                                box-shadow: 0 6px 20px rgba(102, 126, 234, 0.3) !important;
                                transform: scale(1.05);
                            }
                        "}</style>
                        </div>
                    </>
                } else if *is_loading && *has_clicked_begin {
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
                                    src="/imgs/report-bg.png"
                                    alt=""
                                    style="
                                        position: fixed;
                                        bottom: 0;
                                        left: 0;
                                        width: 100vw;
                                        height: 100vh;
                                        z-index: 0;
                                        pointer-events: none;
                                        object-fit: cover;
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

                                    // Section 6: Personality Tag Card (Second to Last)
                                    {if let (Some(temporal), Some(engagement), Some(style), Some(followers)) = (
                            annual_report.as_ref().map(|r| &r.temporal_activity),
                            annual_report.as_ref().map(|r| &r.engagement),
                            annual_report.as_ref().map(|r| &r.content_style),
                            annual_report.as_ref().map(|r| &r.follower_growth),
                        ) {
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
                                                <PersonalityTagSection
                                                    temporal={temporal.clone()}
                                                    engagement={engagement.clone()}
                                                    content_style={style.clone()}
                                                    follower_growth={followers.clone()}
                                                    casts_stats={casts}
                                                />
                                            </ReportCard>
                                        }
                                    } else {
                                        html! {}
                                    }}

                                    // Section 7: 2025 Call to Action Card (Last Page)
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

