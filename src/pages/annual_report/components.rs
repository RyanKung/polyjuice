use yew::prelude::*;

/// ReportCard - 统一的报告卡片容器组件
/// 用于包装所有年度报告中的 section，提供统一的样式和布局
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

/// ReportCardContent - 统一的卡片内容容器
/// 提供统一的 padding、布局和防复制样式
#[derive(Properties, PartialEq, Clone)]
pub struct ReportCardContentProps {
    pub children: Children,
    #[prop_or_default]
    pub flex_direction: Option<String>,
    #[prop_or_default]
    pub padding: Option<String>,
    #[prop_or_default]
    pub overflow_y: Option<String>,
}

#[function_component]
pub fn ReportCardContent(props: &ReportCardContentProps) -> Html {
    let flex_direction = props.flex_direction.as_deref().unwrap_or("column");
    let padding = props.padding.as_deref().unwrap_or("100px 40px 40px 40px");
    let overflow_y = props.overflow_y.as_deref().unwrap_or("auto");
    
    html! {
        <div class="report-card-content" style={format!("
            width: 100%;
            height: calc(100% - 60px);
            display: flex;
            flex-direction: {};
            padding: {};
            box-sizing: border-box;
            overflow-y: {};
            user-select: none;
            -webkit-user-select: none;
            -moz-user-select: none;
            -ms-user-select: none;
            -webkit-user-drag: none;
            -khtml-user-drag: none;
            -moz-user-drag: none;
            -o-user-drag: none;
        ", flex_direction, padding, overflow_y)}
        oncopy={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        oncut={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        onpaste={Callback::from(|e: web_sys::Event| {
            e.prevent_default();
        })}
        ondragstart={Callback::from(|e: web_sys::DragEvent| {
            e.prevent_default();
        })}
        >
            {props.children.clone()}
        </div>
    }
}

/// SectionTitle - 统一的 section 标题样式
#[derive(Properties, PartialEq, Clone)]
pub struct SectionTitleProps {
    pub text: String,
    #[prop_or_default]
    pub font_size: Option<String>,
}

#[function_component]
pub fn SectionTitle(props: &SectionTitleProps) -> Html {
    let font_size = props.font_size.as_deref().unwrap_or("36px");
    
    html! {
        <h2 style={format!("
            font-size: {};
            font-weight: 700;
            margin: 0 0 32px 0;
            color: white;
            text-align: center;
        ", font_size)}>
            {props.text.clone()}
        </h2>
    }
}

/// StatCard - 统一的统计卡片样式
/// 用于显示各种统计数据（如 likes、casts 等）
#[derive(Properties, PartialEq, Clone)]
pub struct StatCardProps {
    pub label: String,
    pub value: String,
    pub children: Option<Children>,
}

#[function_component]
pub fn StatCard(props: &StatCardProps) -> Html {
    html! {
        <div style="
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            -webkit-backdrop-filter: blur(10px);
            border-radius: 16px;
            padding: 24px;
            border: 1px solid rgba(255, 255, 255, 0.2);
        ">
            <div style="font-size: 14px; color: rgba(255, 255, 255, 0.7); margin-bottom: 8px;">
                {props.label.clone()}
            </div>
            <div style="font-size: 32px; font-weight: 700; color: white;">
                {props.value.clone()}
                {if let Some(children) = &props.children {
                    html! { {children.clone()} }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}

