use serde_json::json;
use worker::*;
use image::{Rgba, RgbaImage};

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
/// If params_base64 is provided, use generated report card image instead of tarot card
fn generate_annual_report_meta_tags(fid: i64, base_url: &str, pathname: &str, params_base64: Option<&str>) -> String {
    // Determine image URL: use generated report card if params are provided, otherwise use tarot card
    let image_url = if let Some(params) = params_base64 {
        // Use generated report card image
        format!("{}/api/generate?params={}", base_url, params)
    } else {
        // Use tarot card image
        let (_tarot_name, tarot_filename) = calculate_tarot_card(fid);
        format!("{}/imgs/tarot/{}", base_url, tarot_filename)
    };
    let target_url = format!("{}{}", base_url, pathname);

    // Create embed JSON matching the format from embed.rs (without imageUrl, ember will provide image)
    let embed_json = json!({
        "version": "1",
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

    // Generate frame JSON (for backward compatibility, without imageUrl)
    let frame_json = json!({
        "version": "1",
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

    // Generate Open Graph meta tags (without image URLs, ember will provide image)
    format!(
        r#"<meta name="fc:miniapp" content='{}' />
<meta name="fc:frame" content='{}' />
<meta property="og:title" content="2025 Annual Report - Polyjuice" />
<meta property="og:description" content="View my Farcaster 2025 Annual Report" />
<meta property="og:url" content="{}" />
<meta property="og:type" content="website" />
<meta name="twitter:card" content="summary" />
<meta name="twitter:title" content="2025 Annual Report - Polyjuice" />
<meta name="twitter:description" content="View my Farcaster 2025 Annual Report" />"#,
        embed_json_str, frame_json_str, target_url
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

/// Decoded image params with user info and stats
#[derive(Debug)]
struct ImageParams {
    fid: i64,
    zodiac_index: u8,      // 0-11
    social_type_index: u8, // 0=silent, 1=social
    total_casts: usize,
    total_reactions: usize,
    total_followers: usize,
}

/// Profile data fetched from API
#[derive(Debug, serde::Deserialize)]
struct ProfileApiResponse {
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    pfp_url: Option<String>,
}

/// Get zodiac image URL from index (0-11)
fn get_zodiac_url_from_index(index: u8, base_url: &str) -> String {
    let zodiacs = [
        "capricorn", "aquarius", "pisces", "aries", "taurus", "gemini",
        "cancer", "leo", "virgo", "libra", "scorpio", "sagittarius",
    ];
    let zodiac_name = if (index as usize) < zodiacs.len() {
        zodiacs[index as usize]
    } else {
        "capricorn"
    };
    format!("{}/imgs/zodiac/{}.png", base_url, zodiac_name)
}

/// Get social type image URL from index (0=silent, 1=social)
fn get_social_type_url_from_index(index: u8, base_url: &str) -> String {
    if index == 1 {
        format!("{}/imgs/social_type/social.png", base_url)
    } else {
        format!("{}/imgs/social_type/slient.png", base_url)
    }
}

/// Fetch profile from API
async fn fetch_profile_from_api(fid: i64, api_url: &str) -> Result<(Option<String>, Option<String>), String> {
    let url = format!("{}/api/profiles/fid/{}", api_url.trim_end_matches('/'), fid);
    
    console_log!("📡 Fetching profile for FID {} from: {}", fid, url);
    
    let request = Request::new(&url, Method::Get)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let mut response = Fetch::Request(request)
        .send()
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    
    if response.status_code() != 200 {
        console_log!("⚠️ Profile API returned status: {}", response.status_code());
        return Ok((None, None)); // Return None if not found
    }
    
    let text = response.text().await
        .map_err(|e| format!("Failed to read response: {:?}", e))?;
    
    // Try to parse as ApiResponse<ProfileData>
    let api_response: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // Extract profile data
    let profile = api_response.get("data")
        .or_else(|| api_response.get("profile"));
    
    if let Some(profile_data) = profile {
        let username = profile_data.get("username")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let avatar_url = profile_data.get("pfp_url")
            .or_else(|| profile_data.get("avatar"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        
        console_log!("✅ Fetched profile: username={:?}, avatar={:?}", username, avatar_url);
        Ok((username, avatar_url))
    } else {
        console_log!("⚠️ No profile data in API response");
        Ok((None, None))
    }
}

/// Decode base64 params from compact binary format
/// Format: [0-7]: FID (i64, little-endian), [8]: Zodiac (u8, 0-11), [9]: Social type (u8, 0=silent, 1=social),
///         [10-13]: Total casts (u32), [14-17]: Total reactions (u32), [18-21]: Total followers (u32)
fn decode_image_params(params_base64: &str) -> Result<ImageParams, String> {
    use base64::engine::general_purpose;
    use base64::Engine;
    
    // Decode base64url (URL-safe base64) or standard base64
    // Convert base64url to standard base64 format
    let base64_str = params_base64.replace('-', "+").replace('_', "/");
    
    // Try decoding with padding, if fails try without padding
    let decoded_bytes = general_purpose::STANDARD
        .decode(&base64_str)
        .or_else(|_| {
            // Try with padding
            let mut padded = base64_str.clone();
            while padded.len() % 4 != 0 {
                padded.push('=');
            }
            general_purpose::STANDARD.decode(&padded)
        })
        .map_err(|e| format!("Failed to decode base64: {}", e))?;
    
    // Check minimum length (22 bytes)
    if decoded_bytes.len() < 22 {
        return Err(format!("Invalid params length: {} bytes (expected 22)", decoded_bytes.len()));
    }
    
    // Parse binary format
    // FID (8 bytes, little-endian)
    let fid_bytes: [u8; 8] = [
        decoded_bytes[0], decoded_bytes[1], decoded_bytes[2], decoded_bytes[3],
        decoded_bytes[4], decoded_bytes[5], decoded_bytes[6], decoded_bytes[7],
    ];
    let fid = i64::from_le_bytes(fid_bytes);
    
    // Zodiac index (1 byte)
    let zodiac_index = decoded_bytes[8];
    
    // Social type index (1 byte)
    let social_type_index = decoded_bytes[9];
    
    // Total casts (4 bytes, little-endian)
    let casts_bytes: [u8; 4] = [
        decoded_bytes[10], decoded_bytes[11], decoded_bytes[12], decoded_bytes[13],
    ];
    let total_casts = u32::from_le_bytes(casts_bytes) as usize;
    
    // Total reactions (4 bytes, little-endian)
    let reactions_bytes: [u8; 4] = [
        decoded_bytes[14], decoded_bytes[15], decoded_bytes[16], decoded_bytes[17],
    ];
    let total_reactions = u32::from_le_bytes(reactions_bytes) as usize;
    
    // Total followers (4 bytes, little-endian)
    let followers_bytes: [u8; 4] = [
        decoded_bytes[18], decoded_bytes[19], decoded_bytes[20], decoded_bytes[21],
    ];
    let total_followers = u32::from_le_bytes(followers_bytes) as usize;
    
    Ok(ImageParams {
        fid,
        zodiac_index,
        social_type_index,
        total_casts,
        total_reactions,
        total_followers,
    })
}

/// Fetch image data from URL using Worker Fetch API
async fn fetch_image_data(url: &str) -> Result<Vec<u8>, String> {
    // Parse URL using worker's Request API
    let request = Request::new(url, Method::Get)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;
    
    let mut response = Fetch::Request(request)
        .send()
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;
    
    if response.status_code() != 200 {
        return Err(format!("Failed to fetch image: status {}", response.status_code()));
    }
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response bytes: {:?}", e))?;
    
    Ok(bytes.to_vec())
}

/// Resize image and add circular border (2px low-saturation blue)
fn resize_with_circular_border(img: &RgbaImage, size: u32) -> RgbaImage {
    // Resize image
    let resized = image::imageops::resize(
        img,
        size,
        size,
        image::imageops::FilterType::Lanczos3,
    );
    
    // Create canvas with border (size + 4px for 2px border on each side)
    let canvas_size = size + 4;
    let mut canvas = RgbaImage::new(canvas_size, canvas_size);
    
    // Fill with transparent
    for pixel in canvas.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    // Draw circular mask and copy resized image
    let center = (canvas_size as f32 / 2.0, canvas_size as f32 / 2.0);
    let radius = (size as f32 / 2.0) as f32;
    let border_radius = radius + 2.0;
    
    for y in 0..canvas_size {
        for x in 0..canvas_size {
            let dx = x as f32 - center.0;
            let dy = y as f32 - center.1;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist <= radius {
                // Inside circle - copy from resized image
                let src_x = ((x as f32 - 2.0).max(0.0).min(size as f32 - 1.0)) as u32;
                let src_y = ((y as f32 - 2.0).max(0.0).min(size as f32 - 1.0)) as u32;
                canvas.put_pixel(x, y, *resized.get_pixel(src_x, src_y));
            } else if dist <= border_radius {
                // Border area - draw low-saturation blue border
                canvas.put_pixel(x, y, Rgba([122, 156, 198, 255])); // Low-saturation blue #7A9CC6
            }
        }
    }
    
    canvas
}

/// Resize image, crop to circle, and add circular border (2px) - for avatars
fn resize_with_circular_border_cropped(img: &RgbaImage, size: u32) -> RgbaImage {
    // Resize image to square first
    let resized = image::imageops::resize(
        img,
        size,
        size,
        image::imageops::FilterType::Lanczos3,
    );
    
    // Create canvas with border (size + 4px for 2px border on each side)
    let canvas_size = size + 4;
    let mut canvas = RgbaImage::new(canvas_size, canvas_size);
    
    // Fill with transparent
    for pixel in canvas.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    // Draw circular mask and copy resized image (circular crop)
    let center = (canvas_size as f32 / 2.0, canvas_size as f32 / 2.0);
    let radius = (size as f32 / 2.0) as f32;
    let border_radius = radius + 2.0;
    
    for y in 0..canvas_size {
        for x in 0..canvas_size {
            let dx = x as f32 - center.0;
            let dy = y as f32 - center.1;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist <= radius {
                // Inside circle - copy from resized image (with circular crop)
                let src_x = ((x as f32 - 2.0).max(0.0).min(size as f32 - 1.0)) as u32;
                let src_y = ((y as f32 - 2.0).max(0.0).min(size as f32 - 1.0)) as u32;
                
                // Check if source pixel is inside circle (circular crop)
                let src_center = (size as f32 / 2.0, size as f32 / 2.0);
                let src_dx = src_x as f32 - src_center.0;
                let src_dy = src_y as f32 - src_center.1;
                let src_dist = (src_dx * src_dx + src_dy * src_dy).sqrt();
                
                if src_dist <= radius {
                    canvas.put_pixel(x, y, *resized.get_pixel(src_x, src_y));
                }
            } else if dist <= border_radius {
                // Border area - draw low-saturation blue border
                canvas.put_pixel(x, y, Rgba([122, 156, 198, 255])); // Low-saturation blue #7A9CC6
            }
        }
    }
    
    canvas
}

/// Overlay one image onto another at specified position with alpha blending
fn overlay_image(canvas: &mut RgbaImage, overlay: &RgbaImage, x: u32, y: u32) {
    for (ox, oy, pixel) in overlay.enumerate_pixels() {
        let canvas_x = x + ox;
        let canvas_y = y + oy;

        if canvas_x < canvas.width() && canvas_y < canvas.height() {
            let canvas_pixel = canvas.get_pixel_mut(canvas_x, canvas_y);
            *canvas_pixel = blend_pixels(*canvas_pixel, *pixel);
        }
    }
}

/// Calculate actual text width using font metrics
fn calculate_text_width(font: &rusttype::Font, text: &str, scale: rusttype::Scale) -> f32 {
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(text, scale, rusttype::Point { x: 0.0, y: v_metrics.ascent }).collect();
    
    if let Some(last_glyph) = glyphs.last() {
        if let Some(bounding_box) = last_glyph.pixel_bounding_box() {
            return (bounding_box.max.x as f32) + last_glyph.unpositioned().h_metrics().advance_width;
        }
    }
    
    // Fallback: estimate width
    text.len() as f32 * scale.x * 0.6
}

/// Calculate text height using font metrics (ascent + descent)
fn calculate_text_height(font: &rusttype::Font, scale: rusttype::Scale) -> f32 {
    let v_metrics = font.v_metrics(scale);
    v_metrics.ascent - v_metrics.descent + v_metrics.line_gap
}

/// Draw text with bold numbers (numbers are +3px larger and drawn twice with 2px offset for bold effect)
fn draw_text_with_bold_numbers(
    canvas: &mut RgbaImage,
    font: &rusttype::Font,
    text: &str,
    x: i32,
    y: i32,
    base_scale: f32,
    number_scale: f32,
    color: Rgba<u8>,
) {
    use rusttype::Scale;
    use imageproc::drawing::draw_text_mut;
    
    let mut x_pos = x;
    let mut current_segment = String::new();
    let mut is_number_segment = false;
    
    for c in text.chars() {
        let is_digit = c.is_ascii_digit();
        
        if is_digit != is_number_segment && !current_segment.is_empty() {
            // Draw accumulated segment
            let scale = if is_number_segment { Scale::uniform(number_scale) } else { Scale::uniform(base_scale) };
            draw_text_mut(canvas, color, x_pos, y, scale, font, &current_segment);
            // Draw again with 2px offset for bold effect (only for numbers)
            if is_number_segment {
                draw_text_mut(canvas, color, x_pos + 2, y, scale, font, &current_segment);
            }
            // Calculate actual width
            let actual_width = calculate_text_width(font, &current_segment, scale);
            x_pos += actual_width as i32;
            current_segment.clear();
        }
        
        is_number_segment = is_digit;
        current_segment.push(c);
    }
    
    // Draw remaining segment
    if !current_segment.is_empty() {
        let scale = if is_number_segment { Scale::uniform(number_scale) } else { Scale::uniform(base_scale) };
        draw_text_mut(canvas, color, x_pos, y, scale, font, &current_segment);
        if is_number_segment {
            draw_text_mut(canvas, color, x_pos + 2, y, scale, font, &current_segment);
        }
    }
}

/// Alpha blend two pixels
fn blend_pixels(bottom: Rgba<u8>, top: Rgba<u8>) -> Rgba<u8> {
    let alpha_top = top[3] as f32 / 255.0;
    let alpha_bottom = bottom[3] as f32 / 255.0;
    let alpha_out = alpha_top + alpha_bottom * (1.0 - alpha_top);

    if alpha_out == 0.0 {
        return bottom;
    }

    let r = ((top[0] as f32 * alpha_top + bottom[0] as f32 * alpha_bottom * (1.0 - alpha_top))
        / alpha_out) as u8;
    let g = ((top[1] as f32 * alpha_top + bottom[1] as f32 * alpha_bottom * (1.0 - alpha_top))
        / alpha_out) as u8;
    let b = ((top[2] as f32 * alpha_top + bottom[2] as f32 * alpha_bottom * (1.0 - alpha_top))
        / alpha_out) as u8;
    let a = (alpha_out * 255.0) as u8;

    Rgba([r, g, b, a])
}

/// Composite images: overlay zodiac, social type, and avatar badges on tarot card
/// Returns PNG bytes
async fn composite_tarot_with_badges(
    tarot_url: &str,
    zodiac_url: &str,
    social_type_url: &str,
    avatar_url: Option<&str>,
) -> Result<Vec<u8>, String> {
    console_log!("📥 Fetching tarot image from: {}", tarot_url);
    // Fetch all images
    let tarot_data = fetch_image_data(tarot_url).await
        .map_err(|e| format!("Failed to fetch tarot image: {}", e))?;
    console_log!("✅ Fetched tarot image: {} bytes", tarot_data.len());
    
    console_log!("📥 Fetching zodiac image from: {}", zodiac_url);
    let zodiac_data = fetch_image_data(zodiac_url).await
        .map_err(|e| format!("Failed to fetch zodiac image: {}", e))?;
    console_log!("✅ Fetched zodiac image: {} bytes", zodiac_data.len());
    
    console_log!("📥 Fetching social type image from: {}", social_type_url);
    let social_type_data = fetch_image_data(social_type_url).await
        .map_err(|e| format!("Failed to fetch social type image: {}", e))?;
    console_log!("✅ Fetched social type image: {} bytes", social_type_data.len());
    
    let avatar_data = if let Some(url) = avatar_url {
        console_log!("📥 Fetching avatar image from: {}", url);
        match fetch_image_data(url).await {
            Ok(data) => {
                console_log!("✅ Fetched avatar image: {} bytes", data.len());
                Some(data)
            }
            Err(e) => {
                console_log!("⚠️ Failed to fetch avatar image: {}, continuing without it", e);
                None
            }
        }
    } else {
        console_log!("ℹ️ No avatar URL provided, skipping avatar");
        None
    };

    // Load images
    console_log!("🖼️ Loading images from memory...");
    let tarot_img = image::load_from_memory(&tarot_data)
        .map_err(|e| format!("Failed to load tarot image: {:?}", e))?
        .to_rgba8();
    console_log!("✅ Loaded tarot image: {}x{}", tarot_img.width(), tarot_img.height());
    
    let zodiac_img = image::load_from_memory(&zodiac_data)
        .map_err(|e| format!("Failed to load zodiac image: {:?}", e))?
        .to_rgba8();
    console_log!("✅ Loaded zodiac image: {}x{}", zodiac_img.width(), zodiac_img.height());
    
    let social_type_img = image::load_from_memory(&social_type_data)
        .map_err(|e| format!("Failed to load social type image: {:?}", e))?
        .to_rgba8();
    console_log!("✅ Loaded social type image: {}x{}", social_type_img.width(), social_type_img.height());

    let avatar_img = if let Some(data) = avatar_data {
        console_log!("🖼️ Loading avatar image from memory ({} bytes)...", data.len());
        match image::load_from_memory(&data) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                console_log!("✅ Loaded avatar image: {}x{}", rgba.width(), rgba.height());
                Some(rgba)
            }
            Err(e) => {
                console_log!("❌ Failed to load avatar image: {:?}", e);
                None
            }
        }
    } else {
        console_log!("ℹ️ No avatar data to load");
        None
    };

    // Get tarot card dimensions
    let tarot_width = tarot_img.width();
    let tarot_height = tarot_img.height();
    console_log!("📐 Tarot card dimensions: {}x{}", tarot_width, tarot_height);

    // Badge size is fixed at 50px, avatar is larger (70px)
    let badge_size = 50u32;
    let avatar_size = 70u32; // Avatar is larger than badges
    console_log!("📏 Badge size: {}px, Avatar size: {}px", badge_size, avatar_size);
    
    // Resize badges to badge_size and make them circular with border
    let zodiac_resized = resize_with_circular_border(
        &zodiac_img,
        badge_size,
    );
    let social_type_resized = resize_with_circular_border(
        &social_type_img,
        badge_size,
    );
    let avatar_resized = if let Some(ref avatar) = avatar_img {
        console_log!("🔄 Resizing avatar to {}px with circular border...", avatar_size);
        let resized = resize_with_circular_border_cropped(
            avatar,
            avatar_size,
        );
        console_log!("✅ Avatar resized to {}x{}", resized.width(), resized.height());
        Some(resized)
    } else {
        console_log!("⚠️ No avatar image to resize");
        None
    };

    // Calculate top section height = avatar diameter (including border)
    // Avatar has 2px border on each side, so actual size is avatar_size + 4
    let avatar_actual_size = avatar_size + 4;
    // Top section height should match avatar diameter exactly (including border)
    // This ensures the border aligns with the top and bottom edges of the circular avatar
    let top_section_height = avatar_actual_size;
    console_log!("📐 Top section height: {}px (avatar diameter with border: {}px)", top_section_height, avatar_actual_size);
    
    // Create canvas with extra height for the top border (outside the card)
    let canvas_height = tarot_height + top_section_height;
    let mut canvas = RgbaImage::new(tarot_width, canvas_height);
    
    // Fill canvas with transparent
    for pixel in canvas.pixels_mut() {
        *pixel = Rgba([0, 0, 0, 0]);
    }
    
    // Draw low-saturation blue border at top (outside the card, height = avatar diameter)
    // Fill the entire top section with low-saturation blue (#7A9CC6 - soft blue-gray)
    for y in 0..top_section_height {
        for x in 0..tarot_width {
            canvas.put_pixel(x, y, Rgba([122, 156, 198, 255])); // Low-saturation blue #7A9CC6
        }
    }
    console_log!("✅ Low-saturation blue border drawn at top (height: {}px, same as avatar diameter)", top_section_height);
    
    // Copy tarot card image below the border
    for y in 0..tarot_height {
        for x in 0..tarot_width {
            let pixel = tarot_img.get_pixel(x, y);
            canvas.put_pixel(x, y + top_section_height, *pixel);
        }
    }
    console_log!("✅ Tarot card image placed below border");
    
    // Calculate positions for badges and avatar in top section (outside the card)
    // Note: badges have 2px border on each side, so actual size is badge_size + 4
    // Avatar has 2px border on each side, so actual size is avatar_size + 4
    let badge_actual_size = badge_size + 4;
    let avatar_actual_size = avatar_size + 4;
    
    // Avatar should be positioned so its top edge aligns with top border (y=0)
    // and bottom edge aligns with bottom border (y=top_section_height)
    // Since avatar_actual_size = top_section_height, avatar should start at y=0
    let avatar_y = 0u32;
    
    // Badges should be vertically centered in the top section
    let center_y = (top_section_height / 2) as i32;
    let badge_center_y = center_y - (badge_actual_size as i32 / 2);
    
    console_log!("📍 Avatar position: y={} (top edge at border top, bottom edge at border bottom)", avatar_y);
    
    // Horizontal spacing: left badge, center avatar, right badge
    let padding = 20u32; // Padding from edges
    let left_badge_x = padding;
    let right_badge_x = tarot_width.saturating_sub(badge_actual_size + padding);
    let avatar_x = (tarot_width as i32 / 2) - (avatar_actual_size as i32 / 2);
    
    console_log!("📍 Positioning: left_badge=({}, {}), avatar=({}, {}), right_badge=({}, {})", 
        left_badge_x, badge_center_y, avatar_x, avatar_y, right_badge_x, badge_center_y);
    
    // Top-left: zodiac badge (in top section, outside card)
    if badge_center_y >= 0 {
        console_log!("📍 Overlaying zodiac badge at ({}, {})", left_badge_x, badge_center_y as u32);
        overlay_image(&mut canvas, &zodiac_resized, left_badge_x, badge_center_y as u32);
    }
    
    // Top-center: avatar (larger, in top section, outside card)
    // Avatar top edge aligns with border top (y=0), bottom edge aligns with border bottom
    if let Some(ref avatar) = avatar_resized {
        if avatar_x >= 0 {
            console_log!("📍 Overlaying avatar at ({}, {}) - top edge at border top", avatar_x as u32, avatar_y);
            overlay_image(&mut canvas, avatar, avatar_x as u32, avatar_y);
        }
    } else {
        console_log!("⚠️ No avatar to overlay");
    }
    
    // Top-right: social type badge (in top section, outside card)
    if badge_center_y >= 0 {
        console_log!("📍 Overlaying social type badge at ({}, {})", right_badge_x, badge_center_y as u32);
        overlay_image(&mut canvas, &social_type_resized, right_badge_x, badge_center_y as u32);
    }
    
    console_log!("✅ All badges and avatar overlaid in top section (outside card)");

    // Encode to PNG
    console_log!("💾 Encoding composite image to PNG...");
    
    // Resize image to target file size (~200KB)
    // Note: canvas now includes top section, so height is tarot_height + top_section_height
    let canvas_width = tarot_width;
    let canvas_height_with_border = canvas_height;
    
    // PNG compression ratio for typical images: ~3-5x
    // Target: ~200KB = 200,000 bytes compressed
    // Raw data needed: ~600KB-1MB = ~150K-250K pixels (RGBA8 = 4 bytes/pixel)
    // For 687x1024 aspect ratio: target ~550x820 pixels = ~450K pixels = ~1.8MB raw ≈ ~200KB compressed
    let target_max_dimension = 900u32; // Higher resolution for better quality while keeping ~200KB
    let (final_width, final_height, final_canvas) = if canvas_width > target_max_dimension || canvas_height_with_border > target_max_dimension {
        let scale = (target_max_dimension as f32 / canvas_width.max(canvas_height_with_border) as f32).min(1.0);
        let new_width = (canvas_width as f32 * scale) as u32;
        let new_height = (canvas_height_with_border as f32 * scale) as u32;
        console_log!("📐 Resizing composite from {}x{} to {}x{} for target file size (~200KB)", canvas_width, canvas_height_with_border, new_width, new_height);
        let resized = image::imageops::resize(
            &canvas,
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        (new_width, new_height, image::DynamicImage::ImageRgba8(resized))
    } else {
        console_log!("📐 Keeping original size {}x{}", canvas_width, canvas_height_with_border);
        (canvas_width, canvas_height_with_border, image::DynamicImage::ImageRgba8(canvas))
    };
    
    let mut png_bytes = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        final_canvas
            .write_to(&mut cursor, image::ImageOutputFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {:?}", e))?;
    }
    let file_size_kb = png_bytes.len() as f32 / 1024.0;
    console_log!("✅ Encoded PNG: {} bytes ({:.1}KB)", png_bytes.len(), file_size_kb);
    
    // If file is too large (>250KB), resize further to target ~200KB
    if png_bytes.len() > 250_000 {
        console_log!("⚠️ File size {:.1}KB exceeds target, resizing further...", file_size_kb);
        let scale = (200_000.0 / png_bytes.len() as f32).sqrt(); // Square root to account for 2D scaling
        let new_width = ((final_width as f32 * scale) as u32).max(400);
        let new_height = ((final_height as f32 * scale) as u32).max(600);
        console_log!("📐 Resizing to {}x{} to reduce file size", new_width, new_height);
        let resized = image::imageops::resize(
            &final_canvas.to_rgba8(),
            new_width,
            new_height,
            image::imageops::FilterType::Lanczos3,
        );
        png_bytes.clear();
        {
            let mut cursor = std::io::Cursor::new(&mut png_bytes);
            image::DynamicImage::ImageRgba8(resized)
                .write_to(&mut cursor, image::ImageOutputFormat::Png)
                .map_err(|e| format!("Failed to encode PNG: {:?}", e))?;
        }
        let new_file_size_kb = png_bytes.len() as f32 / 1024.0;
        console_log!("✅ Re-encoded PNG: {} bytes ({:.1}KB)", png_bytes.len(), new_file_size_kb);
    }
    
    Ok(png_bytes)
}

/// Generate report card image with user info, stats, and tarot card
/// Layout: Left side (avatar, username, fid, stats, badges), Right side (tarot card)
async fn generate_report_card(
    tarot_url: &str,
    params: &ImageParams,
    base_url: &str,
    api_url: &str,
) -> Result<Vec<u8>, String> {
    use rusttype::{Font, Scale};
    use imageproc::drawing::draw_text_mut;
    
    // First, fetch and load tarot card to get its actual dimensions
    let tarot_data = fetch_image_data(tarot_url).await
        .map_err(|e| format!("Failed to fetch tarot card: {}", e))?;
    
    let tarot_img = image::load_from_memory(&tarot_data)
        .map_err(|e| format!("Failed to load tarot image: {:?}", e))?
        .to_rgba8();
    
    let original_tarot_width = tarot_img.width();
    let original_tarot_height = tarot_img.height();
    console_log!("📐 Original tarot card dimensions: {}x{}", original_tarot_width, original_tarot_height);
    
    // Load font first (embedded in binary)
    let font_data = include_bytes!("../fonts/Roboto-Regular.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .ok_or_else(|| "Failed to load font".to_string())?;
    
    // Card dimensions: height equals tarot card height + banner, width is double tarot card width
    // This creates a 50/50 split: left side for info, right side for tarot card
    let banner_height = 80u32; // Black banner height
    let card_height = original_tarot_height + banner_height;
    let card_width = original_tarot_width * 2; // 2 * tarot width for 50/50 split
    let mut canvas = RgbaImage::new(card_width, card_height);
    
    console_log!("📐 Report card dimensions: {}x{} (2x tarot width, with {}px banner)", card_width, card_height, banner_height);
    
    // 1. Draw black banner at top
    for y in 0..banner_height {
        for x in 0..card_width {
            canvas.put_pixel(x, y, Rgba([0, 0, 0, 255])); // Black
        }
    }
    
    // 2. Fill rest with blue-purple gradient background
    // Gradient from blue (#667eea) to purple (#764ba2)
    for y in banner_height..card_height {
        let ratio = (y - banner_height) as f32 / original_tarot_height as f32;
        // Interpolate between blue and purple
        let r = (102.0 + (118.0 - 102.0) * ratio) as u8; // 102 -> 118
        let g = (126.0 + (75.0 - 126.0) * ratio) as u8;  // 126 -> 75
        let b = (234.0 + (162.0 - 234.0) * ratio) as u8; // 234 -> 162
        for x in 0..card_width {
            canvas.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    
    // 3. Draw "My 2025 Annual Report" text in banner (centered, positioned higher)
    let banner_text = "My 2025 Annual Report";
    let banner_font_size = 48.0; // Larger font
    let banner_scale = Scale::uniform(banner_font_size);
    let banner_v_metrics = font.v_metrics(banner_scale);
    let banner_text_width = calculate_text_width(&font, banner_text, banner_scale);
    let banner_text_x = ((card_width as f32 - banner_text_width) / 2.0) as i32;
    // Position text higher in banner - move up by at least half text height
    // draw_text_mut uses baseline, text height = ascent - descent
    // Move up by half text height: baseline = top_offset + (ascent + descent) / 2
    let top_offset = 10.0; // Smaller offset from banner top
    let text_height = banner_v_metrics.ascent - banner_v_metrics.descent;
    let banner_text_y = (top_offset + banner_v_metrics.ascent - text_height / 2.0) as i32;
    draw_text_mut(&mut canvas, Rgba([255, 255, 255, 255]), banner_text_x, banner_text_y, banner_scale, &font, banner_text);
    
    // Left side: User info and stats (new layout: avatar + username/fid, then stats)
    let left_padding = 40u32;
    let top_padding = 40u32;
    let bottom_padding = 40u32;
    
    // Fixed sizes
    let avatar_size = 120u32;
    let badge_size = 90u32;
    let avatar_text_gap = 20u32; // Gap between avatar and username/fid
    
    // Content area starts after banner
    let content_start_y = banner_height as f32;
    
    // Calculate badge position first to ensure text doesn't overlap
    let badge_y = (card_height - bottom_padding - badge_size) as u32;
    let badge_top = badge_y as f32;
    
    // Calculate available height for stats (after avatar section and one blank line)
    let avatar_section_height = avatar_size as f32 + 20.0; // Avatar + username/fid area
    let blank_line_height = 30.0; // One blank line
    let available_height = badge_top - (content_start_y + top_padding as f32) - avatar_section_height - blank_line_height;
    
    // Text elements: 3 stats lines
    let line_height_ratio = 1.3; // Compact line spacing (1.3x font size)
    
    // Calculate optimal font sizes for stats
    let stats_font_size = (available_height / (3.0 * line_height_ratio)).max(28.0).min(60.0);
    let stats_number_font_size = stats_font_size + 8.0; // +8px for numbers
    
    // Username and FID font sizes (fixed relative to avatar)
    let username_font_size = 48.0; // Larger size for username
    let fid_font_size = 24.0; // Fixed size for FID
    
    console_log!("📐 Font sizes: username={:.1}px, fid={:.1}px, stats={:.1}px, numbers={:.1}px", 
                 username_font_size, fid_font_size, stats_font_size, stats_number_font_size);
    console_log!("📐 Available height for stats: {:.1}px, Badge top: {:.1}px", available_height, badge_top);
    
    let avatar_y = content_start_y + top_padding as f32;
    let avatar_x = left_padding as f32;
    
    // Fetch profile from API
    let (username, avatar_url) = match fetch_profile_from_api(params.fid, api_url).await {
        Ok(profile) => profile,
        Err(e) => {
            console_log!("⚠️ Failed to fetch profile: {}", e);
            (None, None)
        }
    };
    
    // 1. Avatar (top-left)
    if let Some(ref avatar_url) = avatar_url {
        match fetch_image_data(avatar_url).await {
            Ok(avatar_data) => {
                if let Ok(avatar_img) = image::load_from_memory(&avatar_data) {
                    let avatar_rgba = avatar_img.to_rgba8();
                    let avatar_resized = resize_with_circular_border_cropped(&avatar_rgba, avatar_size);
                    overlay_image(&mut canvas, &avatar_resized, avatar_x as u32, avatar_y as u32);
                }
            }
            Err(e) => console_log!("⚠️ Failed to fetch avatar: {}", e),
        }
    }
    
    // 2. Username (right of avatar, vertically centered with avatar)
    if let Some(ref username) = username {
        if !username.is_empty() {
            let username_text = format!("@{}", username);
        let scale = Scale::uniform(username_font_size);
        let v_metrics = font.v_metrics(scale);
        // Center username vertically with avatar
        let username_baseline_y = avatar_y + (avatar_size as f32 / 2.0) - (v_metrics.ascent - v_metrics.descent) / 2.0;
        let username_x = avatar_x + avatar_size as f32 + avatar_text_gap as f32;
        draw_text_mut(&mut canvas, Rgba([255, 255, 255, 255]), username_x as i32, username_baseline_y as i32, scale, &font, &username_text);
        }
    }
    
    // 3. FID (below avatar, left-aligned with avatar)
    let fid_text = format!("FID: {}", params.fid);
    let scale = Scale::uniform(fid_font_size);
    let v_metrics = font.v_metrics(scale);
    let fid_baseline_y = avatar_y + avatar_size as f32 + 10.0; // Small gap below avatar
    let fid_baseline = fid_baseline_y + v_metrics.ascent;
    draw_text_mut(&mut canvas, Rgba([255, 255, 255, 200]), avatar_x as i32, fid_baseline as i32, scale, &font, &fid_text);
    
    // 4. Blank line (one line height)
    let mut y_pos = fid_baseline_y + calculate_text_height(&font, scale) * line_height_ratio + blank_line_height;
    
    // 5. Stats (using font metrics, numbers bold and larger)
    let stats_scale = Scale::uniform(stats_font_size);
    let stats_text_height = calculate_text_height(&font, stats_scale);
    let stats_v_metrics = font.v_metrics(stats_scale);
    
    // Format without spaces around numbers
    let stats_text = format!("Published{}Casts", params.total_casts);
    let baseline_y = y_pos + stats_v_metrics.ascent;
    draw_text_with_bold_numbers(&mut canvas, &font, &stats_text, left_padding as i32, baseline_y as i32, stats_font_size, stats_number_font_size, Rgba([255, 255, 255, 255]));
    y_pos += stats_text_height * line_height_ratio;
    
    let reactions_text = format!("Received{}Reactions", params.total_reactions);
    let baseline_y = y_pos + stats_v_metrics.ascent;
    draw_text_with_bold_numbers(&mut canvas, &font, &reactions_text, left_padding as i32, baseline_y as i32, stats_font_size, stats_number_font_size, Rgba([255, 255, 255, 255]));
    y_pos += stats_text_height * line_height_ratio;
    
    let followers_text = format!("Gained{}Followers", params.total_followers);
    let baseline_y = y_pos + stats_v_metrics.ascent;
    draw_text_with_bold_numbers(&mut canvas, &font, &followers_text, left_padding as i32, baseline_y as i32, stats_font_size, stats_number_font_size, Rgba([255, 255, 255, 255]));
    y_pos += stats_text_height * line_height_ratio;
    
    // Verify text doesn't overlap with badge
    if y_pos > badge_top - 10.0 {
        console_log!("⚠️ Warning: Text area ({:.1}px) may overlap with badge area ({:.1}px)", y_pos, badge_top);
    }
    
    // 6. Badges (bottom, already calculated above)
    // Get zodiac URL from index
    let zodiac_url = get_zodiac_url_from_index(params.zodiac_index, base_url);
    match fetch_image_data(&zodiac_url).await {
        Ok(zodiac_data) => {
            if let Ok(zodiac_img) = image::load_from_memory(&zodiac_data) {
                let zodiac_rgba = zodiac_img.to_rgba8();
                let zodiac_resized = resize_with_circular_border(&zodiac_rgba, badge_size);
                overlay_image(&mut canvas, &zodiac_resized, left_padding, badge_y);
            }
        }
        Err(e) => console_log!("⚠️ Failed to fetch zodiac badge: {}", e),
    }
    
    // Get social type URL from index
    let social_type_url = get_social_type_url_from_index(params.social_type_index, base_url);
    match fetch_image_data(&social_type_url).await {
        Ok(social_data) => {
            if let Ok(social_img) = image::load_from_memory(&social_data) {
                let social_rgba = social_img.to_rgba8();
                let social_resized = resize_with_circular_border(&social_rgba, badge_size);
                overlay_image(&mut canvas, &social_resized, left_padding + badge_size + 20, badge_y);
            }
        }
        Err(e) => console_log!("⚠️ Failed to fetch social type badge: {}", e),
    }
    
    // Right side: Tarot card (use original dimensions, no distortion)
    // Place tarot card at the right half, maintaining original aspect ratio
    let tarot_x = card_width / 2; // Start at middle (right half)
    let tarot_y = banner_height;  // Start after banner
    
    // Use original tarot card dimensions (already loaded above)
    // No resizing needed - use original size to maintain aspect ratio
    console_log!("📍 Placing tarot card at ({}, {}) with original size {}x{}", 
        tarot_x, tarot_y, original_tarot_width, original_tarot_height);
    overlay_image(&mut canvas, &tarot_img, tarot_x, tarot_y);
    
    // Encode to PNG
    let mut png_bytes = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        image::DynamicImage::ImageRgba8(canvas)
            .write_to(&mut cursor, image::ImageOutputFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {:?}", e))?;
    }
    
    console_log!("✅ Report card generated: {} bytes", png_bytes.len());
    Ok(png_bytes)
}

/// Handle /api/generate endpoint - generate tarot card image
async fn handle_generate_image(
    req: Request,
    env: &Env,
) -> Result<Response> {
    let url = req.url()?;
    let query_params: std::collections::HashMap<String, String> = url
        .query_pairs()
        .into_owned()
        .collect();
    
    // Get params from query params
    let params_base64 = query_params
        .get("params")
        .ok_or_else(|| "Missing 'params' parameter")?;
    
    // Decode params (fid is included in params now)
    let params = decode_image_params(params_base64)
        .map_err(|e| format!("Failed to decode params: {}", e))?;
    
    console_log!("Generating report card for FID: {}", params.fid);
    console_log!("Zodiac index: {}", params.zodiac_index);
    console_log!("Social type index: {}", params.social_type_index);
    console_log!("Stats: {} casts, {} reactions, {} followers", 
        params.total_casts, params.total_reactions, params.total_followers);
    
    // Get base URL for constructing image URLs
    let base_url = env
        .var("BASE_URL")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "https://miniapp.polyjuice.io".to_string());
    
    // Get API URL for fetching profile
    let api_url = env
        .var("API_URL")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "https://api.polyjuice.io".to_string());
    
    // Calculate tarot card based on FID
    let (_tarot_name, tarot_filename) = calculate_tarot_card(params.fid);
    let tarot_image_url = format!("{}/imgs/tarot/{}", base_url, tarot_filename);
    
    // Generate report card image
    let png_bytes = generate_report_card(
        &tarot_image_url,
        &params,
        &base_url,
        &api_url,
    ).await
    .map_err(|e| format!("Failed to generate report card: {}", e))?;
    
    // Return PNG image directly
    let mut response = Response::from_bytes(png_bytes)?;
    response.headers_mut().set("content-type", "image/png")?;
    response.headers_mut().set("access-control-allow-origin", "*")?;
    response.headers_mut().set("cache-control", "public, max-age=3600")?;
    
    Ok(response)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let pathname = url.path();
    let user_agent = req.headers().get("user-agent").ok().flatten();
    
    // Handle /api/generate endpoint
    if pathname == "/api/generate" {
        return handle_generate_image(req, &env).await;
    }

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

                // Extract params from URL if present
                let params_base64 = url.query_pairs()
                    .find(|(key, _)| key == "params")
                    .map(|(_, value)| value.to_string());
                
                console_log!("📦 Meta generation - FID: {}, Has params: {}", fid, params_base64.is_some());
                
                // Generate meta tags based on FID and params
                let meta_tags = generate_annual_report_meta_tags(fid, &base_url, &pathname, params_base64.as_deref());

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

