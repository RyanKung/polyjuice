use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::KeyboardEvent;
use web_sys::InputEvent;
use wasm_bindgen::JsCast;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileData {
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    pfp_url: Option<String>,
    location: Option<String>,
    twitter_username: Option<String>,
    github_username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SocialData {
    fid: i64,
    following_count: usize,
    followers_count: usize,
    influence_score: f32,
    top_followed_users: Vec<UserMention>,
    top_followers: Vec<UserMention>,
    most_mentioned_users: Vec<UserMention>,
    social_circles: SocialCircles,
    interaction_style: InteractionStyle,
    word_cloud: WordCloud,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SocialCircles {
    tech_builders: f32,
    content_creators: f32,
    web3_natives: f32,
    casual_users: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InteractionStyle {
    reply_frequency: f32,
    mention_frequency: f32,
    network_connector: bool,
    community_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserMention {
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    count: usize,
    category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordCloud {
    top_words: Vec<WordFrequency>,
    top_phrases: Vec<WordFrequency>,
    signature_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WordFrequency {
    word: String,
    count: usize,
    percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    profile: ProfileData,
    social: Option<SocialData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatSession {
    session_id: String,
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    conversation_history: Vec<ChatMessage>,
    created_at: u64,
    last_activity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateChatRequest {
    user: String,
    context_limit: usize,
    temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateChatResponse {
    session_id: String,
    fid: i64,
    username: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
    total_casts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessageRequest {
    session_id: String,
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessageResponse {
    session_id: String,
    message: String,
    relevant_casts_count: usize,
    conversation_length: usize,
}

#[function_component]
fn App() -> Html {
    // State management
    let search_input = use_state(|| String::new());
    let search_result = use_state(|| None::<SearchResult>);
    let is_loading = use_state(|| false);
    let error_message = use_state(|| None::<String>);
    let api_url = use_state(|| "http://127.0.0.1:3000".to_string());
    
    // Chat state management
    let chat_session = use_state(|| None::<ChatSession>);
    let chat_message = use_state(|| String::new());
    let chat_messages = use_state(|| Vec::<ChatMessage>::new());
    let is_chat_loading = use_state(|| false);
    let chat_error = use_state(|| None::<String>);
    let current_view = use_state(|| "profile"); // "profile" or "chat"

    // Search handler
    let on_search = {
        let search_input = search_input.clone();
        let search_result = search_result.clone();
        let is_loading = is_loading.clone();
        let error_message = error_message.clone();
        let api_url = api_url.clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();

        Callback::from(move |_| {
            let input = (*search_input).clone();
            if input.trim().is_empty() {
                return;
            }

            let search_result = search_result.clone();
            let is_loading = is_loading.clone();
            let error_message = error_message.clone();
            let api_url = (*api_url).clone();
            let chat_session = chat_session.clone();
            let chat_messages = chat_messages.clone();
            let is_chat_loading = is_chat_loading.clone();
            let chat_error = chat_error.clone();

            spawn_local(async move {
                is_loading.set(true);
                error_message.set(None);

                // Determine if input is FID (numeric) or username (text)
                let trimmed_input = input.trim();
                let is_fid = trimmed_input.parse::<u64>().is_ok();
                
                // Handle username with @ prefix
                let search_query = if is_fid {
                    trimmed_input.to_string()
                } else {
                    // Remove @ prefix if present for username search
                    trimmed_input.trim_start_matches('@').to_string()
                };

                // Make API requests to get both profile and social data
                let request_init = web_sys::RequestInit::new();
                request_init.set_method("GET");
                let headers = web_sys::Headers::new().unwrap();
                headers.set("Content-Type", "application/json").unwrap();
                request_init.set_headers(&headers);
                
                // Choose API endpoint based on input type
                let profile_url = if is_fid {
                    format!("{}/api/profiles/{}", api_url, search_query)
                } else {
                    format!("{}/api/profiles/username/{}", api_url, search_query)
                };
                
                // First, get profile data
                let profile_request = web_sys::Request::new_with_str_and_init(
                    &profile_url,
                    &request_init
                ).unwrap();

                let profile_response = wasm_bindgen_futures::JsFuture::from(
                    web_sys::window()
                        .unwrap()
                        .fetch_with_request(&profile_request)
                ).await;

                let profile_data = match profile_response {
                    Ok(response) => {
                        let response: web_sys::Response = response.dyn_into().unwrap();
                        if response.ok() {
                            match wasm_bindgen_futures::JsFuture::from(response.json().unwrap()).await {
                                Ok(result) => {
                                    let api_response: ApiResponse<ProfileData> = serde_wasm_bindgen::from_value(result).unwrap_or_else(|_| ApiResponse::error("Failed to parse API response".to_string()));
                                    
                                    if api_response.success && api_response.data.is_some() {
                                        Some(api_response.data.unwrap())
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                };

                // Choose social API endpoint based on input type
                let social_url = if is_fid {
                    format!("{}/api/social/{}", api_url, search_query)
                } else {
                    format!("{}/api/social/username/{}", api_url, search_query)
                };
                
                // Then, get social data
                let social_request = web_sys::Request::new_with_str_and_init(
                    &social_url,
                    &request_init
                ).unwrap();

                let social_response = wasm_bindgen_futures::JsFuture::from(
                    web_sys::window()
                        .unwrap()
                        .fetch_with_request(&social_request)
                ).await;

                let social_data = match social_response {
                    Ok(response) => {
                        let response: web_sys::Response = response.dyn_into().unwrap();
                        if response.ok() {
                            match wasm_bindgen_futures::JsFuture::from(response.json().unwrap()).await {
                                Ok(result) => {
                                    let api_response: ApiResponse<SocialData> = serde_wasm_bindgen::from_value(result).unwrap_or_else(|_| ApiResponse::error("Failed to parse API response".to_string()));
                                    
                                    if api_response.success && api_response.data.is_some() {
                                        Some(api_response.data.unwrap())
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                };

                // Combine the results
                if let Some(profile) = profile_data {
                    search_result.set(Some(SearchResult {
                        profile,
                        social: social_data,
                    }));
                    error_message.set(None);

                    // Auto-create chat session after successful search
                    let chat_session = chat_session.clone();
                    let chat_messages = chat_messages.clone();
                    let is_chat_loading = is_chat_loading.clone();
                    let chat_error = chat_error.clone();
                    let api_url = api_url.clone();

                    spawn_local(async move {
                        is_chat_loading.set(true);
                        chat_error.set(None);

                        // Create chat session
                        let request = CreateChatRequest {
                            user: if is_fid {
                                search_query.clone()
                            } else {
                                format!("@{}", search_query)
                            },
                            context_limit: 20,
                            temperature: 0.7,
                        };

                        let request_json = serde_json::to_string(&request).unwrap();
                        let request_init = web_sys::RequestInit::new();
                        request_init.set_method("POST");
                        let headers = web_sys::Headers::new().unwrap();
                        headers.set("Content-Type", "application/json").unwrap();
                        request_init.set_headers(&headers);
                        request_init.set_body(&request_json.into());

                        let chat_request = web_sys::Request::new_with_str_and_init(
                            &format!("{}/api/chat/create", api_url),
                            &request_init
                        ).unwrap();

                        match wasm_bindgen_futures::JsFuture::from(
                            web_sys::window()
                                .unwrap()
                                .fetch_with_request(&chat_request)
                        ).await
                        {
                            Ok(response) => {
                                let response: web_sys::Response = response.dyn_into().unwrap();
                                if response.ok() {
                                    match wasm_bindgen_futures::JsFuture::from(response.json().unwrap()).await {
                                        Ok(result) => {
                                            let api_response: ApiResponse<CreateChatResponse> = serde_wasm_bindgen::from_value(result).unwrap_or_else(|_| ApiResponse::error("Failed to parse API response".to_string()));
                                            
                                            if api_response.success && api_response.data.is_some() {
                                                let chat_data = api_response.data.unwrap();
                                                let session = ChatSession {
                                                    session_id: chat_data.session_id,
                                                    fid: chat_data.fid,
                                                    username: chat_data.username,
                                                    display_name: chat_data.display_name,
                                                    conversation_history: Vec::new(),
                                                    created_at: 0,
                                                    last_activity: 0,
                                                };
                                                chat_session.set(Some(session));
                                                chat_messages.set(Vec::new());
                                                chat_error.set(None);
                } else {
                                                chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                            }
                                        }
                                        Err(e) => {
                                            chat_error.set(Some(format!("Failed to parse response: {:?}", e)));
                                        }
                                    }
                                } else {
                                    chat_error.set(Some("Failed to create chat session".to_string()));
                        }
                    }
                    Err(e) => {
                                chat_error.set(Some(format!("Network error: {:?}", e)));
                            }
                    }

                        is_chat_loading.set(false);
                    });
                } else {
                    error_message.set(Some("User not found or API error".to_string()));
                }

                is_loading.set(false);
            });
        })
    };

    // Enter key handler for search
    let on_keypress = {
        let on_search = on_search.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                on_search.emit(());
            }
        })
    };

    // Back to search handler
    let on_back_to_search = {
        let search_result = search_result.clone();
        let search_input = search_input.clone();
        let error_message = error_message.clone();
        Callback::from(move |_: ()| {
            search_result.set(None);
            search_input.set(String::new());
            error_message.set(None);
        })
    };

    // Popular FID handler
    let on_popular_fid = {
        let search_input = search_input.clone();
        let on_search = on_search.clone();
        Callback::from(move |fid: String| {
            search_input.set(fid.clone());
            on_search.emit(());
        })
    };

    // View switching handler
    let on_switch_to_chat = {
        let current_view = current_view.clone();
        Callback::from(move |_| {
            current_view.set("chat");
        })
    };

    let on_switch_to_profile = {
        let current_view = current_view.clone();
        Callback::from(move |_: ()| {
            current_view.set("profile");
        })
    };

    // Smart back button handler
    let on_smart_back = {
        let current_view = current_view.clone();
        let search_result = search_result.clone();
        let search_input = search_input.clone();
        let error_message = error_message.clone();
        let chat_session = chat_session.clone();
        let chat_messages = chat_messages.clone();
        Callback::from(move |_| {
            match (*current_view).clone() {
                "profile" => {
                    // From profile back to search
                    search_result.set(None);
                    search_input.set(String::new());
                    error_message.set(None);
                    chat_session.set(None);
                    chat_messages.set(Vec::new());
                },
                "chat" => {
                    // From chat back to profile
                    current_view.set("profile");
                },
                _ => {
                    // Default: back to search
                    search_result.set(None);
                    search_input.set(String::new());
                    error_message.set(None);
                    chat_session.set(None);
                    chat_messages.set(Vec::new());
                }
            }
        })
    };

    // Chat message handler
    let on_send_chat_message = {
        let chat_session = chat_session.clone();
        let chat_message = chat_message.clone();
        let chat_messages = chat_messages.clone();
        let is_chat_loading = is_chat_loading.clone();
        let chat_error = chat_error.clone();
        let api_url = api_url.clone();

        Callback::from(move |_| {
            let message = (*chat_message).clone();
            if message.trim().is_empty() {
                return;
            }

            if let Some(session) = (*chat_session).clone() {
                let chat_session = chat_session.clone();
                let chat_message = chat_message.clone();
                let chat_messages = chat_messages.clone();
                let is_chat_loading = is_chat_loading.clone();
                let chat_error = chat_error.clone();
            let api_url = (*api_url).clone();

            spawn_local(async move {
                    is_chat_loading.set(true);
                    chat_error.set(None);

                    // Add user message to chat
                    let user_message = ChatMessage {
                        role: "user".to_string(),
                        content: message.clone(),
                        timestamp: 0,
                    };
                    let mut messages = (*chat_messages).clone();
                    messages.push(user_message);
                    chat_messages.set(messages);

                    // Send message to API
                    let request = ChatMessageRequest {
                        session_id: session.session_id.clone(),
                        message: message.clone(),
                    };

                    let request_json = serde_json::to_string(&request).unwrap();
                    let request_init = web_sys::RequestInit::new();
                    request_init.set_method("POST");
                    let headers = web_sys::Headers::new().unwrap();
                    headers.set("Content-Type", "application/json").unwrap();
                    request_init.set_headers(&headers);
                    request_init.set_body(&request_json.into());

                    let chat_request = web_sys::Request::new_with_str_and_init(
                        &format!("{}/api/chat/message", api_url),
                        &request_init
                    ).unwrap();

                    match wasm_bindgen_futures::JsFuture::from(
                        web_sys::window()
                            .unwrap()
                            .fetch_with_request(&chat_request)
                    ).await
                    {
                        Ok(response) => {
                            let response: web_sys::Response = response.dyn_into().unwrap();
                            if response.ok() {
                                match wasm_bindgen_futures::JsFuture::from(response.json().unwrap()).await {
                                    Ok(result) => {
                                        let api_response: ApiResponse<ChatMessageResponse> = serde_wasm_bindgen::from_value(result).unwrap_or_else(|_| ApiResponse::error("Failed to parse API response".to_string()));
                                        
                                        if api_response.success && api_response.data.is_some() {
                                            let chat_data = api_response.data.unwrap();
                                            let assistant_message = ChatMessage {
                                                role: "assistant".to_string(),
                                                content: chat_data.message,
                                                timestamp: 0,
                                            };
                                            let mut messages = (*chat_messages).clone();
                                            messages.push(assistant_message);
                                            chat_messages.set(messages);
                                            chat_error.set(None);
                                        } else {
                                            chat_error.set(Some(api_response.error.unwrap_or_else(|| "Unknown error".to_string())));
                                        }
                                    }
                                    Err(e) => {
                                        chat_error.set(Some(format!("Failed to parse response: {:?}", e)));
                                    }
                                }
                            } else {
                                chat_error.set(Some("Failed to send message".to_string()));
                            }
                        }
                        Err(e) => {
                            chat_error.set(Some(format!("Network error: {:?}", e)));
                        }
                    }

                    chat_message.set(String::new());
                    is_chat_loading.set(false);
                });
            }
        })
    };

    html! {
        <div class="app-container">
            // Search Page (only show if no search results)
            if (*search_result).is_none() {
                <div class="search-page">
                    <div class="search-header">
                        <div class="logo">
                            // Logo Image
                            <div class="logo-image">
                                <img src="/logo.png" alt="Polyjuice Logo" />
                            </div>
                            <h1>{"polyjuice"}</h1>
                            <p class="tagline">{"Discover & Chat with Farcaster Users"}</p>
                        </div>
                    </div>

                    <div class="search-content">

                        <div class="search-box">
                            <input 
                                type="text" 
                                class="search-input"
                                placeholder="give me a fid/username"
                                value={(*search_input).clone()}
                                oninput={
                                    let search_input = search_input.clone();
                                    Callback::from(move |e: InputEvent| {
                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                            search_input.set(input.value());
                                        }
                                    })
                                }
                                onkeypress={on_keypress}
                            />
                            <button 
                                class="search-button"
                                onclick={on_search.reform(|_| ())}
                                disabled={*is_loading}
                            >
                                {
                                    if *is_loading {
                                        html! { "Searching..." }
                                    } else {
                                        html! { "🔍" }
                                    }
                                }
                            </button>
                        </div>

                        // Mobile-only search button
                        <div class="mobile-search-button">
                            <button 
                                class="mobile-search-btn"
                                onclick={on_search.reform(|_| ())}
                                disabled={*is_loading}
                            >
                                {
                                    if *is_loading {
                                        html! { "Searching..." }
                                    } else {
                                        html! { "Search" }
                                    }
                                }
                            </button>
                        </div>

                        <div class="search-suggestions">
                            <p class="suggestions-title">{"Popular FIDs:"}</p>
                            <div class="suggestion-tags">
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "1".to_string())}>{"1"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "2".to_string())}>{"2"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "99".to_string())}>{"99"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "100".to_string())}>{"100"}</button>
                                <button class="suggestion-tag" onclick={on_popular_fid.clone().reform(|_| "1000".to_string())}>{"1000"}</button>
                            </div>
                        </div>

                        if let Some(error) = (*error_message).clone() {
                            <div class="error-message">
                                <p>{error}</p>
                                        </div>
                        }
                    </div>
                </div>
            }

            // Results Page (Profile + Chat cards)
            if (*search_result).is_some() {
                <div class="results-page">
                    // Smart Back Button
                    <div class="back-to-search">
                        <button class="back-button" onclick={on_smart_back}>
                            {"←"}
                        </button>
                    </div>
                    
                    // Profile Card (only show if current_view is "profile")
                    if (*current_view).clone() == "profile" {
                        <div class="card profile-card">
                            <div class="card-content">
                                if let Some(result) = (*search_result).clone() {
                                <div class="profile-info">
                                    <div class="profile-picture">
                                        if let Some(pfp_url) = &result.profile.pfp_url {
                                            <img src={pfp_url.clone()} alt="Profile" />
                                        } else {
                                            <div class="profile-picture-placeholder">
                                                {"👤"}
                                            </div>
                                        }
                                    </div>

                                    <div class="user-details">
                                        <h2>{result.profile.display_name.clone().unwrap_or_else(|| "Unknown".to_string())}</h2>
                                        if let Some(username) = &result.profile.username {
                                            <p class="username">{"@"}{username}</p>
                                        }
                                        <div class="fid-badge">{"FID: "}{result.profile.fid}</div>
                                        
                                        if let Some(bio) = &result.profile.bio {
                                            <p class="bio">{bio}</p>
                                        }
                                    </div>
                    </div>

                                if let Some(social) = &result.social {
                                    <div class="social-analysis">
                                        <div class="social-stats">
                                            <div class="stat-item">
                                                <div class="stat-label">{"Following"}</div>
                                                <div class="stat-value">{format!("{}", social.following_count)}</div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">{"Followers"}</div>
                                                <div class="stat-value">{format!("{}", social.followers_count)}</div>
                                            </div>
                                            <div class="stat-item">
                                                <div class="stat-label">{"Influence"}</div>
                                                <div class="stat-value">{format!("{:.1}", social.influence_score)}</div>
                                            </div>
                                        </div>

                                        <div class="social-circles">
                                            <h4>{"Social Circles"}</h4>
                                            <div class="circle-item">
                                                <span>{"Tech Builders"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.tech_builders * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Content Creators"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.content_creators * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Web3 Natives"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.web3_natives * 100.0)}></div>
                                                </div>
                                            </div>
                                            <div class="circle-item">
                                                <span>{"Casual Users"}</span>
                                                <div class="composition-bar">
                                                    <div class="composition-fill" style={format!("width: {}%", social.social_circles.casual_users * 100.0)}></div>
                                                </div>
                                            </div>
                                        </div>

                                        // Interaction Style
                                        <div class="interaction-style">
                                            <h4>{"Interaction Style"}</h4>
                                            <div class="interaction-stats">
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Reply Frequency"}</span>
                                                    <div class="interaction-bar">
                                                        <div class="interaction-fill" style={format!("width: {}%", social.interaction_style.reply_frequency * 100.0)}></div>
                                                    </div>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Mention Frequency"}</span>
                                                    <div class="interaction-bar">
                                                        <div class="interaction-fill" style={format!("width: {}%", social.interaction_style.mention_frequency * 100.0)}></div>
                                                    </div>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Network Connector"}</span>
                                                    <span class="interaction-value">{if social.interaction_style.network_connector { "Yes" } else { "No" }}</span>
                                                </div>
                                                <div class="interaction-item">
                                                    <span class="interaction-label">{"Community Role"}</span>
                                                    <span class="interaction-value">{&social.interaction_style.community_role}</span>
                                                </div>
                                            </div>
                                        </div>

                                        // Most Mentioned Users
                                        if !social.most_mentioned_users.is_empty() {
                                            <div class="mentioned-users">
                                                <h4>{"Most Mentioned Users"}</h4>
                                                <div class="mentioned-list">
                                                    {for social.most_mentioned_users.iter().take(5).map(|user| {
                                                        html! {
                                                            <div class="mentioned-item">
                                                                <span class="mentioned-name">
                                                                    {user.display_name.clone().unwrap_or_else(|| user.username.clone().unwrap_or_else(|| format!("FID {}", user.fid)))}
                                                                </span>
                                                                <span class="mentioned-count">{format!("{} mentions", user.count)}</span>
                                                                <span class="mentioned-category">{&user.category}</span>
                                                            </div>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }

                                        // Word Cloud
                                        if !social.word_cloud.top_words.is_empty() {
                                            <div class="word-cloud">
                                                <h4>{"Top Words"}</h4>
                                                <div class="word-tags">
                                                    {for social.word_cloud.top_words.iter().take(10).map(|word| {
                                                        html! {
                                                            <span class="word-tag" style={format!("font-size: {}px", (word.percentage * 20.0 + 12.0).max(10.0).min(18.0))}>
                                                                {&word.word}
                                                            </span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }

                                        // Signature Words
                                        if !social.word_cloud.signature_words.is_empty() {
                                            <div class="signature-words">
                                                <h4>{"Signature Words"}</h4>
                                                <div class="signature-tags">
                                                    {for social.word_cloud.signature_words.iter().map(|word| {
                                                        html! {
                                                            <span class="signature-tag">{word}</span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }
                                    </div>
                                }
                            }
                            </div>
                        </div>
                    }

                    // Chat Card (only show if current_view is "chat" and we have chat session)
                    if (*current_view).clone() == "chat" && (*chat_session).is_some() {
                        <div class="card chat-card">
                            <div class="card-content">
                                if let Some(session) = (*chat_session).clone() {
                                    <div class="chat-user-info">
                                        <div class="chat-user-avatar">
                                            {session.display_name.clone().unwrap_or_else(|| "Unknown".to_string()).chars().next().unwrap_or('?').to_uppercase().collect::<String>()}
                                        </div>
                                        <div class="chat-user-details">
                                            <h3>{session.display_name.clone().unwrap_or_else(|| "Unknown".to_string())}</h3>
                                            <p>{"FID: "}{session.fid}</p>
                                        </div>
                                        <div class="chat-status">
                                            <div class="chat-status-dot"></div>
                                            {"Online"}
                                        </div>
                                    </div>

                                    <div class="chat-messages">
                                        {for (*chat_messages).iter().map(|message| {
                                                html! {
                                                <div class={if message.role == "user" { "user-message" } else { "assistant-message" }}>
                                                    <div class="message-content">
                                                        {&message.content}
                                                    </div>
                                                    <div class="message-time">
                                                        {"Now"}
                                                    </div>
                                                </div>
                                            }
                                        })}
                                        
                                        if *is_chat_loading {
                                            <div class="assistant-message">
                                                <div class="message-content loading">
                                                    <div class="typing-indicator">
                                                        <span></span>
                                                        <span></span>
                                                        <span></span>
                                                    </div>
                                                    {"AI is thinking..."}
                                                </div>
                                            </div>
                                        }
                                    </div>

                                    <div class="chat-input">
                                        <div class="chat-input-box">
                                            <input 
                                                type="text" 
                                                class="chat-input-field"
                                                placeholder="Ask me anything about this user..."
                                                value={(*chat_message).clone()}
                                                            oninput={
                                                    let chat_message = chat_message.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                        if let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                                                            chat_message.set(input.value());
                                                        }
                                                    })
                                                }
                                                onkeypress={
                                                    let on_send_chat_message = on_send_chat_message.clone();
                                                    Callback::from(move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            on_send_chat_message.emit(());
                                                        }
                                                    })
                                                }
                                            />
                                            <button 
                                                class="chat-send-button"
                                                onclick={on_send_chat_message.reform(|_| ())}
                                                disabled={*is_chat_loading}
                                            >
                                                {"➤"}
                                            </button>
                                        </div>
                                    </div>

                                    if let Some(error) = (*chat_error).clone() {
                                        <div class="error-message">
                                            <p>{error}</p>
                                                                </div>
                                                            }
                                                        } else {
                                    <div class="no-chat-session">
                                        <p>{"No chat session available"}</p>
                                                                                </div>
                                                                    }
                                                                </div>
                                                </div>
                    }
                </div>

                // Floating Chat Button (only show on results page when profile is visible)
                if (*search_result).is_some() && (*current_view).clone() == "profile" && (*chat_session).is_some() {
                    <div class="floating-chat-button" onclick={on_switch_to_chat}>
                        {"💬"}
                    </div>
                }
            }
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}