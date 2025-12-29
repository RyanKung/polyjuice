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
        format!(
            "
            flex: 0 0 100%;
            width: 100%;
            height: 100%;
            scroll-snap-align: start;
            scroll-snap-stop: always;
            {};
            box-sizing: border-box;
        ",
            padding_style
        )
    } else {
        format!(
            "
            flex: 0 0 100%;
            width: 100%;
            height: 100%;
            scroll-snap-align: start;
            scroll-snap-stop: always;
            {};
            box-sizing: border-box;
            padding: 0;
            border: none;
        ",
            padding_style
        )
    };

    // Add style tag to override child padding when not own report
    // But preserve top padding for all pages to avoid header overlap
    let child_override_style = if !props.is_own_report {
        html! {
            <style>{r#"
                .annual-report-card > * {
                    padding: 0 !important;
                }
                .annual-report-card .report-card-content:not(.cover-page-content) {
                    padding-top: 100px !important;
                    padding-left: 40px !important;
                    padding-right: 40px !important;
                    padding-bottom: 40px !important;
                }
                .annual-report-card .cover-page-content {
                    padding: 80px 40px 40px 40px !important;
                }
            "#}</style>
        }
    } else {
        html! {}
    };

    html! {
        <>
            {child_override_style}
            <div class="annual-report-card" style={card_style}>
                {props.children.clone()}
            </div>
        </>
    }
}
