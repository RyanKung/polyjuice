use web_sys::InputEvent;
use yew::prelude::*;

use crate::icons;
use crate::models::*;

// ============================================================================
// Chat View Component
// ============================================================================

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
                } else if props.is_chat_loading {
                    // Show loading state while creating session
                    <div class="chat-session-loading">
                        <div class="chat-loading-content">
                            <div class="chat-loading-icon">
                                <div class="chat-icon-pulse">
                                    {icons::chat()}
                                </div>
                            </div>
                            <div class="chat-loading-text">
                                <p class="loading-title">{"Creating chat session"}</p>
                                <p class="loading-subtitle">{"Preparing your conversation..."}</p>
                            </div>
                            <div class="chat-loading-dots">
                                <span></span>
                                <span></span>
                                <span></span>
                            </div>
                        </div>
                    </div>
                } else {
                    <div class="no-chat-session">
                        <div class="no-chat-icon">{"💬"}</div>
                        <p class="no-chat-title">{"No chat session available"}</p>
                        <p class="no-chat-subtitle">{"Please try searching again"}</p>
                    </div>
                }
            </div>
        </div>
    }
}

// ============================================================================
// Chat Header Component
// ============================================================================

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
                            {props.session.get_display_name_initial()}
                        </div>
                    }
                } else {
                    <div class="chat-avatar-placeholder">
                        {props.session.get_display_name_initial()}
                    </div>
                }
            </div>
            <div class="chat-user-details">
                <h3>{props.session.get_display_name()}</h3>
                <p>{"FID: "}{props.session.fid}</p>
            </div>
            <div class="chat-status">
                <div class="chat-status-dot"></div>
                {"Online"}
            </div>
        </div>
    }
}

// ============================================================================
// Chat Messages Component
// ============================================================================

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

// ============================================================================
// Chat Input Component
// ============================================================================

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
                    {icons::send()}
                </button>
            </div>
        </div>
    }
}
