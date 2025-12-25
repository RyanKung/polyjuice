use crate::models::AnnualReportResponse;
use crate::models::ContentStyleResponse;
use crate::models::DomainStatusResponse;
use crate::models::EngagementResponse;
use crate::models::FollowerGrowthResponse;
use crate::models::TemporalActivityResponse;

/// Farcaster epoch: 2021-01-01 00:00:00 UTC
const FARCASTER_EPOCH: i64 = 1_609_459_200;

/// Convert Farcaster timestamp to Unix timestamp
pub fn farcaster_to_unix(farcaster_timestamp: i64) -> i64 {
    farcaster_timestamp + FARCASTER_EPOCH
}

/// Convert Unix timestamp to Farcaster timestamp
#[allow(dead_code)]
pub fn unix_to_farcaster(unix_timestamp: i64) -> i64 {
    unix_timestamp - FARCASTER_EPOCH
}

/// Helper function to extract data from nested API response structure
#[allow(dead_code)]
pub fn extract_nested_data<T>(json_data: serde_json::Value) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    // Handle nested structure: data.data contains the actual data
    // The response structure can be:
    // 1. { "success": true, "data": {...} }
    // 2. { "success": true, "data": { "data": {...}, "status": "...", "message": "..." } }
    // 3. Direct data: {...}

    let data = if let Some(outer_data) = json_data.get("data") {
        // Check if this is an ApiResponse wrapper
        if outer_data.is_object() {
            // If data.data exists, use it; otherwise use data directly
            if outer_data.get("data").is_some() {
                outer_data.get("data").unwrap().clone()
            } else {
                outer_data.clone()
            }
        } else {
            outer_data.clone()
        }
    } else {
        // No "data" field, assume the whole thing is the data
        json_data
    };

    serde_json::from_value::<T>(data).map_err(|e| format!("Failed to parse response: {}", e))
}

/// Convert API annual report response to our expected format
pub fn convert_annual_report_response(
    api_data: serde_json::Value,
) -> Result<AnnualReportResponse, String> {
    // API returns: { "activity": {...}, "content_style": {...}, "engagement": {...}, "social_growth": {...}, "user": {...}, "year": 2025 }
    // We need: { "fid": ..., "username": ..., "display_name": ..., "temporal_activity": ..., "follower_growth": ..., ... }

    let user = api_data.get("user").ok_or("Missing 'user' field")?;
    let fid = user
        .get("fid")
        .and_then(|v| v.as_i64())
        .ok_or("Missing or invalid 'fid' in user")?;
    let username = user
        .get("username")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let display_name = user
        .get("display_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let year = api_data
        .get("year")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .unwrap_or(2025);

    // Convert "activity" to "temporal_activity"
    let activity = api_data.get("activity").ok_or("Missing 'activity' field")?;
    let temporal_activity = serde_json::from_value::<TemporalActivityResponse>(activity.clone())
        .map_err(|e| format!("Failed to parse temporal_activity: {}", e))?;

    // Convert "social_growth" to "follower_growth"
    let social_growth = api_data
        .get("social_growth")
        .ok_or("Missing 'social_growth' field")?;
    let current_followers = social_growth
        .get("current_followers")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid 'current_followers' in social_growth")?
        as usize;
    let followers_at_start = social_growth
        .get("followers_at_start")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid 'followers_at_start' in social_growth")?
        as usize;
    let net_growth = social_growth
        .get("net_growth")
        .and_then(|v| v.as_i64())
        .ok_or("Missing or invalid 'net_growth' in social_growth")?;
    let monthly_snapshots = social_growth
        .get("monthly_snapshots")
        .ok_or("Missing 'monthly_snapshots' in social_growth")?
        .clone();

    let follower_growth = serde_json::json!({
        "current_followers": current_followers,
        "followers_at_start": followers_at_start,
        "net_growth": net_growth,
        "monthly_snapshots": monthly_snapshots
    });
    let follower_growth = serde_json::from_value::<FollowerGrowthResponse>(follower_growth)
        .map_err(|e| format!("Failed to parse follower_growth: {}", e))?;

    // Parse engagement
    let engagement = api_data
        .get("engagement")
        .ok_or("Missing 'engagement' field")?;
    let engagement = serde_json::from_value::<EngagementResponse>(engagement.clone())
        .map_err(|e| format!("Failed to parse engagement: {}", e))?;

    // Parse content_style - require all fields
    let content_style_raw = api_data
        .get("content_style")
        .ok_or("Missing 'content_style' field")?;

    // Parse top_emojis
    let top_emojis = content_style_raw
        .get("top_emojis")
        .ok_or("Missing 'top_emojis' in content_style")?
        .clone();

    // Parse top_words and calculate percentage
    let top_words_raw = content_style_raw
        .get("top_words")
        .and_then(|v| v.as_array())
        .ok_or("Missing or invalid 'top_words' in content_style")?;

    // Calculate total count for percentage calculation
    let total_word_count: usize = top_words_raw
        .iter()
        .filter_map(|w| w.get("count").and_then(|v| v.as_u64()).map(|v| v as usize))
        .sum();

    // Convert top_words with calculated percentage
    let top_words: Vec<serde_json::Value> = top_words_raw
        .iter()
        .map(|word| {
            let word_str = word
                .get("word")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'word' in top_words item")
                .map_err(|e: &str| e.to_string())?;
            let count = word
                .get("count")
                .and_then(|v| v.as_u64())
                .ok_or("Missing 'count' in top_words item")
                .map_err(|e: &str| e.to_string())? as usize;
            let percentage = if total_word_count > 0 {
                (count as f32 / total_word_count as f32) * 100.0
            } else {
                0.0
            };
            Ok(serde_json::json!({
                "word": word_str,
                "count": count,
                "percentage": percentage,
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;

    // Only parse fields that exist in API response - optional fields will use default from serde
    let mut content_style_json = serde_json::json!({
        "top_emojis": top_emojis,
        "top_words": top_words,
    });

    // Only add optional fields if they exist in the API response
    if let Some(avg_cast_length) = content_style_raw
        .get("avg_cast_length")
        .and_then(|v| v.as_f64())
    {
        content_style_json["avg_cast_length"] = serde_json::json!(avg_cast_length);
    }
    if let Some(total_characters) = content_style_raw
        .get("total_characters")
        .and_then(|v| v.as_u64())
    {
        content_style_json["total_characters"] = serde_json::json!(total_characters);
    }
    if let Some(frames_used) = content_style_raw
        .get("frames_used")
        .and_then(|v| v.as_u64())
    {
        content_style_json["frames_used"] = serde_json::json!(frames_used);
    }
    if let Some(frames_created) = content_style_raw
        .get("frames_created")
        .and_then(|v| v.as_u64())
    {
        content_style_json["frames_created"] = serde_json::json!(frames_created);
    }
    if let Some(channels_created) = content_style_raw
        .get("channels_created")
        .and_then(|v| v.as_u64())
    {
        content_style_json["channels_created"] = serde_json::json!(channels_created);
    }

    let content_style = content_style_json;
    let content_style = serde_json::from_value::<ContentStyleResponse>(content_style)
        .map_err(|e| format!("Failed to parse content_style: {}", e))?;

    // Parse domain_status from user data
    let domain_status = serde_json::json!({
        "has_ens": user.get("has_ens").and_then(|v| v.as_bool()).unwrap_or(false),
        "ens_name": user.get("ens_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        "has_farcaster_name": user.get("has_farcaster_name").and_then(|v| v.as_bool()).unwrap_or(false),
        "farcaster_name": user.get("farcaster_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        "username_type": None::<String>
    });
    let domain_status = serde_json::from_value::<DomainStatusResponse>(domain_status)
        .map_err(|e| format!("Failed to parse domain_status: {}", e))?;

    Ok(AnnualReportResponse {
        fid,
        username,
        display_name,
        year,
        engagement,
        temporal_activity,
        content_style,
        follower_growth,
        domain_status,
        network_comparison: None, // API doesn't return this yet
    })
}
