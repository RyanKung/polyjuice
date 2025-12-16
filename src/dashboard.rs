use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::RequestMode;
use yew::prelude::*;

use crate::models::CastsStats;

/// Fetch casts stats from API
pub async fn fetch_casts_stats(
    api_url: &str,
    fid: i64,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
) -> Result<CastsStats, String> {
    let mut url = format!("{}/api/casts/stats/{}", api_url.trim_end_matches('/'), fid);

    // Add query parameters if provided
    let mut query_params = Vec::new();
    if let Some(start) = start_timestamp {
        query_params.push(format!("start_timestamp={}", start));
    }
    if let Some(end) = end_timestamp {
        query_params.push(format!("end_timestamp={}", end));
    }
    if !query_params.is_empty() {
        url.push('?');
        url.push_str(&query_params.join("&"));
    }

    let window = web_sys::window().ok_or("No window object")?;
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    let headers = request.headers();
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set Content-Type: {:?}", e))?;

    // Add authentication headers if configured
    crate::api::add_auth_headers(&headers, "GET", &url, None)
        .map_err(|e| format!("Failed to add auth headers: {}", e))?;

    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: web_sys::Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;

    let status = resp.status();
    if status != 200 {
        return Err(format!("API returned status: {}", status));
    }

    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("No text method: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to get text: {:?}", e))?;

    let body = text.as_string().unwrap_or_default();

    // Parse as ApiResponse first
    let api_response: crate::models::ApiResponse<serde_json::Value> =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse API response: {}", e))?;

    if !api_response.success {
        return Err(api_response
            .error
            .unwrap_or_else(|| "API request failed".to_string()));
    }

    // Extract data from response
    let data = api_response
        .data
        .ok_or_else(|| "No data in response".to_string())?;

    // Check if it's a pending job
    if let Some(status) = data.get("status") {
        if let Some(status_str) = status.as_str() {
            if status_str == "pending" || status_str == "processing" {
                let message = data
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Processing...");
                return Err(format!("Job {}: {}", status_str, message));
            }
        }
    }

    // Parse as CastsStatsResponse
    let response: crate::models::CastsStatsResponse = serde_json::from_value(data)
        .map_err(|e| format!("Failed to parse CastsStatsResponse: {}", e))?;

    // Convert date_distribution to daily_stats with timestamps
    let daily_stats = response
        .date_distribution
        .into_iter()
        .map(|dd| {
            // Parse date string to get timestamp
            let timestamp = parse_date_to_timestamp(&dd.date);
            crate::models::DailyCastStat {
                date: dd.date,
                count: dd.count,
                timestamp,
            }
        })
        .collect();

    // Combine top_nouns and top_verbs into word_cloud
    // Calculate total count for percentage calculation
    let total_word_count: usize = response.top_nouns.iter().map(|w| w.count).sum::<usize>()
        + response.top_verbs.iter().map(|w| w.count).sum::<usize>();

    let mut word_cloud = Vec::new();
    for word in response
        .top_nouns
        .into_iter()
        .chain(response.top_verbs.into_iter())
    {
        let percentage = if total_word_count > 0 {
            word.count as f32 / total_word_count as f32
        } else {
            0.0
        };
        word_cloud.push(crate::models::WordFrequency {
            word: word.word,
            count: word.count,
            percentage,
        });
    }

    // Sort by count descending
    word_cloud.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(crate::models::CastsStats {
        fid,
        daily_stats,
        total_casts: response.total_casts,
        word_cloud,
    })
}

/// Parse date string (YYYY-MM-DD) to Unix timestamp
fn parse_date_to_timestamp(date_str: &str) -> i64 {
    // Parse date string format: YYYY-MM-DD
    // Use same approach as get_current_year_timestamps() - convert to i32 first
    if date_str.len() >= 10 {
        if let (Ok(year_str), Ok(month_str), Ok(day_str)) = (
            date_str[0..4].parse::<i32>(),
            date_str[5..7].parse::<i32>(),
            date_str[8..10].parse::<i32>(),
        ) {
            // Month is 0-based in JavaScript Date (0 = January, 11 = December)
            let _month = (month_str - 1).max(0);

            // Create date using Date constructor string
            let date_str_js = format!("{}-{:02}-{:02}T00:00:00Z", year_str, month_str, day_str);
            let parsed_date = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(&date_str_js));

            return (parsed_date.get_time() / 1000.0) as i64;
        }
    }
    0
}

/// Get start and end timestamps for current calendar year
pub fn get_current_year_timestamps() -> (i64, i64) {
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();

    // Start of year: January 1, 00:00:00
    let start_date = js_sys::Date::new_with_year_month_day_hr_min_sec_milli(year, 0, 1, 0, 0, 0, 0);

    // End of year: December 31, 23:59:59
    let end_date =
        js_sys::Date::new_with_year_month_day_hr_min_sec_milli(year, 11, 31, 23, 59, 59, 999);

    let start_timestamp = (start_date.get_time() / 1000.0) as i64;
    let end_timestamp = (end_date.get_time() / 1000.0) as i64;

    (start_timestamp, end_timestamp)
}

#[derive(Properties, PartialEq, Clone)]
pub struct DashboardProps {
    pub fid: i64,
    pub api_url: String,
}

#[function_component]
pub fn Dashboard(props: &DashboardProps) -> Html {
    let stats = use_state(|| None::<CastsStats>);
    let is_loading = use_state(|| true);
    let error = use_state(|| None::<String>);

    {
        let stats = stats.clone();
        let is_loading = is_loading.clone();
        let error = error.clone();
        let api_url = props.api_url.clone();
        let fid = props.fid;

        use_effect_with((), move |_| {
            let stats = stats.clone();
            let is_loading = is_loading.clone();
            let error = error.clone();
            let api_url = api_url.clone();

            spawn_local(async move {
                is_loading.set(true);
                error.set(None);

                let (start_ts, end_ts) = get_current_year_timestamps();
                match fetch_casts_stats(&api_url, fid, Some(start_ts), Some(end_ts)).await {
                    Ok(data) => {
                        stats.set(Some(data));
                        is_loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        is_loading.set(false);
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="dashboard-container">
            <h2 class="dashboard-title">{"Activity Dashboard"}</h2>

            if *is_loading {
                <div class="loading-container">
                    <div class="skeleton-spinner"></div>
                    <p>{"Loading activity data..."}</p>
                </div>
            } else if let Some(err) = &*error {
                <div class="error-message">
                    <p>{format!("Error loading data: {}", err)}</p>
                </div>
            } else if let Some(stats_data) = &*stats {
                <>
                    // Daily activity chart (GitHub contribution style)
                    <div class="activity-chart-section">
                        <h3>{"Daily Activity"}</h3>
                        <ActivityChart daily_stats={stats_data.daily_stats.clone()} />
                    </div>

                    // Word cloud section
                    if !stats_data.word_cloud.is_empty() {
                        <div class="word-cloud-section">
                            <h3>{"Keywords"}</h3>
                            <WordCloud words={stats_data.word_cloud.clone()} />
                        </div>
                    }
                </>
            }
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct ActivityChartProps {
    daily_stats: Vec<crate::models::DailyCastStat>,
}

#[function_component]
fn ActivityChart(props: &ActivityChartProps) -> Html {
    // Group stats by month for better visualization
    // Calculate max count for color scaling
    let max_count = props.daily_stats.iter().map(|s| s.count).max().unwrap_or(1);

    // Get all days in current year
    let now = js_sys::Date::new_0();
    let year = now.get_full_year();
    let mut all_days = Vec::new();

    // Days in each month (non-leap year)
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    // Check if leap year
    let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

    // Start from January 1
    for month in 0..12 {
        let mut days_in_month = days_in_months[month];
        // Adjust for February in leap year
        if month == 1 && is_leap_year {
            days_in_month = 29;
        }

        for day in 1..=days_in_month {
            let date_str = format!("{:04}-{:02}-{:02}", year, month + 1, day);

            // Find matching stat
            let count = props
                .daily_stats
                .iter()
                .find(|s| s.date == date_str)
                .map(|s| s.count)
                .unwrap_or(0);

            all_days.push((date_str, count));
        }
    }

    // Calculate intensity level (0-4) for color
    let get_intensity = |count: usize| -> usize {
        if count == 0 {
            0
        } else if count <= max_count / 4 {
            1
        } else if count <= max_count / 2 {
            2
        } else if count <= max_count * 3 / 4 {
            3
        } else {
            4
        }
    };

    html! {
        <div class="activity-chart">
            <div class="chart-grid">
                {for all_days.iter().map(|(date_str, count)| {
                    let intensity = get_intensity(*count);
                    let intensity_class = format!("intensity-{}", intensity);
                    let tooltip = if *count > 0 {
                        format!("{}: {} cast{}", date_str, count, if *count > 1 { "s" } else { "" })
                    } else {
                        format!("{}: No casts", date_str)
                    };

                    html! {
                        <div
                            class={format!("chart-day {}", intensity_class)}
                            title={tooltip}
                        >
                        </div>
                    }
                })}
            </div>
            <div class="chart-legend">
                <span>{"Less"}</span>
                <div class="legend-colors">
                    <div class="legend-box intensity-0"></div>
                    <div class="legend-box intensity-1"></div>
                    <div class="legend-box intensity-2"></div>
                    <div class="legend-box intensity-3"></div>
                    <div class="legend-box intensity-4"></div>
                </div>
                <span>{"More"}</span>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct WordCloudProps {
    words: Vec<crate::models::WordFrequency>,
}

#[function_component]
fn WordCloud(props: &WordCloudProps) -> Html {
    // Sort words by frequency and take top 50
    let mut sorted_words = props.words.clone();
    sorted_words.sort_by(|a, b| b.count.cmp(&a.count));
    let top_words = sorted_words.into_iter().take(50).collect::<Vec<_>>();

    // Calculate max count for font size scaling
    let max_count = top_words.first().map(|w| w.count).unwrap_or(1);

    html! {
        <div class="word-cloud">
            {for top_words.iter().map(|word| {
                // Font size based on frequency (12px to 32px)
                let font_size = 12.0 + (word.count as f32 / max_count as f32) * 20.0;
                let opacity = 0.6 + (word.count as f32 / max_count as f32) * 0.4;

                html! {
                    <span
                        class="word-cloud-item"
                        style={format!(
                            "font-size: {}px; opacity: {:.2};",
                            font_size, opacity
                        )}
                        title={format!("{}: {} times", word.word, word.count)}
                    >
                        {&word.word}
                    </span>
                }
            })}
        </div>
    }
}
