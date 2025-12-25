use yew::prelude::*;

/// ReportCard - Unified report card container component
/// Used to wrap all sections in the annual report, providing unified styling and layout
#[derive(Properties, PartialEq, Clone)]
pub struct ReportCardProps {
    pub children: Children,
    #[prop_or(true)]
    pub with_padding_top: bool,
}

#[function_component]
pub fn ReportCard(props: &ReportCardProps) -> Html {
    let padding_style = ""; // No padding-top to keep content flush with headers

    html! {
        <div class="annual-report-card" style={format!("
            flex: 0 0 100%;
            width: 100%;
            height: 100%;
            scroll-snap-align: start;
            scroll-snap-stop: always;
            {};
            box-sizing: border-box;
        ", padding_style)}>
            {props.children.clone()}
        </div>
    }
}
