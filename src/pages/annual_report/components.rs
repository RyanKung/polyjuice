use yew::prelude::*;

/// ReportCard - Unified report card container component
/// Used to wrap all sections in the annual report, providing unified styling and layout
#[derive(Properties, PartialEq, Clone)]
pub struct ReportCardProps {
    pub children: Children,
    #[prop_or(true)]
    pub with_padding_top: bool,
    #[prop_or(true)]
    pub is_own_report: bool,
}

#[function_component]
pub fn ReportCard(props: &ReportCardProps) -> Html {
    let padding_style = ""; // No padding-top to keep content flush with headers
    
    // If not own report, remove padding and border
    let card_style = if props.is_own_report {
        format!("
            flex: 0 0 100%;
            width: 100%;
            height: 100%;
            scroll-snap-align: start;
            scroll-snap-stop: always;
            {};
            box-sizing: border-box;
        ", padding_style)
    } else {
        format!("
            flex: 0 0 100%;
            width: 100%;
            height: 100%;
            scroll-snap-align: start;
            scroll-snap-stop: always;
            {};
            box-sizing: border-box;
            padding: 0;
            border: none;
        ", padding_style)
    };

    html! {
        <div class="annual-report-card" style={card_style}>
            {props.children.clone()}
        </div>
    }
}
