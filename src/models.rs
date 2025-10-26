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

// Search and API Response Structures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub profile: ProfileData,
    pub social: Option<SocialData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointWithPing {
    pub url: String,
    pub latency_ms: Option<f64>,
    pub status: String, // "online", "offline", "checking"
}
