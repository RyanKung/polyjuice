use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use yew::prelude::*;

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
                {if let Some(pfp_url) = &props.profile.pfp_url {
                    if !pfp_url.is_empty() {
                        html! {
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
                        }
                    } else {
                        html! {
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
                    }
                } else {
                    html! {
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
                }}
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

// Helper function to get far zodiac sign based on FID
fn get_far_zodiac_sign(fid: i64) -> &'static str {
    let zodiacs = [
        "Capricorn",
        "Aquarius",
        "Pisces",
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
    ];
    let index = (fid % 12) as usize;
    zodiacs[index]
}

#[function_component]
pub fn IdentitySection(props: &IdentitySectionProps) -> Html {
    // Get registration date and calculate zodiac signs
    // Note: registered_at is already a Unix timestamp, not Farcaster timestamp
    let (birthday_date, zodiac_image_url, zodiac_info) = props
        .profile
        .registered_at
        .map(|timestamp| {
            // registered_at is already Unix timestamp, use directly
            let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
                timestamp as f64 * 1000.0,
            ));
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
        })
        .unwrap_or_else(|| {
            (
                "N/A".to_string(),
                "/imgs/zodiac/capricorn.png".to_string(),
                "N/A".to_string(),
            )
        });

    // Get first cast date
    // Note: first_cast.timestamp is already a Unix timestamp, not Farcaster timestamp
    let first_cast_date = props
        .temporal
        .first_cast
        .as_ref()
        .map(|cast| {
            // first_cast.timestamp is already Unix timestamp, use directly
            let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
                cast.timestamp as f64 * 1000.0,
            ));
            let month = date.get_month() as u32 + 1;
            let day = date.get_date() as u32;
            let year = date.get_full_year();
            format!("{}/{:02}/{:02}", year, month, day)
        })
        .unwrap_or_else(|| "N/A".to_string());

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
                                            if !pfp_url.is_empty() {
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
                        {"This year, your first cast was on "}
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
    let total_casts = props
        .temporal
        .total_casts_in_year
        .unwrap_or(props.temporal.total_casts);
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
                                            if !pfp_url.is_empty() {
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

// Style Section Component
#[derive(Properties, PartialEq, Clone)]
pub struct StyleSectionProps {
    pub style: ContentStyleResponse,
    pub casts_stats: CastsStatsResponse,
    pub profile: ProfileWithRegistration,
}

#[function_component]
pub fn StyleSection(props: &StyleSectionProps) -> Html {
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
                                    if !pfp_url.is_empty() {
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

// Tarot card mapping: index 0-21 corresponds to 22 tarot cards
// Format: (name, filename, description)
const TAROT_CARDS: &[(&str, &str, &str)] = &[
    (
        "The Fool",
        "01-fool.jpg",
        "You're beginning an exciting journey on Farcaster",
    ),
    (
        "The Magician",
        "02-magician.jpg",
        "You create content that captivates and inspires",
    ),
    (
        "The High Priestess",
        "02-thehighpriestess.jpg",
        "You share your wisdom and insights with the community",
    ),
    (
        "The Empress",
        "03-theempress.jpg",
        "You nurture and grow meaningful connections",
    ),
    (
        "The Emperor",
        "04-theempercr.jpg",
        "You lead with authority and structure",
    ),
    (
        "The Hierophant",
        "05-herophant.jpg",
        "You share knowledge and guide others",
    ),
    (
        "The Lovers",
        "06-lover.jpg",
        "You build deep connections through meaningful interactions",
    ),
    (
        "The Chariot",
        "07-charot.jpg",
        "You drive conversations by sharing quality content",
    ),
    (
        "Strength",
        "08-strength.jpg",
        "You show resilience and inner strength in your journey",
    ),
    (
        "The Hermit",
        "09-hermit.jpg",
        "You share thoughtfully, choosing quality over quantity",
    ),
    (
        "Wheel of Fortune",
        "10-wheel.jpg",
        "Your journey has seen ups and downs, but you keep moving forward",
    ),
    (
        "Justice",
        "11-the justic.jpg",
        "You engage in meaningful dialogue and discussions",
    ),
    (
        "The Hanged Man",
        "12-thehangedman.jpg",
        "You see things from a different perspective",
    ),
    (
        "Death",
        "13-death.jpg",
        "You embrace transformation and new beginnings",
    ),
    (
        "Temperance",
        "14-temperance.jpg",
        "You maintain a balanced and consistent presence",
    ),
    (
        "The Devil",
        "15-devil.jpg",
        "You challenge conventions and break free from limitations",
    ),
    (
        "The Tower",
        "16-tower.jpg",
        "You bring about sudden change and revelation",
    ),
    (
        "The Star",
        "17-star.jpg",
        "Your expressive style shines through emojis",
    ),
    (
        "The Moon",
        "18-moon.jpg",
        "You share your thoughts in the quiet hours of the night",
    ),
    (
        "The Sun",
        "19-sun.jpg",
        "Your light attracts a growing community",
    ),
    (
        "Judgement",
        "20-judgement.jpg",
        "You reflect on your journey and make important decisions",
    ),
    (
        "The World",
        "21-world.jpg",
        "You have built a significant presence in the community",
    ),
];

// Helper function to calculate personality tag based on FID hash mod 22
pub(crate) fn calculate_personality_tag(
    _temporal: &crate::models::TemporalActivityResponse,
    _engagement: &crate::models::EngagementResponse,
    _content_style: &crate::models::ContentStyleResponse,
    _follower_growth: &crate::models::FollowerGrowthResponse,
    _casts_stats: &crate::models::CastsStatsResponse,
    fid: i64,
) -> (String, String, String) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    // Calculate hash of FID
    let mut hasher = DefaultHasher::new();
    fid.hash(&mut hasher);
    let hash = hasher.finish();

    // Get index by mod 22 (0-21)
    let index = (hash % 22) as usize;

    // Get tarot card name, image path, and description
    let (name, filename, description) = TAROT_CARDS[index];
    let image_path = format!("/imgs/tarot/{}", filename);

    (name.to_string(), image_path, description.to_string())
}

// Helper function to get zodiac index (0-11) from zodiac name
fn get_zodiac_index(zodiac_name: &str) -> u8 {
    match zodiac_name {
        "Capricorn" => 0,
        "Aquarius" => 1,
        "Pisces" => 2,
        "Aries" => 3,
        "Taurus" => 4,
        "Gemini" => 5,
        "Cancer" => 6,
        "Leo" => 7,
        "Virgo" => 8,
        "Libra" => 9,
        "Scorpio" => 10,
        "Sagittarius" => 11,
        _ => 0, // Default to Capricorn
    }
}

// Helper function to encode user stats as compact binary format for sharing
// Format: [0-7]: FID (i64, little-endian), [8]: Zodiac (u8, 0-11), [9]: Social type (u8, 0=silent, 1=social),
//         [10-13]: Total casts (u32), [14-17]: Total reactions (u32), [18-21]: Total followers (u32)
// Total: 22 bytes -> ~30 chars in base64url
pub(crate) fn encode_image_params_for_share(
    fid: i64,
    _username: Option<&str>,
    _avatar_url: Option<&str>,
    zodiac_url: &str,
    social_type_url: &str,
    total_casts: usize,
    total_reactions: usize,
    total_followers: usize,
) -> String {
    use base64::engine::general_purpose::STANDARD_NO_PAD;
    use base64::Engine;
    
    // Extract zodiac name from URL (e.g., "/imgs/zodiac/capricorn.png" -> "capricorn")
    let zodiac_name = zodiac_url
        .split('/')
        .last()
        .and_then(|s| s.strip_suffix(".png"))
        .unwrap_or("capricorn");
    
    // Convert zodiac name to capitalized format for index lookup
    let zodiac_capitalized = if !zodiac_name.is_empty() {
        let mut chars = zodiac_name.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    } else {
        "Capricorn".to_string()
    };
    
    let zodiac_index = get_zodiac_index(&zodiac_capitalized);
    
    // Extract social type from URL (0 = silent, 1 = social)
    let social_type_index = if social_type_url.contains("social.png") {
        1u8
    } else {
        0u8 // silent
    };
    
    // Pack into binary format
    let mut bytes = Vec::with_capacity(22);
    
    // FID as i64 (8 bytes, little-endian)
    bytes.extend_from_slice(&fid.to_le_bytes());
    
    // Zodiac index (1 byte)
    bytes.push(zodiac_index);
    
    // Social type index (1 byte)
    bytes.push(social_type_index);
    
    // Total casts as u32 (4 bytes, little-endian)
    bytes.extend_from_slice(&(total_casts as u32).to_le_bytes());
    
    // Total reactions as u32 (4 bytes, little-endian)
    bytes.extend_from_slice(&(total_reactions as u32).to_le_bytes());
    
    // Total followers as u32 (4 bytes, little-endian)
    bytes.extend_from_slice(&(total_followers as u32).to_le_bytes());
    
    // Encode to base64url (URL-safe, no padding)
    let base64url = STANDARD_NO_PAD.encode(&bytes)
        .replace('+', "-")
        .replace('/', "_");
    
    base64url
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

// Fetch image data from URL and return as Vec<u8>

// Helper function to build share text
fn build_share_text(
    _profile: &Option<ProfileWithRegistration>,
    report: &Option<AnnualReportResponse>,
    tarot_card_name: Option<&str>,
    share_url: Option<&str>,
) -> String {
    let mut text = String::from("My Annual Report: This year I ");

    if let Some(r) = report {
        // Use total_casts_in_year if available, otherwise fallback to total_casts
        // This matches what's displayed in the report
        let total_casts = r.temporal_activity.total_casts_in_year
            .unwrap_or(r.temporal_activity.total_casts);
        text.push_str(&format!(
            "Published {} Casts this year, ",
            total_casts
        ));
        text.push_str(&format!(
            "Received {} likes, ",
            r.engagement.reactions_received
        ));
        text.push_str(&format!(
            "Received {} recasts, ",
            r.engagement.recasts_received
        ));

        if let Some(most_active) = &r.temporal_activity.most_active_month {
            text.push_str(&format!("Most active month: {}, ", most_active));
        }

        if !r.content_style.top_emojis.is_empty() {
            let top_emoji = &r.content_style.top_emojis[0];
            text.push_str(&format!("Most used emoji: {}", top_emoji.emoji));
        }
    }

    text.push_str("\n\n");

    if let Some(tarot_name) = tarot_card_name {
        text.push_str(&format!("My Annual Tarot Card is {}\n\n", tarot_name));
    }

    if let Some(url) = share_url {
        text.push_str(&format!("url: {}\n\n", url));
    }

    text.push_str("#MyFarcaster2025 #polyjuice");
    text
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
    let base_share_url = props.share_url.clone();
    let is_own_report = props.is_own_report;
    let current_user_fid = props.current_user_fid;
    
    // State for share URL with encoded params
    let share_url_with_params = use_state(|| base_share_url.clone());

    // Calculate personality tag and get image URL
    let (tarot_card_name, personality_tag_image_url) = if let Some(report) = &props.annual_report {
        let temp_casts_stats = crate::models::CastsStatsResponse {
            total_casts: report.temporal_activity.total_casts,
            date_distribution: Vec::new(),
            date_range: None,
            language_distribution: std::collections::HashMap::new(),
            top_nouns: Vec::new(),
            top_verbs: Vec::new(),
        };

        // Get FID from profile or annual report
        let fid = props
            .profile
            .as_ref()
            .map(|p| p.fid)
            .unwrap_or_else(|| report.fid);

        let (tag_name, image_path, _description) = calculate_personality_tag(
            &report.temporal_activity,
            &report.engagement,
            &report.content_style,
            &report.follower_growth,
            &temp_casts_stats,
            fid,
        );
        (Some(tag_name), Some(get_image_url(&image_path)))
    } else {
        (None, None)
    };
    
    // Encode image URLs and stats as base64 params and update share URL
    {
        let profile = props.profile.clone();
        let temporal = props.temporal.clone();
        let engagement = props.engagement.clone();
        let follower_growth = props.follower_growth.clone();
        let share_url_with_params_for_effect = share_url_with_params.clone();
        let base_share_url_for_effect = base_share_url.clone();
        
        use_effect_with(
            (profile.clone(), temporal.clone(), engagement.clone(), follower_growth.clone()),
            move |_| {
                // Get zodiac image URL
                // Note: registered_at is already a Unix timestamp, not Farcaster timestamp
                let zodiac_url = profile.as_ref()
                    .and_then(|p| p.registered_at)
                    .map(|timestamp| {
                        // registered_at is already Unix timestamp, use directly
                        let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(
                            timestamp as f64 * 1000.0,
                        ));
                        let month = date.get_month() as u32 + 1;
                        let day = date.get_date() as u32;
                        let zodiac = get_zodiac_sign(month, day);
                        let zodiac_lower = zodiac.to_lowercase();
                        get_image_url(&format!("/imgs/zodiac/{}.png", zodiac_lower))
                    })
                    .unwrap_or_else(|| get_image_url("/imgs/zodiac/capricorn.png"));
                
                // Get user info and stats - use the same fields as displayed in the report
                let fid = profile.as_ref().map(|p| p.fid).unwrap_or(0);
                
                // Use total_casts_in_year if available, otherwise fallback to total_casts
                // This matches what's displayed in FollowerGrowthSection
                let total_casts = temporal.total_casts_in_year.unwrap_or(temporal.total_casts);
                
                // Use reactions_received for total reactions
                let total_reactions = engagement.reactions_received;
                
                // Use current_followers for total followers
                let total_followers = follower_growth.current_followers;
                
                // Get social type image URL based on total casts (same logic as FollowerGrowthSection)
                let social_type_url = if total_casts >= 200 {
                    get_image_url("/imgs/social_type/social.png")
                } else {
                    get_image_url("/imgs/social_type/slient.png")
                };
                
                // Encode params (only fid, zodiac index, social_type index, and stats)
                // Username and avatar will be fetched by worker from API
                let params_base64 = encode_image_params_for_share(
                    fid,
                    None, // Username will be fetched by worker
                    None, // Avatar will be fetched by worker
                    &zodiac_url,
                    &social_type_url,
                    total_casts,
                    total_reactions,
                    total_followers,
                );
                
                // Append params to share URL
                if let Some(base_url) = base_share_url_for_effect {
                    let url_with_params = format!("{}?params={}", base_url, params_base64);
                    share_url_with_params_for_effect.set(Some(url_with_params));
                }
            },
        );
    }

    // Share text for display and copying (use URL with params)
    let share_url_for_text = share_url_with_params.as_ref().as_ref().as_ref().map(|s| s.as_str());
    let share_text_content = build_share_text(
        &props.profile,
        &props.annual_report,
        tarot_card_name.as_deref(),
        share_url_for_text,
    );

    // Handler for Farcaster share (composeCast)
    let on_farcaster_share = {
        let is_sharing = is_sharing.clone();
        let share_status = share_status.clone();
        let text_for_share = share_text_content.clone();
        let image_url = personality_tag_image_url.clone();
        let url_for_share = share_url_with_params.clone();

        Callback::from(move |_| {
            is_sharing.set(true);
            share_status.set(None);

            let text_clone = text_for_share.clone();
            let share_status_clone = share_status.clone();
            let is_sharing_clone = is_sharing.clone();
            
            // Build embeds: include both image URL and share URL
            let mut embeds = Vec::new();
            if let Some(img_url) = &image_url {
                embeds.push(img_url.clone());
            }
            if let Some(url_str) = url_for_share.as_ref() {
                embeds.push(url_str.clone());
            }
            let embeds_option = if embeds.is_empty() { None } else { Some(embeds) };

            spawn_local(async move {
                match farcaster::compose_cast(&text_clone, embeds_option).await {
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
        // Text already includes URL, so we can use it directly
        Callback::from(move |_| {
            let encoded_text = js_sys::encode_uri_component(&text);
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

    // Calculate tarot card based on FID hash mod 22
    let fid = props
        .profile
        .as_ref()
        .map(|p| p.fid)
        .unwrap_or_else(|| props.annual_report.as_ref().map(|r| r.fid).unwrap_or(0));

    let (name, image_path, description) = calculate_personality_tag(
        &props.temporal,
        &props.engagement,
        &props.content_style,
        &props.follower_growth,
        &props.casts_stats,
        fid,
    );

    let matched_tag = PersonalityTag {
        name,
        description,
        image_path,
        score: 0.0, // Not used anymore
    };

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
            height: calc(100% - 60px);
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: flex-start;
            padding: 100px 40px 40px 40px;
            box-sizing: border-box;
            overflow-y: auto;
        ">
            <div style="
                text-align: center;
                width: 100%;
                max-width: 800px;
            ">
                // Text above the card - changes based on flip state
                <div style="
                    margin-bottom: 24px;
                    min-height: 60px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                ">
                    {if *is_flipped {
                        html! {
                            <p style="
                                font-size: 18px;
                                font-weight: 500;
                                color: rgba(255, 255, 255, 0.95);
                                margin: 0;
                                line-height: 1.6;
                                max-width: 600px;
                                text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
                            ">
                                {format!("{}: {}", matched_tag.name.clone(), matched_tag.description.clone())}
                            </p>
                        }
                    } else {
                        html! {
                            <p style="
                                font-size: 24px;
                                font-weight: 600;
                                color: white;
                                margin: 0;
                                text-shadow: 0 2px 10px rgba(0, 0, 0, 0.3);
                            ">
                                {"Claim Your 2025 Tarot"}
                            </p>
                        }
                    }}
                </div>
                
                <div
                    class="tarot-card"
                    onclick={on_card_click.clone()}
                    style="
                        width: 320px;
                        height: 448px;
                        margin: 0 auto 16px;
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
                                padding: 0;
                                box-sizing: border-box;
                            "
                        >
                            {{
                                // Use original tarot card URL directly
                                let image_src = personality_tag_image_url.clone()
                                    .unwrap_or_else(|| "".to_string());
                                
                                html! {
                                    <img
                                        src={image_src.clone()}
                                        alt={matched_tag.name.clone()}
                                        style="
                                            width: 100%;
                                            height: 100%;
                                            object-fit: contain;
                                            border-radius: 0;
                                            box-shadow: none;
                                            border: none;
                                            padding: 0;
                                            margin: 0;
                                            display: block;
                                        "
                                    />
                                }
                            }}
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
                                border-radius: 0;
                                box-shadow: none;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                padding: 0;
                                box-sizing: border-box;
                            "
                        >
                            <div style="
                                position: relative;
                                width: 100%;
                                height: 100%;
                                border-radius: 0;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 50%, #f093fb 100%);
                                display: flex;
                                flex-direction: column;
                                align-items: center;
                                justify-content: center;
                                padding: 30px;
                                box-sizing: border-box;
                            ">
                                <img
                                    src="/imgs/polyjuice.png"
                                    alt="Polyjuice"
                                    class="embossed-logo"
                                    style="
                                        width: 100%;
                                        height: auto;
                                        max-width: 250px;
                                        object-fit: contain;
                                        filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.6)) drop-shadow(-1px -1px 2px rgba(255, 255, 255, 0.4)) brightness(1.1) contrast(1.2);
                                        opacity: 0.95;
                                        margin-bottom: 16px;
                                    "
                                />
                                <p style="
                                    font-size: 13px;
                                    font-weight: 400;
                                    color: rgba(255, 255, 255, 0.9);
                                    text-align: center;
                                    margin: 0;
                                    line-height: 1.4;
                                    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
                                ">{"Click to flip"}</p>
                            </div>
                        </div>
                    </div>
                </div>

                // Share buttons
                <div style="
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                    align-items: center;
                    width: 100%;
                    max-width: 300px;
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
                                    border-radius: 10px;
                                    padding: 12px 24px;
                                    font-size: 16px;
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
                                    border-radius: 10px;
                                    padding: 12px 24px;
                                    font-size: 16px;
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
                                        border-radius: 10px;
                                        padding: 12px 24px;
                                        font-size: 16px;
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
                                        border-radius: 10px;
                                        padding: 12px 24px;
                                        font-size: 16px;
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

