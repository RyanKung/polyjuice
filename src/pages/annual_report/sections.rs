use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;

use super::utils::farcaster_to_unix;
use crate::farcaster;
use crate::models::AnnualReportResponse;
use crate::models::CastsStatsResponse;
use crate::models::ContentStyleResponse;
use crate::models::EngagementResponse;
use crate::models::FollowerGrowthResponse;
use crate::models::ProfileWithRegistration;
use crate::models::TemporalActivityResponse;

// Unified styles for annual report sections
const REPORT_CARD_CONTENT_STYLE: &str = "
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
";

const REPORT_SECTION_TITLE_STYLE: &str = "
    font-size: 36px;
    font-weight: 700;
    margin: 0 0 32px 0;
    color: white;
    text-align: center;
";

const REPORT_CONTENT_CONTAINER_STYLE: &str = "
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
";

const REPORT_INFO_CARD_STYLE: &str = "
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
    border-radius: 16px;
    padding: 24px;
    border: 1px solid rgba(255, 255, 255, 0.2);
";

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

// Helper function to get zodiac sign from date (month and day)
fn get_zodiac_sign(month: u32, day: u32) -> &'static str {
    match (month, day) {
        (1, 1..=19) | (12, 22..=31) => "Capricorn",
        (1, 20..=31) | (2, 1..=18) => "Aquarius",
        (2, 19..=29) | (3, 1..=20) => "Pisces",
        (3, 21..=31) | (4, 1..=19) => "Aries",
        (4, 20..=30) | (5, 1..=20) => "Taurus",
        (5, 21..=31) | (6, 1..=20) => "Gemini",
        (6, 21..=30) | (7, 1..=22) => "Cancer",
        (7, 23..=31) | (8, 1..=22) => "Leo",
        (8, 23..=31) | (9, 1..=22) => "Virgo",
        (9, 23..=30) | (10, 1..=22) => "Libra",
        (10, 23..=31) | (11, 1..=21) => "Scorpio",
        (11, 22..=30) | (12, 1..=21) => "Sagittarius",
        _ => "Unknown",
    }
}

// Helper function to get zodiac emoji/symbol
fn get_zodiac_symbol(zodiac: &str) -> &'static str {
    match zodiac {
        "Capricorn" => "♑",
        "Aquarius" => "♒",
        "Pisces" => "♓",
        "Aries" => "♈",
        "Taurus" => "♉",
        "Gemini" => "♊",
        "Cancer" => "♋",
        "Leo" => "♌",
        "Virgo" => "♍",
        "Libra" => "♎",
        "Scorpio" => "♏",
        "Sagittarius" => "♐",
        _ => "⭐",
    }
}

// Helper function to get far zodiac sign based on FID
fn get_far_zodiac_sign(fid: i64) -> &'static str {
    let zodiacs = [
        "Capricorn", "Aquarius", "Pisces", "Aries", "Taurus", "Gemini",
        "Cancer", "Leo", "Virgo", "Libra", "Scorpio", "Sagittarius",
    ];
    let index = (fid % 12) as usize;
    zodiacs[index]
}

#[function_component]
pub fn IdentitySection(props: &IdentitySectionProps) -> Html {
    // Get registration date and calculate zodiac signs
    let (birthday_date, zodiac_image_url, zodiac_info) = props.profile.registered_at.map(|timestamp| {
        let unix_timestamp = farcaster_to_unix(timestamp);
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(unix_timestamp as f64 * 1000.0));
        let month = date.get_month() as u32 + 1; // get_month returns 0-11
        let day = date.get_date() as u32;
        let year = date.get_full_year();
        let zodiac = get_zodiac_sign(month, day);
        let far_zodiac = get_far_zodiac_sign(props.profile.fid);
        let zodiac_info = format!("{}-{}", zodiac, far_zodiac);
        let birthday_date = format!("{}/{:02}/{:02}", year, month, day);
        // Build image URL from zodiac name (convert to lowercase)
        let zodiac_lower = zodiac.to_lowercase();
        let zodiac_image_url = format!("/imgs/zodiac/{}.png", zodiac_lower);
        (birthday_date, zodiac_image_url, zodiac_info)
    }).unwrap_or_else(|| ("N/A".to_string(), "/imgs/zodiac/capricorn.png".to_string(), "N/A".to_string()));

    // Get first cast date
    let first_cast_date = props.temporal.first_cast.as_ref().map(|cast| {
            let unix_timestamp = farcaster_to_unix(cast.timestamp);
        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(unix_timestamp as f64 * 1000.0));
        let month = date.get_month() as u32 + 1;
        let day = date.get_date() as u32;
        let year = date.get_full_year();
        format!("{}/{:02}/{:02}", year, month, day)
    }).unwrap_or_else(|| "N/A".to_string());

    html! {
        <div class="report-card-content" style={REPORT_CARD_CONTENT_STYLE}
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
            <h2 style={REPORT_SECTION_TITLE_STYLE}>{zodiac_info.clone()}</h2>
            <div style={format!("{} width: 100%; max-width: 700px; margin: 0 auto;", REPORT_INFO_CARD_STYLE)}>
                <div style="
                    display: flex;
                    flex-direction: column;
                    gap: 16px;
                    font-size: 16px;
                    color: rgba(255, 255, 255, 0.85);
                    line-height: 1.6;
                ">
                    // Zodiac symbol image
                    <div style="
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        margin-bottom: 4px;
                    ">
                        <img
                            src={zodiac_image_url.clone()}
                            alt="Zodiac"
                            style="
                                width: 100px;
                                height: 100px;
                                object-fit: contain;
                            "
                        />
                    </div>
                    
                    // First cast quote - chat bubble style
                    {if let Some(first_cast) = &props.temporal.first_cast {
                        if !first_cast.text.trim().is_empty() {
                            html! {
                                <div style="
                                    display: flex;
                                    align-items: flex-start;
                                    gap: 12px;
                                    margin: 8px 0;
                                    padding: 0;
                                ">
                                    // Avatar
                                    <div style="
                                        width: 40px;
                                        height: 40px;
                                        border-radius: 50%;
                                        overflow: hidden;
                                        flex-shrink: 0;
                                        background: rgba(255, 255, 255, 0.1);
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                    ">
                                        {if let Some(pfp_url) = &props.profile.pfp_url {
                                            html! {
                                                <img
                                                    src={pfp_url.clone()}
                                                    alt="Avatar"
                                                    style="
                                                        width: 100%;
                                                        height: 100%;
                                                        object-fit: cover;
                                                    "
                                                />
                                            }
            } else {
                                            html! {
                                                <div style="
                                                    width: 100%;
                                                    height: 100%;
                                                    display: flex;
                                                    align-items: center;
                                                    justify-content: center;
                                                    font-size: 20px;
                                                    color: white;
                                                ">
                                                    {"👤"}
                                                </div>
                                            }
                                        }}
                                    </div>
                                    // Username and cast content
                                    <div style="
                                        flex: 1;
                                        display: flex;
                                        flex-direction: column;
                                        gap: 4px;
                                    ">
                                        <div style="
                                            font-size: 14px;
                                            font-weight: 600;
                                            color: white;
                                        ">
                                            {if let Some(username) = &props.profile.username {
                                                html! {
                                                    <span>{format!("@{}", username)}</span>
                                                }
                                            } else {
                                                html! {
                                                    <span>{format!("FID: {}", props.profile.fid)}</span>
                                                }
                                            }}
                                            <span style="
                                                margin: 0 8px;
                                                color: rgba(255, 255, 255, 0.6);
                                            ">{">"}</span>
                                        </div>
                                        <div style="
                                            font-size: 16px;
                                            color: rgba(255, 255, 255, 0.85);
                                            line-height: 1.6;
                                            word-wrap: break-word;
                                        ">
                                            {first_cast.text.clone()}
                                        </div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    } else {
                        html! {}
                    }}
                    
                    <div>
                        {"On "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{birthday_date.clone()}</span>
                        {", you were born on Farcaster. This was the day you took your first step into this vibrant community."}
                    </div>
                    
                    <div>
                        {"Your first cast was on "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{first_cast_date.clone()}</span>
                        {". That moment marked the beginning of your voice in this digital realm."}
                    </div>
                    
                    <div>
                        {"Your Farcaster Zodiac is "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{zodiac_info.clone()}</span>
                        {", a unique combination that reflects both your birth date and your Farcaster identity."}
                    </div>
                </div>
            </div>
        </div>
    }
}

// Follower Growth Section Component
#[derive(Properties, PartialEq, Clone)]
pub struct FollowerGrowthSectionProps {
    pub followers: FollowerGrowthResponse,
    pub temporal: TemporalActivityResponse,
    pub engagement: EngagementResponse,
    pub profile: ProfileWithRegistration,
}

#[function_component]
pub fn FollowerGrowthSection(props: &FollowerGrowthSectionProps) -> Html {
    let follower_change =
        props.followers.current_followers as i64 - props.followers.followers_at_start as i64;
    
    // Calculate total casts and average per week
    let total_casts = props.temporal.total_casts_in_year.unwrap_or(props.temporal.total_casts);
    let avg_per_week = (total_casts as f32 / 52.0).round() as usize;
    
    // Determine personality trait based on average casts per week
    // If average >= 3 casts per week, consider "talkative", otherwise "reserved"
    let personality_trait = if avg_per_week >= 3 {
        "talkative"
    } else {
        "reserved"
    };
    
    // Get most active month
    let most_active_month = props
        .temporal
        .monthly_distribution
        .iter()
        .max_by_key(|m| m.count)
        .map(|m| {
            let parts: Vec<&str> = m.month.split('-').collect();
            if parts.len() >= 2 {
                let month_num: u32 = parts[1].parse().unwrap_or(1);
                match month_num {
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
                }
            } else {
                "N/A"
            }
        })
        .unwrap_or("N/A");
    
    // Get most active hour
    let most_active_hour = props
        .temporal
        .most_active_hour
        .map(|h| format!("{}:00", h))
        .unwrap_or_else(|| "N/A".to_string());
    
    // Determine social type image and title based on total casts
    let (social_type_image, section_title) = if total_casts >= 200 {
        ("/imgs/social_type/social.png", "Social Butterfly")
    } else {
        ("/imgs/social_type/slient.png", "Man of Few Words")
    };

    html! {
        <div class="report-card-content" style={REPORT_CARD_CONTENT_STYLE}
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
            <h2 style={REPORT_SECTION_TITLE_STYLE}>{section_title}</h2>
            <div style={format!("{} width: 100%; max-width: 700px; margin: 0 auto;", REPORT_INFO_CARD_STYLE)}>
            <div style="
                display: flex;
                flex-direction: column;
                    gap: 16px;
                    font-size: 16px;
                    color: rgba(255, 255, 255, 0.85);
                    line-height: 1.6;
            ">
                    // Social type image
                <div style="
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        margin-bottom: 4px;
                    ">
                        <img
                            src={social_type_image}
                            alt="Social Type"
                            style="
                                width: 100px;
                                height: 100px;
                                object-fit: contain;
                            "
                        />
                </div>
                    
                    // Popular cast quote - chat bubble style
                    {if let Some(popular_cast) = &props.engagement.most_popular_cast {
                        if !popular_cast.text.trim().is_empty() {
                            html! {
                <div style="
                                    display: flex;
                                    align-items: flex-start;
                                    gap: 12px;
                                    margin: 8px 0;
                                    padding: 0;
                                ">
                                    // Avatar
                                    <div style="
                                        width: 40px;
                                        height: 40px;
                                        border-radius: 50%;
                                        overflow: hidden;
                                        flex-shrink: 0;
                    background: rgba(255, 255, 255, 0.1);
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                    ">
                                        {if let Some(pfp_url) = &props.profile.pfp_url {
                            html! {
                                                <img
                                                    src={pfp_url.clone()}
                                                    alt="Avatar"
                                                    style="
                                                        width: 100%;
                                                        height: 100%;
                                                        object-fit: cover;
                                                    "
                                                />
                                            }
                                        } else {
                                            html! {
                                                <div style="
                                                    width: 100%;
                                                    height: 100%;
                                                    display: flex;
                                                    align-items: center;
                                                    justify-content: center;
                                                    font-size: 20px;
                                                    color: white;
                                                ">
                                                    {"👤"}
                                                </div>
                                            }
                                        }}
                                    </div>
                                    // Username and cast content
                                    <div style="
                                        flex: 1;
                                        display: flex;
                                        flex-direction: column;
                                        gap: 4px;
                                    ">
                                        <div style="
                                            font-size: 14px;
                                            font-weight: 600;
                                            color: white;
                                        ">
                                            {if let Some(username) = &props.profile.username {
                                                html! {
                                                    <span>{format!("@{}", username)}</span>
                                                }
                                            } else {
                                                html! {
                                                    <span>{format!("FID: {}", props.profile.fid)}</span>
                                                }
                                            }}
                                            <span style="
                                                margin: 0 8px;
                                                color: rgba(255, 255, 255, 0.6);
                                            ">{">"}</span>
                                        </div>
                                        <div style="
                                            font-size: 16px;
                                            color: rgba(255, 255, 255, 0.85);
                                            line-height: 1.6;
                                            word-wrap: break-word;
                                        ">
                                            {popular_cast.text.clone()}
                                        </div>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                            }
                        } else {
                            html! {}
                        }}
                    
                    <div>
                        {"This year, you published "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{total_casts.to_string()}</span>
                        {" messages in total, averaging "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{avg_per_week.to_string()}</span>
                        {" per week. It shows you are "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{personality_trait}</span>
                        {"."}
                </div>
                    
                    <div>
                        {"Your most active month was "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{most_active_month}</span>
                        {", and you always start sharing your life at "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{most_active_hour.clone()}</span>
                        {"."}
                </div>
                    
                    {if let Some(popular_cast) = &props.engagement.most_popular_cast {
                        html! {
                            <div>
                                {"This year, your voice was heard. The most popular one received "}
                                <span style="font-weight: 700; font-size: 18px; color: white;">{popular_cast.reactions.to_string()}</span>
                                {" likes, "}
                                <span style="font-weight: 700; font-size: 18px; color: white;">{popular_cast.recasts.to_string()}</span>
                                {" recasts, and "}
                                <span style="font-weight: 700; font-size: 18px; color: white;">{popular_cast.replies.to_string()}</span>
                                {" replies."}
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                    
                    <div>
                        {"You have "}
                        <span style="font-weight: 700; font-size: 18px; color: white;">{props.followers.current_followers.to_string()}</span>
                        {" followers"}
                        {if follower_change > 0 {
                            html! {
                                <>
                                    {", "}
                                    <span style="font-weight: 700; font-size: 18px; color: white;">{follower_change.to_string()}</span>
                                    {" of which were gained this year."}
                                </>
                            }
                        } else {
                            html! {"."}
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

    // Format most active month as "February" (without zodiac sign)
    let most_active_month = props
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
                month_name.to_string()
            } else {
                "N/A".to_string()
            }
        })
        .unwrap_or_else(|| "N/A".to_string());

    let most_active_hour = props
        .temporal
        .most_active_hour
        .map(|h| format!("{}:00", h))
        .unwrap_or_else(|| "N/A".to_string());

    let _max_monthly_count = props
        .temporal
        .monthly_distribution
        .iter()
        .map(|m| m.count)
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
                                    let height_percent = month.count as f32 / max_monthly_count as f32 * 100.0;
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
                    let rainbow_colors = [
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
                                    let height_percent = hour.count as f32 / max_hourly_count as f32 * 100.0;
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
    pub current_user_fid: Option<i64>,
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
            ">{"Friendships You've Gained"}</h2>
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
                                // Display all top reactors (up to 10), excluding current user
                                let filtered_reactors: Vec<_> = props.engagement.top_reactors.iter()
                                    .filter(|reactor| {
                                        // Exclude current user if FID matches
                                        if let Some(current_fid) = props.current_user_fid {
                                            reactor.fid != current_fid
                                        } else {
                                            true
                                        }
                                    })
                                    .collect();
                                
                                let max_count = filtered_reactors.iter()
                                            .map(|r| r.interaction_count)
                                            .max()
                                            .unwrap_or(1);
                                
                                let mut reactors_with_sizes: Vec<_> = filtered_reactors.iter()
                                    .map(|reactor| {
                                        // Calculate bubble size based on interaction count
                                        // Use a base size and scale based on count
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
                                                .or(reactor.display_name.as_ref()).cloned()
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
    let total_casts = props
        .temporal
        .total_casts_in_year
        .unwrap_or(props.temporal.total_casts);
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
            / props.followers.followers_at_start as f32)
            * 100.0
    } else {
        0.0
    };

    // Find month with highest growth
    let max_growth_month = props
        .followers
        .monthly_snapshots
        .iter()
        .zip(props.followers.monthly_snapshots.iter().skip(1))
        .map(|(prev, curr)| {
            let growth = curr.followers as i32 - prev.followers as i32;
            (curr.month.clone(), growth)
        })
        .max_by_key(|(_, growth)| *growth);

    let max_followers = props
        .followers
        .monthly_snapshots
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
                    let rainbow_colors = [
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
                                    let height_percent = snapshot.followers as f32 / max_followers as f32 * 100.0;
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

// Style Section Component
#[derive(Properties, PartialEq, Clone)]
pub struct StyleSectionProps {
    pub style: ContentStyleResponse,
    pub casts_stats: CastsStatsResponse,
    pub profile: ProfileWithRegistration,
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
            ">{"Your Style"}</h2>
            <div style="
                display: flex;
                flex-direction: column;
                gap: 24px;
                max-width: 800px;
                margin: 0 auto;
                width: 100%;
            ">
                <div style="
                    width: 100%;
                    aspect-ratio: 1;
                    position: relative;
                    margin: 0 auto;
                    max-width: min(90vw, 500px);
                    transform-style: preserve-3d;
                    perspective: 1000px;
                ">
                    // User avatar in the center - fixed, not rotating
                    {{
                        let container_size = 500.0;
                            html! {
                            <div style={format!("
                                position: absolute;
                                left: 50%;
                                top: 50%;
                                transform: translate(-50%, -50%);
                                width: {}px;
                                height: {}px;
                                border-radius: 50%;
                                overflow: hidden;
                                z-index: 10;
                                border: 3px solid rgba(255, 255, 255, 0.3);
                                box-shadow: 0 0 20px rgba(0, 0, 0, 0.5);
                            ", 
                                (container_size * 0.25) as u32,  // 25% of container size
                                (container_size * 0.25) as u32
                            )}>
                                {if let Some(pfp_url) = &props.profile.pfp_url {
                                    html! {
                                        <img
                                            src={pfp_url.clone()}
                                            alt="Avatar"
                                            style="
                                                width: 100%;
                                                height: 100%;
                                                object-fit: cover;
                                            "
                                        />
                                    }
                                } else {
                                    html! {
                <div style="
                                            width: 100%;
                                            height: 100%;
                                            background: rgba(255, 255, 255, 0.2);
                        display: flex;
                        align-items: center;
                        justify-content: center;
                                            font-size: 48px;
                                            color: white;
                                        ">
                                            {"👤"}
                                        </div>
                                    }
                                }}
                            </div>
                        }
                    }}
                    <div style="
                        width: 100%;
                        height: 100%;
                        position: relative;
                        transform-style: preserve-3d;
                        animation: rotateSphere 30s linear infinite;
                    ">
                        <style>
                            {r#"
                            @keyframes rotateSphere {
                                from {
                                    transform: rotateY(0deg) rotateX(15deg);
                                }
                                to {
                                    transform: rotateY(360deg) rotateX(15deg);
                                }
                            }
                            "#}
                        </style>
                        {{
                            // Sort words by count (descending) to ensure highest frequency words are first
                            let mut sorted_words: Vec<_> = top_words.iter().enumerate().collect();
                            sorted_words.sort_by(|a, b| b.1.count.cmp(&a.1.count));
                            
                            let container_size = 500.0;
                            let center = container_size / 2.0;
                            let sphere_radius = container_size / 2.5; // Sphere radius in 3D space
                            let mut positions: Vec<(f32, f32, f32, f32, f32)> = Vec::new(); // (x_3d, y_3d, z_3d, font_size, rotation_y)
                            
                            // Distribute words evenly on a sphere surface using Fibonacci sphere algorithm
                            let total_words = sorted_words.len();
                            for (idx, (_original_idx, word)) in sorted_words.iter().enumerate() {
                            let size_ratio = word.count as f32 / max_count as f32;
                                let font_size = (18.0 + size_ratio * 28.0).clamp(18.0, 46.0);

                                // Fibonacci sphere algorithm - ensures even distribution on sphere surface
                                let golden_angle = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt());
                                let theta = golden_angle * idx as f32;
                                
                                // y ranges from -1 to 1 (top to bottom of sphere)
                                let y_normalized = 1.0 - (idx as f32 / (total_words - 1).max(1) as f32) * 2.0;
                                
                                // Calculate radius at this y level (circle cross-section)
                                let radius_at_y = (1.0 - y_normalized * y_normalized).sqrt();
                                
                                // Angle around the circle at this y level
                                let phi = theta % (2.0 * std::f32::consts::PI);
                                
                                // 3D coordinates on sphere surface (unit sphere)
                                let x_3d_unit = radius_at_y * phi.cos();
                                let z_3d_unit = radius_at_y * phi.sin();
                                let y_3d_unit = y_normalized;
                                
                                // Scale to sphere radius
                                let x_3d = x_3d_unit * sphere_radius;
                                let y_3d = y_3d_unit * sphere_radius;
                                let z_3d = z_3d_unit * sphere_radius;
                                
                                // Calculate rotation to face user (billboard effect)
                                // The text should rotate around Y axis to face the camera
                                // Angle is based on the position on the sphere
                                let rotation_y = phi.to_degrees();
                                
                                positions.push((x_3d, y_3d, z_3d, font_size, rotation_y));
                            }
                        
                        // Colors that stand out on purple background (avoid purple/violet)
                        let vibrant_colors = [
                            "#FFFFFF",      // White - very visible
                            "#FFFF00",      // Yellow - high contrast
                            "#00FFFF",      // Cyan - bright
                            "#00FF00",      // Green - vibrant
                            "#FFA500",      // Orange - warm
                            "#FF69B4",      // Hot Pink - bright
                            "#FFD700",      // Gold - rich
                            "#00CED1",      // Dark Turquoise - bright
                            "#FF1493",      // Deep Pink - vivid
                            "#32CD32",      // Lime Green - bright
                            "#FF4500",      // Orange Red - vibrant
                            "#1E90FF",      // Dodger Blue - bright
                        ];
                        
                        html! {
                            <>
                                {for sorted_words.iter().enumerate().map(|(display_idx, (original_idx, word))| {
                                    let (x_3d, y_3d, z_3d, font_size, rotation_y) = positions[display_idx];
                                    let size_ratio = word.count as f32 / max_count as f32;
                                    
                                    // Project 3D coordinates to 2D screen space (orthographic projection)
                                    // The sphere is centered at (center, center) in 2D space
                                    let x_2d = center + x_3d;
                                    let y_2d = center + y_3d;
                                    
                                    // Use z-depth for opacity and scale (3D effect)
                                    // z ranges from -sphere_radius to +sphere_radius
                                    let z_normalized = (z_3d + sphere_radius) / (2.0 * sphere_radius); // 0 to 1
                                    let opacity = 0.6 + z_normalized * 0.4; // 0.6 to 1.0 (back to front)
                                    let scale_3d = 0.7 + z_normalized * 0.3; // 0.7 to 1.0 (back smaller, front larger)
                                    
                                    // Select color based on index, avoiding purple
                                    let color_idx = (*original_idx + (word.word.len() % vibrant_colors.len())) % vibrant_colors.len();
                                    let color = vibrant_colors[color_idx];

                            html! {
                                <span style={format!("
                                            position: absolute;
                                            left: {}%;
                                            top: {}%;
                                            transform: translate(-50%, -50%) translateZ({}px) rotateY({}deg) scale({});
                                    font-size: {}px;
                                    font-weight: {};
                                    color: {};
                                            opacity: {};
                                            white-space: nowrap;
                                            pointer-events: none;
                                            user-select: none;
                                            text-shadow: 0 0 8px rgba(0, 0, 0, 0.5), 0 2px 4px rgba(0, 0, 0, 0.3);
                                            transform-style: preserve-3d;
                                ",
                                            (x_2d / container_size) * 100.0,
                                            (y_2d / container_size) * 100.0,
                                            z_3d,
                                            rotation_y,
                                            scale_3d,
                                    font_size,
                                            if size_ratio > 0.5 { "700" } else { "600" },
                                            color,
                                            opacity
                                )}>
                                    {word.word.clone()}
                                </span>
                            }
                        })}
                            </>
                        }
                    }}
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
    pub is_farcaster_env: bool,
    pub share_url: Option<String>,
    pub is_own_report: bool,
    pub current_user_fid: Option<i64>,
}

// Helper function to copy text to clipboard (async version for modern Clipboard API)
async fn copy_to_clipboard_async(text: &str) -> bool {
    let window = web_sys::window().unwrap();

    // Try modern Clipboard API first using js_sys::Reflect
    if let Ok(navigator_val) = js_sys::Reflect::get(&window, &"navigator".into()) {
        if !navigator_val.is_null() && !navigator_val.is_undefined() {
            if let Ok(clipboard_val) = js_sys::Reflect::get(&navigator_val, &"clipboard".into()) {
                if !clipboard_val.is_null() && !clipboard_val.is_undefined() {
                    if let Ok(write_text_fn) =
                        js_sys::Reflect::get(&clipboard_val, &"writeText".into())
                    {
                        if let Some(write_fn) = write_text_fn.dyn_ref::<js_sys::Function>() {
                            if let Ok(promise_val) = write_fn.call1(&clipboard_val, &text.into()) {
                                if let Ok(promise) = promise_val.dyn_into::<js_sys::Promise>() {
                                    match JsFuture::from(promise).await {
                                        Ok(_) => {
                                            web_sys::console::log_1(
                                                &"✅ Text copied using Clipboard API".into(),
                                            );
                                            return true;
                                        }
                                        Err(e) => {
                                            web_sys::console::warn_1(
                                                &format!("⚠️ Clipboard API failed: {:?}", e).into(),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback: use document.execCommand
    let document = window.document().unwrap();
    let textarea = document.create_element("textarea").unwrap();
    let textarea_js: &wasm_bindgen::JsValue = textarea.as_ref();

    if js_sys::Reflect::set(textarea_js, &"value".into(), &text.into()).is_err() {
        return false;
    }

    let style = match js_sys::Reflect::get(textarea_js, &"style".into()) {
        Ok(s) => s,
        Err(_) => return false,
    };

    js_sys::Reflect::set(&style, &"position".into(), &"fixed".into()).ok();
    js_sys::Reflect::set(&style, &"left".into(), &"-9999px".into()).ok();

    if document.body().unwrap().append_child(&textarea).is_err() {
        return false;
    }

    js_sys::Reflect::get(textarea_js, &"select".into())
        .and_then(|f| js_sys::Function::from(f).call0(textarea_js))
        .ok();

    let success = js_sys::Reflect::get(&document, &"execCommand".into())
        .and_then(|f| {
            js_sys::Function::from(f)
                .call2(&document, &"copy".into(), &wasm_bindgen::JsValue::FALSE)
                .map(|_| true)
        })
        .unwrap_or(false);

    document.body().unwrap().remove_child(&textarea).ok();
    success
}

// Helper function to calculate personality tag (reused from PersonalityTagSection logic)
pub(crate) fn calculate_personality_tag(
    temporal: &crate::models::TemporalActivityResponse,
    engagement: &crate::models::EngagementResponse,
    content_style: &crate::models::ContentStyleResponse,
    follower_growth: &crate::models::FollowerGrowthResponse,
    casts_stats: &crate::models::CastsStatsResponse,
) -> (String, String) {
    // This matches the logic in PersonalityTagSection
    let total_casts = temporal.total_casts.max(casts_stats.total_casts);

    let mut tags: Vec<(String, String, f32)> = Vec::new();

    // 1. The Moon (18-moon.jpg) - Late night activity
    let late_night_activity: usize = temporal
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
    tags.push((
        "The Moon".to_string(),
        "/imgs/tarot/18-moon.jpg".to_string(),
        late_night_ratio * 100.0,
    ));

    // 2. The Star (17-star.jpg) - High emoji usage
    let total_emoji_count: usize = content_style.top_emojis.iter().map(|e| e.count).sum();
    let emoji_ratio = if total_casts > 0 {
        (total_emoji_count as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    let emoji_diversity_score = (content_style.top_emojis.len() as f32 / 10.0).min(1.0) * 30.0;
    tags.push((
        "The Star".to_string(),
        "/imgs/tarot/17-star.jpg".to_string(),
        emoji_ratio * 70.0 + emoji_diversity_score,
    ));

    // 3. The Chariot (07-charot.jpg) - High recast ratio
    let recast_ratio = if total_casts > 0 {
        (engagement.recasts_received as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    tags.push((
        "The Chariot".to_string(),
        "/imgs/tarot/07-charot.jpg".to_string(),
        recast_ratio * 100.0,
    ));

    // 4. The Lovers (06-lover.jpg) - High interaction rate
    let interaction_rate = if total_casts > 0 {
        ((engagement.reactions_received + engagement.replies_received) as f32 / total_casts as f32)
            .min(10.0)
    } else {
        0.0
    };
    tags.push((
        "The Lovers".to_string(),
        "/imgs/tarot/06-lover.jpg".to_string(),
        (interaction_rate / 10.0) * 100.0,
    ));

    // 5. The Sun (19-sun.jpg) - High follower growth
    let growth_rate = if follower_growth.followers_at_start > 0 {
        (follower_growth.net_growth as f32 / follower_growth.followers_at_start.max(1) as f32)
            .min(5.0)
    } else if follower_growth.net_growth > 0 {
        1.0
    } else {
        0.0
    };
    let absolute_growth_score = (follower_growth.net_growth as f32 / 200.0).min(1.0) * 50.0;
    tags.push((
        "The Sun".to_string(),
        "/imgs/tarot/19-sun.jpg".to_string(),
        (growth_rate / 5.0) * 50.0 + absolute_growth_score,
    ));

    // 6. The Hermit (09-hermit.jpg) - Low activity
    let activity_score = if total_casts < 50 {
        100.0 - (total_casts as f32 / 50.0) * 50.0
    } else {
        0.0
    };
    tags.push((
        "The Hermit".to_string(),
        "/imgs/tarot/09-hermit.jpg".to_string(),
        activity_score,
    ));

    // 7. The Magician (02-magician.jpg) - High content quality
    let avg_engagement = if total_casts > 0 {
        engagement.total_engagement as f32 / total_casts as f32
    } else {
        0.0
    };
    tags.push((
        "The Magician".to_string(),
        "/imgs/tarot/02-magician.jpg".to_string(),
        (avg_engagement / 20.0).min(1.0) * 100.0,
    ));

    // 8. The World (21-theworld.jpg) - High overall influence
    let influence_score = (follower_growth.current_followers as f32 / 1000.0).min(1.0) * 50.0
        + (engagement.total_engagement as f32 / 5000.0).min(1.0) * 50.0;
    tags.push((
        "The World".to_string(),
        "/imgs/tarot/21-theworld.jpg".to_string(),
        influence_score,
    ));

    // 9. Temperance (14-temperance.jpg) - Balanced activity
    let balance_score = {
        let cast_consistency = if temporal.monthly_distribution.len() > 0 {
            let avg_per_month = total_casts as f32 / temporal.monthly_distribution.len() as f32;
            let variance: f32 = temporal.monthly_distribution.iter()
                .map(|m| (m.count as f32 - avg_per_month).powi(2))
                .sum::<f32>() / temporal.monthly_distribution.len() as f32;
            100.0 - (variance / (avg_per_month + 1.0)).min(100.0)
        } else {
            0.0
        };
        cast_consistency
    };
    tags.push((
        "Temperance".to_string(),
        "/imgs/tarot/14-temperance.jpg".to_string(),
        balance_score,
    ));

    // 10. The Fool (01-fool.jpg) - New user
    let new_user_score = if total_casts < 20 && follower_growth.followers_at_start < 10 {
        100.0
    } else if total_casts < 50 {
        50.0
    } else {
        0.0
    };
    tags.push((
        "The Fool".to_string(),
        "/imgs/tarot/01-fool.jpg".to_string(),
        new_user_score,
    ));

    // 11. Justice (11-the justic.jpg) - High reply rate
    let reply_ratio = if total_casts > 0 {
        (engagement.replies_received as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    tags.push((
        "Justice".to_string(),
        "/imgs/tarot/11-the justic.jpg".to_string(),
        reply_ratio * 100.0,
    ));

    // 12. Wheel of Fortune (10-wheel.jpg) - Variable growth
    let growth_variance = if follower_growth.monthly_snapshots.len() > 1 {
        let changes: Vec<i32> = follower_growth.monthly_snapshots.windows(2)
            .map(|w| w[1].followers as i32 - w[0].followers as i32)
            .collect();
        let variance: f32 = if changes.len() > 0 {
            let avg: f32 = changes.iter().sum::<i32>() as f32 / changes.len() as f32;
            changes.iter().map(|c| ((*c as f32 - avg).powi(2))).sum::<f32>() / changes.len() as f32
        } else {
            0.0
        };
        (variance / 100.0).min(1.0) * 100.0
    } else {
        0.0
    };
    tags.push((
        "Wheel of Fortune".to_string(),
        "/imgs/tarot/10-wheel.jpg".to_string(),
        growth_variance,
    ));

    // Find the tag with highest score
    let matched = tags
        .iter()
        .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
        .unwrap_or_else(|| ("The Fool".to_string(), "/imgs/tarot/01-fool.jpg".to_string(), 0.0));

    (matched.0, matched.1) // Return (name, image_path)
}

// Helper function to convert relative image path to absolute URL
pub(crate) fn get_image_url(image_path: &str) -> String {
    if image_path.starts_with("http://") || image_path.starts_with("https://") {
        return image_path.to_string();
    }

    // Get current origin
    if let Some(window) = web_sys::window() {
        if let Ok(origin) = window.location().origin() {
            return format!("{}{}", origin, image_path);
        }
    }

    // Fallback to relative path
    image_path.to_string()
}

// Helper function to build share text
fn build_share_text(
    profile: &Option<ProfileWithRegistration>,
    report: &Option<AnnualReportResponse>,
) -> String {
    let mut text = String::from("🎉 Farcaster 2025 Annual Report\n\n");

    if let Some(p) = profile {
        if let Some(username) = &p.username {
            text.push_str(&format!("@{}'s 2025 Annual Report\n\n", username));
        }
    }

    if let Some(r) = report {
        text.push_str(&format!(
            "📊 Published {} Casts this year\n",
            r.engagement.total_engagement
        ));
        text.push_str(&format!(
            "❤️ Received {} likes\n",
            r.engagement.reactions_received
        ));
        text.push_str(&format!(
            "🔁 Received {} recasts\n",
            r.engagement.recasts_received
        ));

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
    text
}

#[function_component]
pub fn CallToActionSection(props: &CallToActionSectionProps) -> Html {
    let share_text = use_state(String::new);
    let is_sharing = use_state(|| false);
    let share_status = use_state(|| None::<String>); // Success/error message
    let is_farcaster_env = props.is_farcaster_env;
    let share_url = props.share_url.clone();
    let is_own_report = props.is_own_report;
    let current_user_fid = props.current_user_fid;

    // Share text for display and copying
    let share_text_content = build_share_text(&props.profile, &props.annual_report);

    // Calculate personality tag and get image URL
    let personality_tag_image_url = if let Some(report) = &props.annual_report {
        // Create a minimal CastsStatsResponse for the calculation
        // Since we don't have casts_stats in props, we'll use total_casts from temporal_activity
        let temp_casts_stats = crate::models::CastsStatsResponse {
            total_casts: report.temporal_activity.total_casts,
            date_distribution: Vec::new(),
            date_range: None,
            language_distribution: std::collections::HashMap::new(),
            top_nouns: Vec::new(),
            top_verbs: Vec::new(),
        };

        let (_tag_name, image_path) = calculate_personality_tag(
            &report.temporal_activity,
            &report.engagement,
            &report.content_style,
            &report.follower_growth,
            &temp_casts_stats,
        );
        Some(get_image_url(&image_path))
    } else {
        None
    };

    // Handler for Farcaster share (composeCast)
    let on_farcaster_share = {
        let is_sharing = is_sharing.clone();
        let share_status = share_status.clone();
        let text_for_share = share_text_content.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            is_sharing.set(true);
            share_status.set(None);

            let text_clone = text_for_share.clone();
            let share_status_clone = share_status.clone();
            let is_sharing_clone = is_sharing.clone();
            let embeds = image_url.as_ref().map(|url| vec![url.clone()]);

            spawn_local(async move {
                match farcaster::compose_cast(&text_clone, embeds).await {
                    Ok(_) => {
                        share_status_clone.set(Some("Share dialog opened!".to_string()));
                        web_sys::console::log_1(&"✅ Compose cast opened successfully".into());
                    }
                    Err(e) => {
                        share_status_clone.set(Some(format!("Failed to open share: {}", e)));
                        web_sys::console::error_1(
                            &format!("❌ Failed to compose cast: {}", e).into(),
                        );
                    }
                }
                is_sharing_clone.set(false);
            });
        })
    };

    // Handler for Twitter share
    let on_twitter_share = {
        let text = share_text_content.clone();
        let url = share_url.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            let mut share_text_for_twitter = if let Some(url_str) = &url {
                format!("{} {}", text, url_str)
            } else {
                text.clone()
            };

            // Add image URL to the share text if available
            if let Some(img_url) = &image_url {
                share_text_for_twitter.push_str(&format!(" {}", img_url));
            }

            // URL encode the text
            let encoded_text = js_sys::encode_uri_component(&share_text_for_twitter);
            let twitter_url = format!("https://twitter.com/intent/tweet?text={}", encoded_text);

            if let Some(window) = web_sys::window() {
                // Open in new window/tab
                if let Ok(Some(_)) = window.open_with_url_and_target(&twitter_url, "_blank") {
                    web_sys::console::log_1(&"✅ Twitter share opened".into());
                } else {
                    web_sys::console::error_1(&"⚠️ Failed to open Twitter share window".into());
                }
            }
        })
    };

    // Handler for copy to clipboard
    let on_copy = {
        let text = share_text_content.clone();
        let share_text = share_text.clone();
        let share_status = share_status.clone();
        let is_sharing = is_sharing.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            let mut text_with_image = text.clone();

            // Add image URL to the copied text if available
            if let Some(img_url) = &image_url {
                text_with_image.push_str(&format!("\n\nImage: {}", img_url));
            }

            share_text.set(text_with_image.clone());
            share_status.set(None);
            is_sharing.set(true);

            let text_clone = text_with_image.clone();
            let share_status_clone = share_status.clone();
            let is_sharing_clone = is_sharing.clone();

            spawn_local(async move {
                if copy_to_clipboard_async(&text_clone).await {
                    share_status_clone.set(Some("Copied to clipboard!".to_string()));
                    web_sys::console::log_1(&"✅ Text copied to clipboard".into());
                } else {
                    share_status_clone.set(Some("Failed to copy to clipboard".to_string()));
                    web_sys::console::warn_1(&"⚠️ Failed to copy to clipboard".into());
                }
                is_sharing_clone.set(false);
            });
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

            <div style="
                display: flex;
                flex-direction: column;
                gap: 16px;
                align-items: center;
                width: 100%;
                max-width: 400px;
            ">
                {if !is_own_report {
                    // Viewing someone else's report: show button to view own report
                    html! {
                <button
                            onclick={Callback::from({
                                let current_user_fid_clone = current_user_fid;
                                move |_| {
                                    if let Some(user_fid) = current_user_fid_clone {
                                        crate::services::update_annual_report_url(user_fid);
                                        // Force reload by reloading the page
                                        if let Some(window) = web_sys::window() {
                                            window.location().reload().ok();
                                        }
                                    }
                                }
                            })}
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
                                width: 100%;
                            "
                        >
                            {"View Your Annual Report"}
                        </button>
                    }
                } else if is_farcaster_env {
                    // Farcaster environment: show compose cast button
                    html! {
                        <button
                            onclick={on_farcaster_share.clone()}
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
                                width: 100%;
                "
                >
                    {if *is_sharing {
                                "Opening share..."
                    } else {
                                "Share on Farcaster"
                    }}
                </button>
                    }
                } else {
                    // Non-Farcaster environment: show Twitter and Copy buttons
                    html! {
                        <>
                            <button
                                onclick={on_twitter_share.clone()}
                                disabled={*is_sharing}
                                style="
                                    background: rgba(29, 161, 242, 0.8);
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
                                    width: 100%;
                                "
                            >
                                {"Share on Twitter"}
                            </button>
                            <button
                                onclick={on_copy.clone()}
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
                                    width: 100%;
                                "
                            >
                                {if *is_sharing {
                                    "Copying..."
                                } else {
                                    "Copy Link"
                                }}
                            </button>
                        </>
                    }
                }}
            </div>

            // Status message
            {if let Some(status) = (*share_status).as_ref() {
                html! {
                    <div style="
                        margin-top: 24px;
                        padding: 12px 24px;
                        background: rgba(255, 255, 255, 0.1);
                        backdrop-filter: blur(10px);
                        -webkit-backdrop-filter: blur(10px);
                        border-radius: 8px;
                        border: 1px solid rgba(255, 255, 255, 0.2);
                        font-size: 14px;
                        color: rgba(255, 255, 255, 0.9);
                        text-align: center;
                    ">
                        {status.clone()}
                    </div>
                }
            } else {
                html! {}
            }}

            // Show copied text in non-Farcaster environment
            {if !is_farcaster_env && !(*share_text).is_empty() {
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
                        ">{"Share text:"}</p>
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
    pub profile: Option<ProfileWithRegistration>,
    pub annual_report: Option<AnnualReportResponse>,
    pub is_farcaster_env: bool,
    pub share_url: Option<String>,
    pub is_own_report: bool,
    pub current_user_fid: Option<i64>,
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
    let share_text = use_state(String::new);
    let is_sharing = use_state(|| false);
    let share_status = use_state(|| None::<String>);
    let is_farcaster_env = props.is_farcaster_env;
    let share_url = props.share_url.clone();
    let is_own_report = props.is_own_report;
    let current_user_fid = props.current_user_fid;

    // Share text for display and copying
    let share_text_content = build_share_text(&props.profile, &props.annual_report);

    // Calculate personality tag and get image URL
    let personality_tag_image_url = if let Some(report) = &props.annual_report {
        let temp_casts_stats = crate::models::CastsStatsResponse {
            total_casts: report.temporal_activity.total_casts,
            date_distribution: Vec::new(),
            date_range: None,
            language_distribution: std::collections::HashMap::new(),
            top_nouns: Vec::new(),
            top_verbs: Vec::new(),
        };

        let (_tag_name, image_path) = calculate_personality_tag(
            &report.temporal_activity,
            &report.engagement,
            &report.content_style,
            &report.follower_growth,
            &temp_casts_stats,
        );
        Some(get_image_url(&image_path))
    } else {
        None
    };

    // Handler for Farcaster share (composeCast)
    let on_farcaster_share = {
        let is_sharing = is_sharing.clone();
        let share_status = share_status.clone();
        let text_for_share = share_text_content.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            is_sharing.set(true);
            share_status.set(None);

            let text_clone = text_for_share.clone();
            let share_status_clone = share_status.clone();
            let is_sharing_clone = is_sharing.clone();
            let embeds = image_url.as_ref().map(|url| vec![url.clone()]);

            spawn_local(async move {
                match farcaster::compose_cast(&text_clone, embeds).await {
                    Ok(_) => {
                        share_status_clone.set(Some("Share dialog opened!".to_string()));
                        web_sys::console::log_1(&"✅ Compose cast opened successfully".into());
                    }
                    Err(e) => {
                        share_status_clone.set(Some(format!("Failed to open share: {}", e)));
                        web_sys::console::error_1(
                            &format!("❌ Failed to compose cast: {}", e).into(),
                        );
                    }
                }
                is_sharing_clone.set(false);
            });
        })
    };

    // Handler for Twitter share
    let on_twitter_share = {
        let text = share_text_content.clone();
        let url = share_url.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            let mut share_text_for_twitter = if let Some(url_str) = &url {
                format!("{} {}", text, url_str)
            } else {
                text.clone()
            };

            if let Some(img_url) = &image_url {
                share_text_for_twitter.push_str(&format!(" {}", img_url));
            }

            let encoded_text = js_sys::encode_uri_component(&share_text_for_twitter);
            let twitter_url = format!("https://twitter.com/intent/tweet?text={}", encoded_text);

            if let Some(window) = web_sys::window() {
                if let Ok(Some(_)) = window.open_with_url_and_target(&twitter_url, "_blank") {
                    web_sys::console::log_1(&"✅ Twitter share opened".into());
                } else {
                    web_sys::console::error_1(&"⚠️ Failed to open Twitter share window".into());
                }
            }
        })
    };

    // Handler for copy to clipboard
    let on_copy = {
        let text = share_text_content.clone();
        let share_text = share_text.clone();
        let share_status = share_status.clone();
        let is_sharing = is_sharing.clone();
        let image_url = personality_tag_image_url.clone();

        Callback::from(move |_| {
            let mut text_with_image = text.clone();

            if let Some(img_url) = &image_url {
                text_with_image.push_str(&format!("\n\nImage: {}", img_url));
            }

            share_text.set(text_with_image.clone());
            share_status.set(None);
            is_sharing.set(true);

            let text_clone = text_with_image.clone();
            let share_status_clone = share_status.clone();
            let is_sharing_clone = is_sharing.clone();

            spawn_local(async move {
                if copy_to_clipboard_async(&text_clone).await {
                    share_status_clone.set(Some("Copied to clipboard!".to_string()));
                    web_sys::console::log_1(&"✅ Text copied to clipboard".into());
                } else {
                    share_status_clone.set(Some("Failed to copy to clipboard".to_string()));
                    web_sys::console::warn_1(&"⚠️ Failed to copy to clipboard".into());
                }
                is_sharing_clone.set(false);
            });
        })
    };

    let total_casts = props
        .temporal
        .total_casts
        .max(props.casts_stats.total_casts);

    // Calculate scores for each tarot card based on user behavior
    let mut tags = Vec::new();

    // 1. The Moon (18-moon.jpg) - Late night activity (0-6 AM)
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
        name: "The Moon".to_string(),
        description: "You share your thoughts in the quiet hours of the night".to_string(),
        image_path: "/imgs/tarot/18-moon.jpg".to_string(),
        score: late_night_ratio * 100.0,
    });

    // 2. The Star (17-star.jpg) - High emoji usage
    let total_emoji_count: usize = props.content_style.top_emojis.iter().map(|e| e.count).sum();
    let emoji_ratio = if total_casts > 0 {
        (total_emoji_count as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    let emoji_diversity_score =
        (props.content_style.top_emojis.len() as f32 / 10.0).min(1.0) * 30.0;
    tags.push(PersonalityTag {
        name: "The Star".to_string(),
        description: "Your expressive style shines through emojis".to_string(),
        image_path: "/imgs/tarot/17-star.jpg".to_string(),
        score: emoji_ratio * 70.0 + emoji_diversity_score,
    });

    // 3. The Chariot (07-charot.jpg) - High recast ratio (sharing content)
    let recast_ratio = if total_casts > 0 {
        (props.engagement.recasts_received as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "The Chariot".to_string(),
        description: "You drive conversations by sharing quality content".to_string(),
        image_path: "/imgs/tarot/07-charot.jpg".to_string(),
        score: recast_ratio * 100.0,
    });

    // 4. The Lovers (06-lover.jpg) - High interaction rate
    let interaction_rate = if total_casts > 0 {
        ((props.engagement.reactions_received + props.engagement.replies_received) as f32
            / total_casts as f32)
            .min(10.0)
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "The Lovers".to_string(),
        description: "You build deep connections through meaningful interactions".to_string(),
        image_path: "/imgs/tarot/06-lover.jpg".to_string(),
        score: (interaction_rate / 10.0) * 100.0,
    });

    // 5. The Sun (19-sun.jpg) - High follower growth
    let growth_rate = if props.follower_growth.followers_at_start > 0 {
        (props.follower_growth.net_growth as f32
            / props.follower_growth.followers_at_start.max(1) as f32)
            .min(5.0)
    } else if props.follower_growth.net_growth > 0 {
        1.0 // New user with growth
    } else {
        0.0
    };
    let absolute_growth_score = (props.follower_growth.net_growth as f32 / 200.0).min(1.0) * 50.0;
    tags.push(PersonalityTag {
        name: "The Sun".to_string(),
        description: "Your light attracts a growing community".to_string(),
        image_path: "/imgs/tarot/19-sun.jpg".to_string(),
        score: (growth_rate / 5.0) * 50.0 + absolute_growth_score,
    });

    // 6. The Hermit (09-hermit.jpg) - Low activity, selective sharing
    let activity_score = if total_casts < 50 {
        100.0 - (total_casts as f32 / 50.0) * 50.0
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "The Hermit".to_string(),
        description: "You share thoughtfully, choosing quality over quantity".to_string(),
        image_path: "/imgs/tarot/09-hermit.jpg".to_string(),
        score: activity_score,
    });

    // 7. The Magician (02-magician.jpg) - High content quality (high engagement per cast)
    let avg_engagement = if total_casts > 0 {
        props.engagement.total_engagement as f32 / total_casts as f32
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "The Magician".to_string(),
        description: "You create content that captivates and inspires".to_string(),
        image_path: "/imgs/tarot/02-magician.jpg".to_string(),
        score: (avg_engagement / 20.0).min(1.0) * 100.0,
    });

    // 8. The World (21-theworld.jpg) - High overall influence
    let influence_score = (props.follower_growth.current_followers as f32 / 1000.0).min(1.0) * 50.0
        + (props.engagement.total_engagement as f32 / 5000.0).min(1.0) * 50.0;
    tags.push(PersonalityTag {
        name: "The World".to_string(),
        description: "You have built a significant presence in the community".to_string(),
        image_path: "/imgs/tarot/21-theworld.jpg".to_string(),
        score: influence_score,
    });

    // 9. Temperance (14-temperance.jpg) - Balanced activity
    let balance_score = {
        let cast_consistency = if props.temporal.monthly_distribution.len() > 0 {
            let avg_per_month = total_casts as f32 / props.temporal.monthly_distribution.len() as f32;
            let variance: f32 = props.temporal.monthly_distribution.iter()
                .map(|m| (m.count as f32 - avg_per_month).powi(2))
                .sum::<f32>() / props.temporal.monthly_distribution.len() as f32;
            100.0 - (variance / (avg_per_month + 1.0)).min(100.0)
        } else {
            0.0
        };
        cast_consistency
    };
    tags.push(PersonalityTag {
        name: "Temperance".to_string(),
        description: "You maintain a balanced and consistent presence".to_string(),
        image_path: "/imgs/tarot/14-temperance.jpg".to_string(),
        score: balance_score,
    });

    // 10. The Fool (01-fool.jpg) - New user or low activity
    let new_user_score = if total_casts < 20 && props.follower_growth.followers_at_start < 10 {
        100.0
    } else if total_casts < 50 {
        50.0
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "The Fool".to_string(),
        description: "You're beginning an exciting journey on Farcaster".to_string(),
        image_path: "/imgs/tarot/01-fool.jpg".to_string(),
        score: new_user_score,
    });

    // 11. Justice (11-the justic.jpg) - High reply rate (fair engagement)
    let reply_ratio = if total_casts > 0 {
        (props.engagement.replies_received as f32 / total_casts as f32).min(1.0)
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "Justice".to_string(),
        description: "You engage in meaningful dialogue and discussions".to_string(),
        image_path: "/imgs/tarot/11-the justic.jpg".to_string(),
        score: reply_ratio * 100.0,
    });

    // 12. Wheel of Fortune (10-wheel.jpg) - Variable growth pattern
    let growth_variance = if props.follower_growth.monthly_snapshots.len() > 1 {
        let changes: Vec<i32> = props.follower_growth.monthly_snapshots.windows(2)
            .map(|w| w[1].followers as i32 - w[0].followers as i32)
            .collect();
        let variance: f32 = if changes.len() > 0 {
            let avg: f32 = changes.iter().sum::<i32>() as f32 / changes.len() as f32;
            changes.iter().map(|c| ((*c as f32 - avg).powi(2))).sum::<f32>() / changes.len() as f32
        } else {
            0.0
        };
        (variance / 100.0).min(1.0) * 100.0
    } else {
        0.0
    };
    tags.push(PersonalityTag {
        name: "Wheel of Fortune".to_string(),
        description: "Your journey has seen ups and downs, but you keep moving forward".to_string(),
        image_path: "/imgs/tarot/10-wheel.jpg".to_string(),
        score: growth_variance,
    });

    // Find the tag with highest score
    let matched_tag = tags
        .iter()
        .max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or_else(|| PersonalityTag {
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
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            timeout_closure.as_ref().unchecked_ref::<js_sys::Function>(),
            200,
        );
        timeout_closure.forget();

        || ()
    });

    let is_flipped = use_state(|| false);

    // Handler for card flip
    let on_card_click = {
        let is_flipped = is_flipped.clone();
        Callback::from(move |_| {
            if !*is_flipped {
                is_flipped.set(true);
            }
        })
    };

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
                    onclick={on_card_click.clone()}
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
                        style={format!("
                            position: relative;
                            width: 100%;
                            height: 100%;
                            transform-style: preserve-3d;
                            transition: transform 0.8s ease-in-out;
                            transform: rotateY({}deg);
                        ", if *is_flipped { 0 } else { 180 })}
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
                                flex-direction: column;
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
                                        margin-bottom: 24px;
                                    "
                                />
                                <p style="
                                    font-size: 14px;
                                    font-weight: 400;
                                    color: rgba(255, 255, 255, 0.9);
                                    text-align: center;
                                    margin: 0;
                                    line-height: 1.5;
                                    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
                                ">{"Click to flip"}</p>
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

                // Share buttons
                <div style="
                    display: flex;
                    flex-direction: column;
                    gap: 16px;
                    align-items: center;
                    width: 100%;
                    max-width: 400px;
                    margin: 0 auto;
                ">
                    {if !is_own_report {
                        html! {
                            <button
                                onclick={Callback::from({
                                    let current_user_fid_clone = current_user_fid;
                                    move |_| {
                                        if let Some(user_fid) = current_user_fid_clone {
                                            crate::services::update_annual_report_url(user_fid);
                                            if let Some(window) = web_sys::window() {
                                                window.location().reload().ok();
                                            }
                                        }
                                    }
                                })}
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
                                    width: 100%;
                                "
                            >
                                {"View Your Annual Report"}
                            </button>
                        }
                    } else if is_farcaster_env {
                        html! {
                            <button
                                onclick={on_farcaster_share.clone()}
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
                                    width: 100%;
                                "
                            >
                                {if *is_sharing {
                                    "Opening share..."
                                } else {
                                    "Share on Farcaster"
                                }}
                            </button>
                        }
                    } else {
                        html! {
                            <>
                                <button
                                    onclick={on_twitter_share.clone()}
                                    style="
                                        background: rgba(29, 161, 242, 0.8);
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
                                        width: 100%;
                                    "
                                >
                                    {"Share on Twitter"}
                                </button>
                                <button
                                    onclick={on_copy.clone()}
                                    disabled={*is_sharing}
                                    style="
                                        background: rgba(255, 255, 255, 0.1);
                                        color: white;
                                        border: 1px solid rgba(255, 255, 255, 0.2);
                                        border-radius: 12px;
                                        padding: 16px 32px;
                                        font-size: 18px;
                                        font-weight: 600;
                                        cursor: pointer;
                                        transition: all 0.3s ease;
                                        backdrop-filter: blur(10px);
                                        -webkit-backdrop-filter: blur(10px);
                                        width: 100%;
                                    "
                                >
                                    {if *is_sharing {
                                        "Copying..."
                                    } else {
                                        "Copy Share Text"
                                    }}
                                </button>
                            </>
                        }
                    }}
                    {if let Some(status) = (*share_status).as_ref() {
                        html! {
                    <p style="
                        font-size: 14px;
                                color: rgba(255, 255, 255, 0.8);
                                margin: 8px 0 0 0;
                                text-align: center;
                            ">{status.clone()}</p>
                        }
                    } else {
                        html! {}
                    }}
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
