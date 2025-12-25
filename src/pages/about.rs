use yew::prelude::*;

use crate::views::EndpointView;

#[derive(Properties, PartialEq, Clone)]
pub struct AboutPageProps {
    pub endpoint_data: Option<crate::models::EndpointData>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub ping_results: Vec<(String, Option<f64>)>,
    pub selected_endpoint: Option<String>,
    pub on_select_endpoint: Callback<String>,
    pub custom_endpoints: Vec<String>,
    pub custom_url_input: String,
    pub on_custom_url_input_change: Callback<web_sys::InputEvent>,
    pub on_add_custom_endpoint: Callback<()>,
    pub custom_endpoint_error: Option<String>,
    pub is_adding_endpoint: bool,
    pub on_fetch_endpoints: Callback<()>,
}

/// About page component
#[function_component]
pub fn AboutPage(props: &AboutPageProps) -> Html {
    // Fetch endpoints when component mounts if not already loaded
    {
        let endpoint_data = props.endpoint_data.clone();
        let is_loading = props.is_loading;
        let on_fetch_endpoints = props.on_fetch_endpoints.clone();
        use_effect_with((), move |_| {
            if endpoint_data.is_none() && !is_loading {
                on_fetch_endpoints.emit(());
            }
            || ()
        });
    }

    html! {
        <div class="about-page">
            <div class="about-page-content">
                <div class="about-header">
                    <div class="logo-image">
                        <img src="/logo.png" alt="Polyjuice Logo" />
                    </div>
                    <h1>{"polyjuice"}</h1>
                    <p class="tagline">{"Discover & Chat with Farcaster Users"}</p>
                </div>

                <div class="about-section">
                    <h2>{"About"}</h2>
                    <p>{"Polyjuice is a powerful tool for discovering and interacting with Farcaster users. Search for users by FID or username, view their profiles, and chat with them using AI-powered conversations."}</p>
                </div>

                <div class="about-section">
                    <h2>{"Features"}</h2>
                    <ul class="about-features">
                        <li>{"🔍 Search Farcaster users by FID or username"}</li>
                        <li>{"👤 View detailed user profiles and analytics"}</li>
                        <li>{"💬 Chat with users using AI"}</li>
                        <li>{"📊 MBTI personality analysis"}</li>
                        <li>{"🌐 Social network insights"}</li>
                    </ul>
                </div>

                <div class="about-section">
                    <h2>{"Built With"}</h2>
                    <ul class="about-tech">
                        <li>{"Rust + WebAssembly"}</li>
                        <li>{"Yew Framework"}</li>
                        <li>{"Farcaster Mini App SDK"}</li>
                        <li>{"EIP-1193 & EIP-6963"}</li>
                    </ul>
                </div>

                <div class="about-section" style="margin-top: 40px;">
                    <EndpointView
                        endpoint_data={props.endpoint_data.clone()}
                        is_loading={props.is_loading}
                        error={props.error.clone()}
                        ping_results={props.ping_results.clone()}
                        selected_endpoint={props.selected_endpoint.clone()}
                        on_select_endpoint={props.on_select_endpoint.clone()}
                        custom_endpoints={props.custom_endpoints.clone()}
                        custom_url_input={props.custom_url_input.clone()}
                        on_custom_url_input_change={props.on_custom_url_input_change.clone()}
                        on_add_custom_endpoint={props.on_add_custom_endpoint.clone()}
                        custom_endpoint_error={props.custom_endpoint_error.clone()}
                        is_adding_endpoint={props.is_adding_endpoint}
                    />
                </div>
            </div>
        </div>
    }
}
