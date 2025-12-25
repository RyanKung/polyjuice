use yew::prelude::*;

/// Icon component for Font Awesome icons
#[derive(Properties, PartialEq, Clone)]
pub struct IconProps {
    pub name: String,
    #[prop_or_default]
    pub class: String,
    #[prop_or_default]
    pub style: String,
}

#[function_component]
pub fn Icon(props: &IconProps) -> Html {
    let class = if props.class.is_empty() {
        format!("fas fa-{}", props.name)
    } else {
        format!("fas fa-{} {}", props.name, props.class)
    };
    
    html! {
        <i class={class} style={props.style.clone()}></i>
    }
}

/// Helper function to render common icons
pub fn back_arrow() -> Html {
    html! { <Icon name="arrow-left" /> }
}

pub fn share() -> Html {
    // Modern share icon options (uncomment to try different styles):
    // - "arrow-up-from-bracket" - Modern iOS-style share (currently used)
    // - "share-nodes" - Network nodes share icon
    // - "share-alt" - Alternative share icon
    // - "share-square" - Share with square background
    // - "external-link-alt" - External link icon
    // - "share-from-square" - Share from square (newer)
    html! { <Icon name="arrow-up-from-bracket" /> }
}

pub fn user() -> Html {
    html! { <Icon name="user" /> }
}

pub fn search() -> Html {
    html! { <Icon name="search" /> }
}

pub fn info() -> Html {
    html! { <Icon name="info-circle" /> }
}

pub fn chat() -> Html {
    html! { <Icon name="comment" /> }
}

pub fn send() -> Html {
    html! { <Icon name="paper-plane" /> }
}

pub fn close() -> Html {
    html! { <Icon name="times" /> }
}

