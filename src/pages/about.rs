use yew::prelude::*;

/// About page component
#[function_component]
pub fn AboutPage() -> Html {
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
            </div>
        </div>
    }
}
