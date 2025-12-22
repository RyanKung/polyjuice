use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::models::{
    AnnualReportResponse, CastsStatsResponse, ContentStyleResponse,
    EngagementResponse, FollowerGrowthResponse,
    ProfileWithRegistration, TemporalActivityResponse,
};

use super::utils::farcaster_to_unix;

// Cover Page Component
#[derive(Properties, PartialEq, Clone)]
pub struct AnnualReportCoverProps {
    pub profile: ProfileWithRegistration,
}

#[function_component]
pub fn AnnualReportCover(props: &AnnualReportCoverProps) -> Html {
    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: 100%;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: flex-start;
            padding: 80px 40px 40px 40px;
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
                            width: 100px;
                            height: 100px;
                            border-radius: 50%;
                            border: 3px solid rgba(255, 255, 255, 0.3);
                            margin-bottom: 20px;
                            object-fit: cover;
                        "
                    />
                } else {
                    <div style="
                        width: 100px;
                        height: 100px;
                        border-radius: 50%;
                        border: 3px solid rgba(255, 255, 255, 0.3);
                        margin: 0 auto 20px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 50px;
                        background: rgba(255, 255, 255, 0.1);
                    ">{"👤"}</div>
                }
                <div class="cover-info">
                    <h1 style="
                        font-size: 36px;
                        font-weight: 700;
                        margin: 0 0 12px 0;
                        color: white;
                        text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
                    ">{"Your Farcaster 2025"}</h1>
                    <p style="
                        font-size: 16px;
                        color: rgba(255, 255, 255, 0.9);
                        margin: 0 0 20px 0;
                        line-height: 1.5;
                    ">{"This year, you made your voice heard and built connections"}</p>
                    if let Some(username) = &props.profile.username {
                        <p style="
                            font-size: 20px;
                            font-weight: 600;
                            color: white;
                            margin: 0 0 6px 0;
                        ">{format!("@{}", username)}</p>
                    }
                    <p style="
                        font-size: 14px;
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
pub struct IdentitySectionProps {
    pub profile: ProfileWithRegistration,
    pub temporal: TemporalActivityResponse,
    pub followers: FollowerGrowthResponse,
}

#[function_component]
pub fn IdentitySection(props: &IdentitySectionProps) -> Html {
    // Format first cast as "XXX Ago"
    // cast.timestamp is a Farcaster timestamp, need to convert to Unix timestamp
    let first_cast_ago = props
        .temporal
        .first_cast
        .as_ref()
        .map(|cast| {
            let unix_timestamp = farcaster_to_unix(cast.timestamp);
            let now = js_sys::Date::now() / 1000.0; // Current time in seconds
            let diff_seconds = now - (unix_timestamp as f64);
            let diff_days = (diff_seconds / 86400.0) as i64;
            let diff_months = diff_days / 30;
            let diff_years = diff_days / 365;
            
            if diff_years > 0 {
                format!("{} {} ago", diff_years, if diff_years == 1 { "year" } else { "years" })
            } else if diff_months > 0 {
                format!("{} {} ago", diff_months, if diff_months == 1 { "month" } else { "months" })
            } else if diff_days > 0 {
                format!("{} {} ago", diff_days, if diff_days == 1 { "day" } else { "days" })
            } else {
                "Today".to_string()
            }
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
                    <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">{"First Cast"}</div>
                    <div style="font-size: 24px; font-weight: 600; color: white;">{first_cast_ago}</div>
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
pub struct VoiceFrequencySectionProps {
    pub temporal: TemporalActivityResponse,
    pub casts_stats: CastsStatsResponse,
    pub network_comparison: Option<crate::models::NetworkComparison>,
}

#[function_component]
pub fn VoiceFrequencySection(props: &VoiceFrequencySectionProps) -> Html {
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

    // Format most active month as "February" with zodiac sign
    let (most_active_month, zodiac_sign) = props
        .temporal
        .monthly_distribution
        .iter()
        .max_by_key(|m| m.count)
        .map(|m| {
            // Parse YYYY-MM format
            let parts: Vec<&str> = m.month.split('-').collect();
            if parts.len() >= 2 {
                let month_num: u32 = parts[1].parse().unwrap_or(1);
                let month_name = match month_num {
                    1 => "January",
                    2 => "February",
                    3 => "March",
                    4 => "April",
                    5 => "May",
                    6 => "June",
                    7 => "July",
                    8 => "August",
                    9 => "September",
                    10 => "October",
                    11 => "November",
                    12 => "December",
                    _ => "Unknown",
                };
                // Zodiac signs based on month
                let zodiac = match month_num {
                    1 => "♑", // Capricorn
                    2 => "♒", // Aquarius
                    3 => "♓", // Pisces
                    4 => "♈", // Aries
                    5 => "♉", // Taurus
                    6 => "♊", // Gemini
                    7 => "♋", // Cancer
                    8 => "♌", // Leo
                    9 => "♍", // Virgo
                    10 => "♎", // Libra
                    11 => "♏", // Scorpio
                    12 => "♐", // Sagittarius
                    _ => "",
                };
                (month_name.to_string(), zodiac.to_string())
            } else {
                ("N/A".to_string(), "".to_string())
            }
        })
        .unwrap_or_else(|| ("N/A".to_string(), "".to_string()));

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
                        <div style="font-size: 20px; font-weight: 600; color: white;">
                            {most_active_month.clone()}
                            {if !zodiac_sign.is_empty() {
                                html! {
                                    <span style="margin-left: 8px; font-size: 24px;">{zodiac_sign.clone()}</span>
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
pub struct ActivityDistributionSectionProps {
    pub temporal: TemporalActivityResponse,
}

#[function_component]
pub fn ActivityDistributionSection(props: &ActivityDistributionSectionProps) -> Html {
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
pub struct TopInteractiveUsersSectionProps {
    pub engagement: EngagementResponse,
}

#[function_component]
pub fn TopInteractiveUsersSection(props: &TopInteractiveUsersSectionProps) -> Html {
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
                margin: 0 0 48px 0;
                color: white;
                text-align: center;
            ">{"Top Interactive Users"}</h2>
            <div style="
                display: flex;
                flex-wrap: wrap;
                justify-content: center;
                align-items: center;
                gap: 32px;
                max-width: 900px;
                margin: 0 auto;
                width: 100%;
                min-height: 300px;
                position: relative;
            ">
                {if !props.engagement.top_reactors.is_empty() {
                    html! {
                        <>
                            {{
                                // Sort reactors by interaction count and calculate sizes
                                // Display all top reactors (up to 10)
                                let mut reactors_with_sizes: Vec<_> = props.engagement.top_reactors.iter()
                                    .map(|reactor| {
                                        // Calculate bubble size based on interaction count
                                        // Use a base size and scale based on count
                                        let max_count = props.engagement.top_reactors.iter()
                                            .map(|r| r.interaction_count)
                                            .max()
                                            .unwrap_or(1);
                                        let min_size = 80.0;
                                        let max_size = 140.0;
                                        let size = if max_count > 0 {
                                            min_size + (reactor.interaction_count as f32 / max_count as f32) * (max_size - min_size)
                                        } else {
                                            min_size
                                        };
                                        
                                        // Generate random offsets for positioning
                                        // Use FID as seed for consistent positioning
                                        let seed = reactor.fid as u64;
                                        let offset_x = ((seed * 7) % 100) as i32 - 50; // -50 to 50
                                        let offset_y = ((seed * 13) % 100) as i32 - 50; // -50 to 50
                                        
                                        (reactor, size, offset_x, offset_y)
                                    })
                                    .collect();
                                
                                reactors_with_sizes.sort_by(|a, b| b.0.interaction_count.cmp(&a.0.interaction_count));
                                
                                html! {
                                    <>
                                        {for reactors_with_sizes.iter().enumerate().map(|(idx, (reactor, size, offset_x, offset_y))| {
                                            let bubble_size = format!("{}px", *size as i32);
                                            let avatar_url = reactor.pfp_url.as_ref().cloned();
                                            let username = reactor.username.as_ref()
                                                .or(reactor.display_name.as_ref())
                                                .map(|n| n.clone())
                                                .unwrap_or_else(|| format!("FID {}", reactor.fid));
                                            
                                            html! {
                                                <div style={format!("
                                                    position: relative;
                                                    width: {};
                                                    height: {};
                                                    display: flex;
                                                    flex-direction: column;
                                                    align-items: center;
                                                    justify-content: center;
                                                    background: rgba(255, 255, 255, 0.15);
                                                    backdrop-filter: blur(15px);
                                                    -webkit-backdrop-filter: blur(15px);
                                                    border-radius: 50%;
                                                    border: 2px solid rgba(255, 255, 255, 0.3);
                                                    padding: 16px;
                                                    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
                                                    transform: translate({}px, {}px);
                                                    animation: float 3s ease-in-out infinite;
                                                    animation-delay: {}s;
                                                ", bubble_size, bubble_size, offset_x, offset_y, idx as f32 * 0.3)}>
                                                    <div style="
                                                        width: 60%;
                                                        height: 60%;
                                                        border-radius: 50%;
                                                        background: rgba(255, 255, 255, 0.2);
                                                        display: flex;
                                                        align-items: center;
                                                        justify-content: center;
                                                        margin-bottom: 8px;
                                                        overflow: hidden;
                                                    ">
                                                        {if let Some(url) = avatar_url {
                                                            html! {
                                                                <img 
                                                                    src={url}
                                                                    alt=""
                                                                    style="
                                                                        width: 100%;
                                                                        height: 100%;
                                                                        object-fit: cover;
                                                                    "
                                                                />
                                                            }
                                                        } else {
                                                            // Show loading state while fetching
                                                            html! {
                                                                <div style="
                                                                    width: 100%;
                                                                    height: 100%;
                                                                    display: flex;
                                                                    align-items: center;
                                                                    justify-content: center;
                                                                    background: rgba(255, 255, 255, 0.1);
                                                                "></div>
                                                            }
                                                        }}
                                                    </div>
                                                    <div style="
                                                        font-size: 12px;
                                                        font-weight: 600;
                                                        color: white;
                                                        text-align: center;
                                                        margin-top: 4px;
                                                        text-overflow: ellipsis;
                                                        overflow: hidden;
                                                        white-space: nowrap;
                                                        max-width: 100%;
                                                    ">
                                                        {username}
                                                    </div>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }}
                            <style>{"
                                @keyframes float {
                                    0%, 100% {
                                        transform: translateY(0px);
                                    }
                                    50% {
                                        transform: translateY(-10px);
                                    }
                                }
                            "}</style>
                        </>
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
pub struct EngagementSectionProps {
    pub engagement: EngagementResponse,
    pub engagement_2024: Option<EngagementResponse>,
}

#[function_component]
pub fn EngagementSection(props: &EngagementSectionProps) -> Html {
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
                            background: rgba(102, 126, 234, 0.2);
                            backdrop-filter: blur(10px);
                            -webkit-backdrop-filter: blur(10px);
                            border-radius: 16px;
                            padding: 24px;
                            border: 1px solid rgba(118, 75, 162, 0.3);
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
pub struct EngagementQualitySectionProps {
    pub engagement: EngagementResponse,
    pub temporal: TemporalActivityResponse,
}

#[function_component]
pub fn EngagementQualitySection(props: &EngagementQualitySectionProps) -> Html {
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
pub struct GrowthTrendSectionProps {
    pub followers: FollowerGrowthResponse,
}

#[function_component]
pub fn GrowthTrendSection(props: &GrowthTrendSectionProps) -> Html {
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
pub struct ContentThemeSectionProps {
    pub style: ContentStyleResponse,
}

#[function_component]
pub fn ContentThemeSection(props: &ContentThemeSectionProps) -> Html {
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
pub struct StyleSectionProps {
    pub style: ContentStyleResponse,
    pub casts_stats: CastsStatsResponse,
}

#[function_component]
pub fn StyleSection(props: &StyleSectionProps) -> Html {
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
pub struct HighlightsSectionProps {
    pub temporal: TemporalActivityResponse,
}

#[function_component]
pub fn HighlightsSection(props: &HighlightsSectionProps) -> Html {
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
pub struct CallToActionSectionProps {
    pub profile: Option<ProfileWithRegistration>,
    pub annual_report: Option<AnnualReportResponse>,
}

#[function_component]
pub fn CallToActionSection(props: &CallToActionSectionProps) -> Html {
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

// Personality Tag Section Component - Classifies user into one tag
#[derive(Properties, PartialEq, Clone)]
pub struct PersonalityTagSectionProps {
    pub temporal: TemporalActivityResponse,
    pub engagement: EngagementResponse,
    pub content_style: ContentStyleResponse,
    pub follower_growth: FollowerGrowthResponse,
    pub casts_stats: CastsStatsResponse,
}

#[derive(Clone, PartialEq)]
struct PersonalityTag {
    name: String,
    description: String,
    image_path: String,
    score: f32,
}

#[function_component]
pub fn PersonalityTagSection(props: &PersonalityTagSectionProps) -> Html {
    let total_casts = props.temporal.total_casts.max(props.casts_stats.total_casts);
    
    // Calculate scores for each tag
    let mut tags = Vec::new();
    
    // 1. Night Philosopher - Check late night activity (0-5 AM)
    let late_night_activity: usize = props
        .temporal
        .hourly_distribution
        .iter()
        .filter(|h| h.hour >= 0 && h.hour < 6)
        .map(|h| h.count)
        .sum();
    let late_night_ratio = if total_casts > 0 {
        late_night_activity as f32 / total_casts as f32
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "Night Philosopher".to_string(),
        description: "Often shares deep thoughts in the early hours".to_string(),
        image_path: "/imgs/Philosopher.png".to_string(),
        score: late_night_ratio * 100.0,
    });
    
    // 2. Meme Merchant - Check emoji usage
    let total_emoji_count: usize = props.content_style.top_emojis.iter().map(|e| e.count).sum();
    let emoji_ratio = if total_casts > 0 {
        (total_emoji_count as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    let emoji_diversity_score = (props.content_style.top_emojis.len() as f32 / 10.0).min(1.0) * 30.0;
    tags.push(PersonalityTag {
        name: "Meme Merchant".to_string(),
        description: "High-frequency meme creator and sharer".to_string(),
        image_path: "/imgs/meme.png".to_string(),
        score: emoji_ratio * 70.0 + emoji_diversity_score,
    });
    
    // 3. Alpha Curator - Check recast ratio
    let recast_ratio = if total_casts > 0 {
        (props.engagement.recasts_received as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "Alpha Curator".to_string(),
        description: "Frequently shares quality content from others".to_string(),
        image_path: "/imgs/alpha.png".to_string(),
        score: recast_ratio * 100.0,
    });
    
    // 4. Social Butterfly - Check interaction rate
    let interaction_rate = if total_casts > 0 {
        ((props.engagement.reactions_received + props.engagement.replies_received) as f32 / total_casts as f32).min(10.0)
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "Social Butterfly".to_string(),
        description: "Highly engaged with exceptional interaction rates".to_string(),
        image_path: "/imgs/meme.png".to_string(), // Using meme.png as fallback, can be updated later
        score: (interaction_rate / 10.0) * 100.0,
    });
    
    // 5. Rising Star - Check follower growth
    let growth_rate = if props.follower_growth.followers_at_start > 0 {
        (props.follower_growth.net_growth as f32 / props.follower_growth.followers_at_start.max(1) as f32).min(5.0)
    } else if props.follower_growth.net_growth > 0 {
        1.0 // New user with growth
    } else {
        0.0
    };
    let absolute_growth_score = (props.follower_growth.net_growth as f32 / 200.0).min(1.0) * 50.0;
    tags.push(PersonalityTag {
        name: "Rising Star".to_string(),
        description: "Rapidly growing with fast follower growth".to_string(),
        image_path: "/imgs/newstar.png".to_string(),
        score: (growth_rate / 5.0) * 50.0 + absolute_growth_score,
    });
    
    // Find the tag with highest score
    let matched_tag = tags.iter().max_by(|a, b| {
        a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal)
    }).cloned().unwrap_or_else(|| PersonalityTag {
        name: "Active User".to_string(),
        description: "An active member of the Farcaster community".to_string(),
        image_path: "/imgs/meme.png".to_string(),
        score: 0.0,
    });

    // Setup 3D card rotation effect using JavaScript injection
    use_effect_with((), move |_| {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        
        // Check if script already exists
        if document.get_element_by_id("tarot-card-script").is_none() {
            let script = document.create_element("script").unwrap();
            script.set_id("tarot-card-script");
            script.set_text_content(Some(r#"
                (function() {
                    function initTarotCard(cardElement) {
                        if (cardElement.dataset.initialized === 'true') return;
                        cardElement.dataset.initialized = 'true';
                        
                        const inner = cardElement.querySelector('.tarot-card-inner');
                        if (!inner) return;
                        
                        let isDragging = false;
                        let startX = 0;
                        let startY = 0;
                        let rotateX = 0;
                        let rotateY = 0;
                        
                        const handleStart = (clientX, clientY) => {
                            isDragging = true;
                            startX = clientX;
                            startY = clientY;
                        };
                        
                        const handleMove = (clientX, clientY) => {
                            if (!isDragging) return;
                            const deltaX = clientX - startX;
                            const deltaY = clientY - startY;
                            rotateY = deltaX * 0.5;
                            rotateX = -deltaY * 0.5;
                            inner.style.transform = `rotateY(${rotateY}deg) rotateX(${rotateX}deg)`;
                        };
                        
                        const handleEnd = () => {
                            if (!isDragging) return;
                            isDragging = false;
                            const returnAnimation = () => {
                                rotateY *= 0.9;
                                rotateX *= 0.9;
                                inner.style.transform = `rotateY(${rotateY}deg) rotateX(${rotateX}deg)`;
                                if (Math.abs(rotateY) > 0.1 || Math.abs(rotateX) > 0.1) {
                                    requestAnimationFrame(returnAnimation);
                                } else {
                                    rotateY = 0;
                                    rotateX = 0;
                                    inner.style.transform = 'rotateY(0deg) rotateX(0deg)';
                                }
                            };
                            requestAnimationFrame(returnAnimation);
                        };
                        
                        cardElement.addEventListener('mousedown', (e) => {
                            e.preventDefault();
                            handleStart(e.clientX, e.clientY);
                        });
                        
                        const mousemoveHandler = (e) => {
                            if (isDragging) handleMove(e.clientX, e.clientY);
                        };
                        document.addEventListener('mousemove', mousemoveHandler);
                        
                        document.addEventListener('mouseup', handleEnd);
                        
                        cardElement.addEventListener('touchstart', (e) => {
                            e.preventDefault();
                            if (e.touches.length > 0) {
                                handleStart(e.touches[0].clientX, e.touches[0].clientY);
                            }
                        });
                        
                        cardElement.addEventListener('touchmove', (e) => {
                            e.preventDefault();
                            if (isDragging && e.touches.length > 0) {
                                handleMove(e.touches[0].clientX, e.touches[0].clientY);
                            }
                        });
                        
                        cardElement.addEventListener('touchend', handleEnd);
                    }
                    
                    // Initialize all tarot cards
                    function initAllCards() {
                        const cards = document.querySelectorAll('.tarot-card');
                        cards.forEach(initTarotCard);
                    }
                    
                    // Initialize when DOM is ready
                    if (document.readyState === 'loading') {
                        document.addEventListener('DOMContentLoaded', initAllCards);
                    } else {
                        initAllCards();
                    }
                    
                    // Also use MutationObserver to handle dynamically added cards
                    const observer = new MutationObserver(initAllCards);
                    observer.observe(document.body, { childList: true, subtree: true });
                })();
            "#));
            document.head().unwrap().append_child(&script).ok();
        }
        
        // Initialize cards after a short delay to ensure DOM is ready
        let timeout_closure = Closure::<dyn FnMut()>::new(move || {
            // Use JavaScript eval to initialize cards
            let js_code = r#"
                const cards = document.querySelectorAll('.tarot-card');
                cards.forEach(card => {
                    if (card.dataset.initialized === 'true') return;
                    card.dataset.initialized = 'true';
                    const inner = card.querySelector('.tarot-card-inner');
                    if (!inner) return;
                    let isDragging = false;
                    let startX = 0, startY = 0;
                    let currentRotateX = 0, currentRotateY = 0;
                    let baseRotateY = 0;
                    
                    const handleStart = (x, y) => { 
                        isDragging = true; 
                        startX = x; 
                        startY = y; 
                    };
                    
                    const handleMove = (x, y) => {
                        if (!isDragging) return;
                        const deltaX = x - startX;
                        const deltaY = y - startY;
                        
                        // Accumulate rotation for 360 degree rotation
                        baseRotateY += deltaX * 0.5;
                        currentRotateX = -deltaY * 0.5;
                        
                        // Normalize rotateY to 0-360 range (for continuous rotation)
                        baseRotateY = ((baseRotateY % 360) + 360) % 360;
                        
                        inner.style.transform = `rotateY(${baseRotateY}deg) rotateX(${currentRotateX}deg)`;
                    };
                    
                    const handleEnd = () => {
                        if (!isDragging) return;
                        isDragging = false;
                        const animate = () => {
                            currentRotateX *= 0.9;
                            inner.style.transform = `rotateY(${baseRotateY}deg) rotateX(${currentRotateX}deg)`;
                            if (Math.abs(currentRotateX) > 0.1) {
                                requestAnimationFrame(animate);
                            } else {
                                currentRotateX = 0;
                                inner.style.transform = `rotateY(${baseRotateY}deg) rotateX(0deg)`;
                            }
                        };
                        requestAnimationFrame(animate);
                    };
                    
                    card.addEventListener('mousedown', (e) => { 
                        e.preventDefault(); 
                        handleStart(e.clientX, e.clientY); 
                    });
                    document.addEventListener('mousemove', (e) => { 
                        if (isDragging) handleMove(e.clientX, e.clientY); 
                    });
                    document.addEventListener('mouseup', handleEnd);
                    card.addEventListener('touchstart', (e) => { 
                        e.preventDefault(); 
                        if (e.touches[0]) handleStart(e.touches[0].clientX, e.touches[0].clientY); 
                    });
                    card.addEventListener('touchmove', (e) => { 
                        e.preventDefault(); 
                        if (isDragging && e.touches[0]) handleMove(e.touches[0].clientX, e.touches[0].clientY); 
                    });
                    card.addEventListener('touchend', handleEnd);
                });
            "#;
            let _ = js_sys::eval(js_code);
        });
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout_closure.as_ref().unchecked_ref::<js_sys::Function>(),
            200
        );
        timeout_closure.forget();
        
        || ()
    });

    html! {
        <div class="report-card-content" style="
            width: 100%;
            height: 100%;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 40px;
            box-sizing: border-box;
        ">
            <div style="
                text-align: center;
                width: 100%;
                max-width: 600px;
            ">
                <div 
                    class="tarot-card"
                    style="
                        width: 280px;
                        height: 400px;
                        margin: 0 auto 32px;
                        perspective: 1000px;
                        cursor: pointer;
                    "
                >
                    <div 
                        class="tarot-card-inner"
                        style="
                            position: relative;
                            width: 100%;
                            height: 100%;
                            transform-style: preserve-3d;
                            transition: transform 0.1s ease-out;
                        "
                    >
                        <div
                            class="tarot-card-front"
                            style="
                                position: absolute;
                                width: 100%;
                                height: 100%;
                                backface-visibility: hidden;
                                -webkit-backface-visibility: hidden;
                                transform: rotateY(0deg);
                            "
                        >
                            <img 
                                src={matched_tag.image_path.clone()}
                                alt={matched_tag.name.clone()}
                                style="
                                    width: 100%;
                                    height: 100%;
                                    object-fit: cover;
                                    border-radius: 16px;
                                    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
                                    border: 3px solid rgba(255, 255, 255, 0.3);
                                "
                            />
                        </div>
                        <div
                            class="tarot-card-back"
                            style="
                                position: absolute;
                                width: 100%;
                                height: 100%;
                                backface-visibility: hidden;
                                -webkit-backface-visibility: hidden;
                                transform: rotateY(180deg);
                                border-radius: 16px;
                                box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
                                background: linear-gradient(90deg, #ff0000, #ff7f00, #ffff00, #00ff00, #0000ff, #4b0082, #9400d3, #ff0000);
                                background-size: 200% 100%;
                                animation: rainbow-border-animation 3s linear infinite;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                padding: 2px;
                                box-sizing: border-box;
                            "
                        >
                            <div style="
                                position: relative;
                                width: 100%;
                                height: 100%;
                                border-radius: 14px;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                padding: 40px;
                                box-sizing: border-box;
                            ">
                                <img 
                                    src="/imgs/polyjuice.png"
                                    alt="Polyjuice"
                                    class="embossed-logo"
                                    style="
                                        width: 100%;
                                        height: auto;
                                        max-width: 200px;
                                        object-fit: contain;
                                        filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.6)) drop-shadow(-1px -1px 2px rgba(255, 255, 255, 0.4)) brightness(1.1) contrast(1.2);
                                        opacity: 0.95;
                                    "
                                />
                            </div>
                        </div>
                    </div>
                </div>
                
                <h2 style="
                    font-size: 36px;
                    font-weight: 700;
                    color: white;
                    margin: 0 0 16px 0;
                    text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
                ">{matched_tag.name.clone()}</h2>
                
                <p style="
                    font-size: 18px;
                    color: rgba(255, 255, 255, 0.9);
                    margin: 0 0 40px 0;
                    line-height: 1.6;
                ">{matched_tag.description.clone()}</p>

                <div style="
                    background: rgba(255, 255, 255, 0.1);
                    backdrop-filter: blur(10px);
                    -webkit-backdrop-filter: blur(10px);
                    border-radius: 16px;
                    padding: 24px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    margin-top: 20px;
                ">
                    <p style="
                        font-size: 14px;
                        color: rgba(255, 255, 255, 0.7);
                        margin: 0;
                        line-height: 1.5;
                    ">{"This tag is based on your activity patterns, engagement style, and growth metrics throughout the year."}</p>
                </div>
            </div>
            <style>{r#"
                .tarot-card {
                    touch-action: none;
                }
                
                .tarot-card-inner {
                    will-change: transform;
                }
                
                @media (hover: hover) {
                    .tarot-card:hover .tarot-card-inner {
                        transform: rotateY(5deg) rotateX(5deg);
                    }
                }
                
                @keyframes rainbow-border-animation {
                    0% {
                        background-position: 0% 50%;
                    }
                    100% {
                        background-position: 200% 50%;
                    }
                }
            "#}</style>
            <script>{r#"
                (function() {
                    const cards = document.querySelectorAll('.tarot-card');
                    cards.forEach(card => {
                        const inner = card.querySelector('.tarot-card-inner');
                        let isDragging = false;
                        let startX = 0;
                        let startY = 0;
                        let currentX = 0;
                        let currentY = 0;
                        let rotateX = 0;
                        let rotateY = 0;
                        
                        const handleStart = (clientX, clientY) => {
                            isDragging = true;
                            startX = clientX;
                            startY = clientY;
                        };
                        
                        const handleMove = (clientX, clientY) => {
                            if (!isDragging) return;
                            
                            const deltaX = clientX - startX;
                            const deltaY = clientY - startY;
                            
                            rotateY = deltaX * 0.5;
                            rotateX = -deltaY * 0.5;
                            
                            inner.style.transform = `rotateY(${rotateY}deg) rotateX(${rotateX}deg)`;
                        };
                        
                        const handleEnd = () => {
                            if (!isDragging) return;
                            isDragging = false;
                            
                            // Smooth return to center
                            const returnAnimation = () => {
                                rotateY *= 0.9;
                                rotateX *= 0.9;
                                inner.style.transform = `rotateY(${rotateY}deg) rotateX(${rotateX}deg)`;
                                
                                if (Math.abs(rotateY) > 0.1 || Math.abs(rotateX) > 0.1) {
                                    requestAnimationFrame(returnAnimation);
                                } else {
                                    rotateY = 0;
                                    rotateX = 0;
                                    inner.style.transform = 'rotateY(0deg) rotateX(0deg)';
                                }
                            };
                            requestAnimationFrame(returnAnimation);
                        };
                        
                        // Mouse events
                        card.addEventListener('mousedown', (e) => {
                            e.preventDefault();
                            handleStart(e.clientX, e.clientY);
                        });
                        
                        document.addEventListener('mousemove', (e) => {
                            if (isDragging) handleMove(e.clientX, e.clientY);
                        });
                        
                        document.addEventListener('mouseup', () => {
                            handleEnd();
                        });
                        
                        // Touch events
                        card.addEventListener('touchstart', (e) => {
                            e.preventDefault();
                            const touch = e.touches[0];
                            handleStart(touch.clientX, touch.clientY);
                        });
                        
                        card.addEventListener('touchmove', (e) => {
                            e.preventDefault();
                            if (isDragging && e.touches.length > 0) {
                                const touch = e.touches[0];
                                handleMove(touch.clientX, touch.clientY);
                            }
                        });
                        
                        card.addEventListener('touchend', () => {
                            handleEnd();
                        });
                    });
                })();
            "#}</script>
        </div>
    }
}
