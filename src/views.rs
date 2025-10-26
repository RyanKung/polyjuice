use yew::prelude::*;

use crate::models::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ProfileViewProps {
    pub search_result: Option<SearchResult>,
}

/// Profile view component
#[function_component]
pub fn ProfileView(props: &ProfileViewProps) -> Html {
    if let Some(result) = &props.search_result {
        html! {
            <div class="card profile-card">
                <div class="card-content">
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
                        <SocialAnalysis social={social.clone()} />
                    }
                </div>
            </div>
        }
    } else {
        html! {}
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SocialAnalysisProps {
    pub social: SocialData,
}

/// Social analysis component
#[function_component]
fn SocialAnalysis(props: &SocialAnalysisProps) -> Html {
    html! {
        <div class="social-analysis">
            <div class="social-stats">
                <div class="stat-item">
                    <div class="stat-label">{"Following"}</div>
                    <div class="stat-value">{format!("{}", props.social.following_count)}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">{"Followers"}</div>
                    <div class="stat-value">{format!("{}", props.social.followers_count)}</div>
                </div>
                <div class="stat-item">
                    <div class="stat-label">{"Influence"}</div>
                    <div class="stat-value">{format!("{:.1}", props.social.influence_score)}</div>
                </div>
            </div>

            <div class="social-circles">
                <h4>{"Social Circles"}</h4>
                <div class="circle-item">
                    <span>{"Tech Builders"}</span>
                    <div class="composition-bar">
                        <div class="composition-fill" style={format!("width: {}%", props.social.social_circles.tech_builders * 100.0)}></div>
                    </div>
                </div>
                <div class="circle-item">
                    <span>{"Content Creators"}</span>
                    <div class="composition-bar">
                        <div class="composition-fill" style={format!("width: {}%", props.social.social_circles.content_creators * 100.0)}></div>
                    </div>
                </div>
                <div class="circle-item">
                    <span>{"Web3 Natives"}</span>
                    <div class="composition-bar">
                        <div class="composition-fill" style={format!("width: {}%", props.social.social_circles.web3_natives * 100.0)}></div>
                    </div>
                </div>
                <div class="circle-item">
                    <span>{"Casual Users"}</span>
                    <div class="composition-bar">
                        <div class="composition-fill" style={format!("width: {}%", props.social.social_circles.casual_users * 100.0)}></div>
                    </div>
                </div>
            </div>

            <div class="interaction-style">
                <h4>{"Interaction Style"}</h4>
                <div class="interaction-stats">
                    <div class="interaction-item">
                        <span class="interaction-label">{"Reply Frequency"}</span>
                        <div class="interaction-bar">
                            <div class="interaction-fill" style={format!("width: {}%", props.social.interaction_style.reply_frequency * 100.0)}></div>
                        </div>
                    </div>
                    <div class="interaction-item">
                        <span class="interaction-label">{"Mention Frequency"}</span>
                        <div class="interaction-bar">
                            <div class="interaction-fill" style={format!("width: {}%", props.social.interaction_style.mention_frequency * 100.0)}></div>
                        </div>
                    </div>
                    <div class="interaction-item">
                        <span class="interaction-label">{"Network Connector"}</span>
                        <span class="interaction-value">{if props.social.interaction_style.network_connector { "Yes" } else { "No" }}</span>
                    </div>
                    <div class="interaction-item">
                        <span class="interaction-label">{"Community Role"}</span>
                        <span class="interaction-value">{&props.social.interaction_style.community_role}</span>
                    </div>
                </div>
            </div>

            if !props.social.most_mentioned_users.is_empty() {
                <div class="mentioned-users">
                    <h4>{"Most Mentioned Users"}</h4>
                    <div class="mentioned-list">
                        {for props.social.most_mentioned_users.iter().take(5).map(|user| {
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

            if !props.social.word_cloud.top_words.is_empty() {
                <div class="word-cloud">
                    <h4>{"Top Words"}</h4>
                    <div class="word-tags">
                        {for props.social.word_cloud.top_words.iter().take(10).map(|word| {
                            html! {
                                <span class="word-tag" style={format!("font-size: {}px", (word.percentage * 20.0 + 12.0).max(10.0).min(18.0))}>
                                    {&word.word}
                                </span>
                            }
                        })}
                    </div>
                </div>
            }

            if !props.social.word_cloud.signature_words.is_empty() {
                <div class="signature-words">
                    <h4>{"Signature Words"}</h4>
                    <div class="signature-tags">
                        {for props.social.word_cloud.signature_words.iter().map(|word| {
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

#[derive(Properties, PartialEq, Clone)]
pub struct ChatViewProps {
    pub chat_session: Option<ChatSession>,
    pub chat_messages: Vec<ChatMessage>,
    pub chat_message: String,
    pub is_chat_loading: bool,
    pub chat_error: Option<String>,
    pub search_result: Option<SearchResult>,
    pub on_input_change: Callback<InputEvent>,
    pub on_keypress: Callback<web_sys::KeyboardEvent>,
    pub on_send_message: Callback<()>,
}

/// Chat view component
#[function_component]
pub fn ChatView(props: &ChatViewProps) -> Html {
    html! {
        <div class="card chat-card">
            <div class="card-content">
                if let Some(session) = &props.chat_session {
                    <ChatHeader session={session.clone()} search_result={props.search_result.clone()} />
                    <ChatMessages messages={props.chat_messages.clone()} is_loading={props.is_chat_loading} />
                    <ChatInput 
                        message={props.chat_message.clone()}
                        is_loading={props.is_chat_loading}
                        on_input_change={props.on_input_change.clone()}
                        on_keypress={props.on_keypress.clone()}
                        on_send_message={props.on_send_message.clone()}
                    />
                    if let Some(error) = &props.chat_error {
                        <div class="error-message">
                            <p>{error}</p>
                        </div>
                    }
                } else {
                    <div class="no-chat-session">
                        <p>{"No chat session available. Please try searching again."}</p>
                    </div>
                }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ChatHeaderProps {
    pub session: ChatSession,
    pub search_result: Option<SearchResult>,
}

/// Chat header component
#[function_component]
fn ChatHeader(props: &ChatHeaderProps) -> Html {
    html! {
        <div class="chat-user-info">
            <div class="chat-user-avatar">
                if let Some(result) = &props.search_result {
                    if let Some(pfp_url) = &result.profile.pfp_url {
                        <img src={pfp_url.clone()} alt="Profile" />
                    } else {
                        <div class="chat-avatar-placeholder">
                            {props.session.display_name.clone().unwrap_or_else(|| "Unknown".to_string()).chars().next().unwrap_or('?').to_uppercase().collect::<String>()}
                        </div>
                    }
                } else {
                    <div class="chat-avatar-placeholder">
                        {props.session.display_name.clone().unwrap_or_else(|| "Unknown".to_string()).chars().next().unwrap_or('?').to_uppercase().collect::<String>()}
                    </div>
                }
            </div>
            <div class="chat-user-details">
                <h3>{props.session.display_name.clone().unwrap_or_else(|| "Unknown".to_string())}</h3>
                <p>{"FID: "}{props.session.fid}</p>
            </div>
            <div class="chat-status">
                <div class="chat-status-dot"></div>
                {"Online"}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ChatMessagesProps {
    pub messages: Vec<ChatMessage>,
    pub is_loading: bool,
}

/// Chat messages component
#[function_component]
fn ChatMessages(props: &ChatMessagesProps) -> Html {
    html! {
        <div class="chat-messages">
            {for props.messages.iter().map(|message| {
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
            
            if props.is_loading {
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
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ChatInputProps {
    pub message: String,
    pub is_loading: bool,
    pub on_input_change: Callback<InputEvent>,
    pub on_keypress: Callback<web_sys::KeyboardEvent>,
    pub on_send_message: Callback<()>,
}

/// Chat input component
#[function_component]
fn ChatInput(props: &ChatInputProps) -> Html {
    html! {
        <div class="chat-input">
            <div class="chat-input-box">
                <input 
                    type="text" 
                    class="chat-input-field"
                    placeholder="Ask me anything"
                    value={props.message.clone()}
                    oninput={props.on_input_change.clone()}
                    onkeypress={props.on_keypress.clone()}
                />
                <button 
                    class="chat-send-button"
                    onclick={props.on_send_message.clone().reform(|_| ())}
                    disabled={props.is_loading}
                >
                    {"➤"}
                </button>
            </div>
        </div>
    }
}