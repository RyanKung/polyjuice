use serde::Deserialize;
use serde::Serialize;

// Profile and Social Data Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileData {
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub pfp_url: Option<String>,
    pub location: Option<String>,
    pub twitter_username: Option<String>,
    pub github_username: Option<String>,
}

impl ProfileData {
    /// Get a consistent display name for the user
    /// Priority: display_name -> username -> format!("User {}", fid) -> "Unknown User"
    pub fn get_display_name(&self) -> String {
        if let Some(ref display_name) = self.display_name {
            if !display_name.is_empty() {
                return display_name.clone();
            }
        }
        if let Some(ref username) = self.username {
            if !username.is_empty() {
                return format!("@{}", username);
            }
        }
        format!("User {}", self.fid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialData {
    pub fid: i64,
    pub following_count: usize,
    pub followers_count: usize,
    pub influence_score: f32,
    pub top_followed_users: Vec<UserMention>,
    pub top_followers: Vec<UserMention>,
    pub most_mentioned_users: Vec<UserMention>,
    pub social_circles: SocialCircles,
    pub interaction_style: InteractionStyle,
    pub word_cloud: WordCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SocialCircles {
    pub tech_builders: f32,
    pub content_creators: f32,
    pub web3_natives: f32,
    pub casual_users: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InteractionStyle {
    pub reply_frequency: f32,
    pub mention_frequency: f32,
    pub network_connector: bool,
    pub community_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMention {
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub count: usize,
    pub category: String,
}

impl UserMention {
    /// Get a consistent display name for the user
    /// Priority: display_name -> username -> format!("User {}", fid)
    pub fn get_display_name(&self) -> String {
        if let Some(ref display_name) = self.display_name {
            if !display_name.is_empty() {
                return display_name.clone();
            }
        }
        if let Some(ref username) = self.username {
            if !username.is_empty() {
                return format!("@{}", username);
            }
        }
        format!("User {}", self.fid)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordCloud {
    pub top_words: Vec<WordFrequency>,
    pub top_phrases: Vec<WordFrequency>,
    pub signature_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordFrequency {
    pub word: String,
    pub count: usize,
    pub percentage: f32,
}

// MBTI Personality Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MbtiProfile {
    pub fid: i64,
    pub mbti_type: String,          // e.g., "INTJ", "ENFP"
    pub confidence: f32,            // 0.0-1.0 confidence score
    pub dimensions: MbtiDimensions, // Individual dimension scores
    pub traits: Vec<String>,        // Key personality traits
    pub analysis: String,           // Detailed analysis
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MbtiDimensions {
    pub ei_score: f32,      // 0.0 = E (Extravert), 1.0 = I (Introvert)
    pub sn_score: f32,      // 0.0 = S (Sensing), 1.0 = N (Intuition)
    pub tf_score: f32,      // 0.0 = T (Thinking), 1.0 = F (Feeling)
    pub jp_score: f32,      // 0.0 = J (Judging), 1.0 = P (Perceiving)
    pub ei_confidence: f32, // Confidence for E/I dimension
    pub sn_confidence: f32,
    pub tf_confidence: f32,
    pub jp_confidence: f32,
}

// Search and API Response Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub profile: ProfileData,
    pub social: Option<SocialData>,
    pub mbti: Option<MbtiProfile>,
    pub pending_jobs: Option<Vec<PendingJob>>,
}

/// Pending job information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingJob {
    pub job_key: String,
    pub job_type: String,        // "social", "mbti", etc.
    pub status: Option<String>,  // "pending", "processing", "completed", "failed"
    pub started_at: Option<u64>, // Timestamp
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {}

/// Pending job response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PendingResponse {
    pub status: String,
    pub job_key: Option<String>,
    pub message: Option<String>,
}

/// Job status response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JobStatusResponse {
    pub job_key: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
}

// Chat-related Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatSession {
    pub session_id: String,
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub conversation_history: Vec<ChatMessage>,
    pub created_at: u64,
    pub last_activity: u64,
}

impl ChatSession {
    /// Get a consistent display name for the user
    /// Priority: display_name -> username -> format!("User {}", fid) -> "Unknown User"
    pub fn get_display_name(&self) -> String {
        if let Some(ref display_name) = self.display_name {
            if !display_name.is_empty() {
                return display_name.clone();
            }
        }
        if let Some(ref username) = self.username {
            if !username.is_empty() {
                return format!("@{}", username);
            }
        }
        format!("User {}", self.fid)
    }

    /// Get the first character of the display name for avatar placeholder
    pub fn get_display_name_initial(&self) -> String {
        let name = self.get_display_name();
        name.chars()
            .next()
            .unwrap_or('?')
            .to_uppercase()
            .collect::<String>()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatRequest {
    pub user: String,
    pub context_limit: usize,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChatResponse {
    pub session_id: String,
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub total_casts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageRequest {
    pub session_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageResponse {
    pub session_id: String,
    pub message: String,
    pub relevant_casts_count: usize,
    pub conversation_length: usize,
}

// Endpoint-related Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointData {
    pub endpoints: Vec<String>,
    pub contract_address: String,
    pub network: String,
}

// Casts Stats Structures - API Response format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CastsStatsResponse {
    #[serde(default)]
    pub total_casts: usize,
    #[serde(default)]
    pub date_distribution: Vec<DateDistribution>,
    #[serde(default)]
    pub date_range: Option<DateRange>,
    #[serde(default)]
    pub language_distribution: std::collections::HashMap<String, usize>,
    #[serde(default)]
    pub top_nouns: Vec<TopWord>,
    #[serde(default)]
    pub top_verbs: Vec<TopWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DateDistribution {
    pub count: usize,
    pub date: String, // Format: YYYY-MM-DD
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopWord {
    pub count: usize,
    pub language: String,
    pub word: String,
}

// Internal representation for UI
#[derive(Debug, Clone, PartialEq)]
pub struct CastsStats {
    pub fid: i64,
    pub daily_stats: Vec<DailyCastStat>,
    pub total_casts: usize,
    pub word_cloud: Vec<WordFrequency>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DailyCastStat {
    pub date: String, // Format: YYYY-MM-DD
    pub count: usize,
    pub timestamp: i64, // Unix timestamp for that day
}

// Annual Report Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnnualReportResponse {
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub year: i32,
    pub engagement: EngagementResponse,
    pub temporal_activity: TemporalActivityResponse,
    pub content_style: ContentStyleResponse,
    pub follower_growth: FollowerGrowthResponse,
    pub domain_status: DomainStatusResponse,
    pub network_comparison: Option<NetworkComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngagementResponse {
    pub reactions_received: usize,
    pub recasts_received: usize,
    pub replies_received: usize,
    pub total_engagement: usize,
    pub most_popular_cast: Option<PopularCast>,
    pub top_reactors: Vec<TopReactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PopularCast {
    pub message_hash: String,
    pub text: String,
    pub reactions: usize,
    pub recasts: usize,
    pub replies: usize,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TopReactor {
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub interaction_count: usize,
    #[serde(default)]
    pub pfp_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemporalActivityResponse {
    #[serde(default)]
    pub total_casts: usize,
    #[serde(default)]
    pub total_casts_in_year: Option<usize>,
    pub hourly_distribution: Vec<HourlyDistribution>,
    pub monthly_distribution: Vec<MonthlyDistribution>,
    pub most_active_hour: Option<i32>,
    pub most_active_month: Option<String>,
    pub first_cast: Option<CastInfo>,
    pub last_cast: Option<CastInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HourlyDistribution {
    pub hour: i32,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlyDistribution {
    pub month: String, // Format: YYYY-MM
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CastInfo {
    pub message_hash: String,
    pub text: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentStyleResponse {
    pub top_emojis: Vec<EmojiFrequency>,
    pub top_words: Vec<WordFrequency>,
    #[serde(default)]
    pub avg_cast_length: f32,
    #[serde(default)]
    pub total_characters: usize,
    #[serde(default)]
    pub frames_used: usize,
    #[serde(default)]
    pub frames_created: usize,
    #[serde(default)]
    pub channels_created: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmojiFrequency {
    pub emoji: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FollowerGrowthResponse {
    pub current_followers: usize,
    pub current_following: usize,
    pub followers_at_start: usize,
    pub following_at_start: usize,
    pub net_growth: i64,
    pub monthly_snapshots: Vec<MonthlySnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonthlySnapshot {
    pub month: String, // Format: YYYY-MM
    pub followers: usize,
    pub following: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DomainStatusResponse {
    pub has_ens: bool,
    pub ens_name: Option<String>,
    pub has_farcaster_name: bool,
    pub farcaster_name: Option<String>,
    pub username_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkComparison {
    pub avg_casts_per_user: f32,
    pub avg_reactions_per_user: f32,
    pub avg_followers_per_user: f32,
    pub total_active_users: usize,
    pub percentiles: Option<Percentiles>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Percentiles {
    pub casts: Option<PercentileData>,
    pub reactions: Option<PercentileData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PercentileData {
    pub p50: usize,
    pub p75: usize,
    pub p90: usize,
}

// Profile with registered_at for annual report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileWithRegistration {
    pub fid: i64,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub pfp_url: Option<String>,
    pub location: Option<String>,
    pub twitter_username: Option<String>,
    pub github_username: Option<String>,
    pub registered_at: Option<i64>,
    pub total_casts: Option<usize>,
    pub total_reactions: Option<usize>,
    pub total_links: Option<usize>,
}
