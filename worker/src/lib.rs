use serde_json::json;
use worker::*;

// Tarot card mapping: index 0-21 corresponds to 22 tarot cards
// This matches the TAROT_CARDS constant in src/pages/annual_report/sections.rs
// Updated to match actual image files in imgs/tarot/
const TAROT_CARDS: &[(&str, &str)] = &[
    ("The Fool", "01-fool.jpg"),
    ("The Magician", "02-magician.jpg"),
    ("The High Priestess", "02-thehighpriestess.jpg"),
    ("The Empress", "03-theempress.jpg"),
    ("The Emperor", "04-theempercr.jpg"),
    ("The Hierophant", "05-herophant.jpg"),
    ("The Lovers", "06-lover.jpg"),
    ("The Chariot", "07-charot.jpg"),
    ("Strength", "08-strength.jpg"),
    ("The Hermit", "09-hermit.jpg"),
    ("Wheel of Fortune", "10-wheel.jpg"),
    ("Justice", "11-the justic.jpg"),
    ("The Hanged Man", "12-thehangedman.jpg"),
    ("Death", "13-death.jpg"),
    ("Temperance", "14-temperance.jpg"),
    ("The Devil", "15-devil.jpg"),
    ("The Tower", "16-tower.jpg"),
    ("The Star", "17-star.jpg"),
    ("The Moon", "18-moon.jpg"),
    ("The Sun", "19-sun.jpg"),
    ("Judgement", "20-judgement.jpg"),
    ("The World", "21-world.jpg"),
];

/// Calculate tarot card based on FID hash mod 22
/// This matches the logic in src/pages/annual_report/sections.rs::calculate_personality_tag
fn calculate_tarot_card(fid: i64) -> (&'static str, &'static str) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Calculate hash of FID
    let mut hasher = DefaultHasher::new();
    fid.hash(&mut hasher);
    let hash = hasher.finish();

    // Get index by mod 22 (0-21)
    let index = (hash % 22) as usize;

    // Get tarot card name and filename
    TAROT_CARDS[index]
}

/// Generate meta tags for annual report based on FID
fn generate_annual_report_meta_tags(fid: i64, base_url: &str, pathname: &str) -> String {
    let (_tarot_name, tarot_filename) = calculate_tarot_card(fid);
    let tarot_image_url = format!("{}/imgs/tarot/{}", base_url, tarot_filename);
    let target_url = format!("{}{}", base_url, pathname);

    // Create embed JSON matching the format from embed.rs
    let embed_json = json!({
        "version": "1",
        "imageUrl": tarot_image_url,
        "button": {
            "title": "View Annual Report",
            "action": {
                "type": "launch_miniapp",
                "url": target_url,
                "name": "polyjuice",
                "splashImageUrl": format!("{}/imgs/splash.png", base_url),
                "splashBackgroundColor": "#667eea"
            }
        }
    });

    let embed_json_str = serde_json::to_string(&embed_json).unwrap_or_default();

    // Generate frame JSON (for backward compatibility)
    let frame_json = json!({
        "version": "1",
        "imageUrl": tarot_image_url,
        "button": {
            "title": "View Annual Report",
            "action": {
                "type": "launch_frame",
                "url": target_url,
                "name": "polyjuice",
                "splashImageUrl": format!("{}/imgs/splash.png", base_url),
                "splashBackgroundColor": "#667eea"
            }
        }
    });

    let frame_json_str = serde_json::to_string(&frame_json).unwrap_or_default();

    // Generate Open Graph meta tags as well
    format!(
        r#"<meta name="fc:miniapp" content='{}' />
<meta name="fc:frame" content='{}' />
<meta property="og:title" content="2025 Annual Report - Polyjuice" />
<meta property="og:description" content="View my Farcaster 2025 Annual Report" />
<meta property="og:image" content="{}" />
<meta property="og:url" content="{}" />
<meta property="og:type" content="website" />
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content="2025 Annual Report - Polyjuice" />
<meta name="twitter:description" content="View my Farcaster 2025 Annual Report" />
<meta name="twitter:image" content="{}" />"#,
        embed_json_str, frame_json_str, tarot_image_url, target_url, tarot_image_url
    )
}

/// Check if the request is from a Farcaster crawler/bot
fn is_farcaster_bot(user_agent: Option<&str>, headers: &Headers) -> bool {
    // Check User-Agent
    if let Some(ua) = user_agent {
        let ua_lower = ua.to_lowercase();
        if ua_lower.contains("farcaster")
            || ua_lower.contains("bot")
            || ua_lower.contains("crawler")
            || ua_lower.contains("spider")
        {
            return true;
        }
    }

    // Check for custom headers that Farcaster might send
    if headers.get("x-farcaster-bot").is_ok() {
        return true;
    }

    false
}

/// Extract FID from annual report URL path
/// Format: /annual-report/{fid}
fn extract_fid_from_path(pathname: &str) -> Option<i64> {
    if pathname.starts_with("/annual-report/") {
        let fid_str = pathname.strip_prefix("/annual-report/")?;
        // Remove trailing slash if present
        let fid_str = fid_str.trim_end_matches('/');
        fid_str.parse().ok()
    } else {
        None
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let pathname = url.path();
    let user_agent = req.headers().get("user-agent").ok().flatten();

    // Get base URL from environment or default
    let base_url = env
        .var("BASE_URL")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "https://miniapp.polyjuice.io".to_string());

    // Check if this is a Farcaster bot request
    let is_bot = is_farcaster_bot(user_agent.as_deref(), req.headers());

    // Only process annual report routes for bots
    if is_bot && pathname.starts_with("/annual-report/") {
        // Extract FID from path
        let fid = match extract_fid_from_path(&pathname) {
            Some(fid) => fid,
            None => {
                console_log!("Failed to extract FID from path: {}", pathname);
                return Response::error("Invalid FID in URL path", 400);
            }
        };

        // Get source URL from environment or use default GitHub Pages format
        let source_url = match env.var("SOURCE_URL") {
            Ok(url) => {
                // If SOURCE_URL is set, use it directly
                url.to_string()
            }
            Err(_) => {
                // Fallback to GitHub Pages format
                let github_username = env
                    .var("GITHUB_USERNAME")
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "your-username".to_string());
                format!("https://{}.github.io/index.html", github_username)
            }
        };

        // Try to fetch from the original source
        let source_url_parsed = match source_url.parse() {
            Ok(url) => url,
            Err(e) => {
                console_log!("Failed to parse source URL: {:?}", e);
                return Response::error(
                    format!("Invalid source URL configuration: {}", e),
                    500,
                );
            }
        };

        let fetch_result = Fetch::Url(source_url_parsed).send().await;

        match fetch_result {
            Ok(mut response) => {
                // Check response status
                if response.status_code() >= 400 {
                    console_log!(
                        "Source returned error status: {}",
                        response.status_code()
                    );
                    return Response::error(
                        format!("Failed to fetch source: HTTP {}", response.status_code()),
                        502,
                    );
                }

                // Only process HTML responses
                let content_type = response
                    .headers()
                    .get("content-type")
                    .ok()
                    .flatten()
                    .unwrap_or_default();

                if !content_type.contains("text/html") {
                    console_log!("Source is not HTML, content-type: {}", content_type);
                    return Response::error("Source is not HTML", 502);
                }

                // Read the HTML
                let html = match response.text().await {
                    Ok(html) => html,
                    Err(e) => {
                        console_log!("Error reading HTML: {:?}", e);
                        return Response::error("Failed to read source HTML", 502);
                    }
                };

                // Generate meta tags based on FID
                let meta_tags = generate_annual_report_meta_tags(fid, &base_url, &pathname);

                // Remove existing fc:miniapp, fc:frame, og:*, and twitter:* meta tags
                let html_cleaned = html
                    .lines()
                    .filter(|line| {
                        !line.contains("name=\"fc:miniapp\"")
                            && !line.contains("name=\"fc:frame\"")
                            && !line.contains("property=\"og:")
                            && !line.contains("name=\"twitter:")
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Inject meta tags before </head>
                let modified_html = if html_cleaned.contains("</head>") {
                    html_cleaned.replace("</head>", &format!("{}\n</head>", meta_tags))
                } else if html_cleaned.contains("<head>") {
                    html_cleaned.replace("<head>", &format!("<head>\n{}", meta_tags))
                } else {
                    // If no head tag, prepend to body or html
                    if html_cleaned.contains("<body>") {
                        html_cleaned.replace(
                            "<body>",
                            &format!("<head>{}</head>\n<body>", meta_tags),
                        )
                    } else {
                        format!("<head>{}</head>\n{}", meta_tags, html_cleaned)
                    }
                };

                // Return modified HTML with proper headers
                let mut response = Response::from_html(modified_html)?;
                response
                    .headers_mut()
                    .set("content-type", "text/html; charset=utf-8")?;
                return Ok(response);
            }
            Err(e) => {
                console_log!("Error fetching from source: {:?}", e);
                return Response::error("Failed to fetch source content", 502);
            }
        }
    }

    // For non-bot requests or non-annual-report routes, proxy the request
    // Get source URL from environment or use default GitHub Pages format
    let source_base_url = match env.var("SOURCE_URL") {
        Ok(url) => {
            // If SOURCE_URL is set, use it as base and append path
            let base = url.to_string();
            // Remove trailing slash if present
            let base = base.trim_end_matches('/');
            format!(
                "{}{}",
                base,
                if pathname == "/" || pathname.is_empty() {
                    "/index.html"
                } else {
                    pathname
                }
            )
        }
        Err(_) => {
            // Fallback to GitHub Pages format
            let github_username = env
                .var("GITHUB_USERNAME")
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "your-username".to_string());
            format!(
                "https://{}.github.io{}",
                github_username,
                if pathname == "/" || pathname.is_empty() {
                    "/index.html"
                } else {
                    pathname
                }
            )
        }
    };

    // Forward the request using Fetch
    match source_base_url.parse() {
        Ok(url) => Fetch::Url(url).send().await,
        Err(e) => {
            console_log!("Failed to parse proxy URL: {:?}", e);
            Response::error(format!("Invalid proxy URL: {}", e), 500)
        }
    }
}

