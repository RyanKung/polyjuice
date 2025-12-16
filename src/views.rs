use web_sys::InputEvent;
use yew::prelude::*;

use crate::models::*;

// MBTI type descriptions
const MBTI_DESCRIPTIONS: &[(&str, &str)] = &[
    ("INTJ", "Architect - Strategic, analytical, and independent"),
    ("INTP", "Logician - Innovative, curious, and intellectual"),
    ("ENTJ", "Commander - Bold, imaginative, and strong-willed"),
    ("ENTP", "Debater - Smart, curious, and quick-thinking"),
    ("INFJ", "Advocate - Idealistic, insightful, and principled"),
    ("INFP", "Mediator - Poetic, kind, and altruistic"),
    (
        "ENFJ",
        "Protagonist - Charismatic, inspiring, and natural leaders",
    ),
    ("ENFP", "Campaigner - Enthusiastic, creative, and sociable"),
    ("ISTJ", "Logistician - Practical, fact-minded, and reliable"),
    ("ISFJ", "Defender - Warm, dedicated, and responsible"),
    (
        "ESTJ",
        "Executive - Organized, traditional, and administrators",
    ),
    ("ESFJ", "Consul - Caring, social, and popular"),
    ("ISTP", "Virtuoso - Bold, practical, and experimental"),
    ("ISFP", "Adventurer - Flexible, charming, and artistic"),
    ("ESTP", "Entrepreneur - Smart, energetic, and perceptive"),
    ("ESFP", "Entertainer - Spontaneous, enthusiastic, and fun"),
];


#[derive(Properties, PartialEq, Clone)]
pub struct MbtiAnalysisProps {
    pub mbti: MbtiProfile,
}

#[derive(Properties, PartialEq, Clone)]
pub struct MbtiRadarChartProps {
    pub ei_score: f32,
    pub sn_score: f32,
    pub tf_score: f32,
    pub jp_score: f32,
}

/// MBTI Radar Chart Component
/// 8 dimensions: E, I, S, N, T, F, J, P
/// E and I are opposite, S and N are opposite, T and F are opposite, J and P are opposite
#[function_component]
fn MbtiRadarChart(props: &MbtiRadarChartProps) -> Html {
    // Radar chart configuration
    let size = 280.0;
    let center = size / 2.0;
    let radius = 100.0;

    // Convert scores to individual dimension values
    // Score definition: 0.0 = first letter, 1.0 = second letter
    // ei_score: 0.0 = E, 1.0 = I
    // sn_score: 0.0 = S, 1.0 = N
    // tf_score: 0.0 = T, 1.0 = F
    // jp_score: 0.0 = J, 1.0 = P
    // So if ei_score = 0.4, then E = 0.6 (60%), I = 0.4 (40%)
    let e_value = 1.0 - props.ei_score; // E is opposite of I
    let i_value = props.ei_score; // I is the score itself
    let s_value = 1.0 - props.sn_score; // S is opposite of N
    let n_value = props.sn_score; // N is the score itself
    let t_value = 1.0 - props.tf_score; // T is opposite of F
    let f_value = props.tf_score; // F is the score itself
    let j_value = 1.0 - props.jp_score; // J is opposite of P
    let p_value = props.jp_score; // P is the score itself

    // Convert to radius distances
    let e_radius = e_value * radius;
    let i_radius = i_value * radius;
    let s_radius = s_value * radius;
    let n_radius = n_value * radius;
    let t_radius = t_value * radius;
    let f_radius = f_value * radius;
    let j_radius = j_value * radius;
    let p_radius = p_value * radius;

    // Calculate positions for 8 dimensions evenly spaced (45 degrees apart)
    // Layout: E (top), S (top-right), T (right), J (bottom-right), I (bottom), N (bottom-left), F (left), P (top-left)
    // Angles: 0°, 45°, 90°, 135°, 180°, 225°, 270°, 315°
    use std::f32::consts::PI;

    let angle_to_coords = |angle_deg: f32, r: f32| -> (f32, f32) {
        let angle_rad = (angle_deg - 90.0) * PI / 180.0; // -90 to start from top
        let x = center + r * angle_rad.cos();
        let y = center + r * angle_rad.sin();
        (x, y)
    };

    // Calculate coordinates for each dimension
    let (e_x, e_y) = angle_to_coords(0.0, e_radius); // Top: E
    let (s_x, s_y) = angle_to_coords(45.0, s_radius); // Top-right: S
    let (t_x, t_y) = angle_to_coords(90.0, t_radius); // Right: T
    let (j_x, j_y) = angle_to_coords(135.0, j_radius); // Bottom-right: J
    let (i_x, i_y) = angle_to_coords(180.0, i_radius); // Bottom: I (opposite of E)
    let (n_x, n_y) = angle_to_coords(225.0, n_radius); // Bottom-left: N (opposite of S)
    let (f_x, f_y) = angle_to_coords(270.0, f_radius); // Left: F (opposite of T)
    let (p_x, p_y) = angle_to_coords(315.0, p_radius); // Top-left: P (opposite of J)

    // Create polygon path (8 points in order)
    let polygon_points = format!(
        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
        e_x, e_y, s_x, s_y, t_x, t_y, j_x, j_y, i_x, i_y, n_x, n_y, f_x, f_y, p_x, p_y
    );

    // Create straight line grid (spider web style)
    // Grid levels: 4 concentric levels (25%, 50%, 75%, 100%)
    let grid_levels = 4;
    let axis_angles = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];

    // Create grid polygons (straight lines connecting points at same level)
    let grid_polygons = (1..=grid_levels).map(|i| {
        let r = radius * i as f32 / grid_levels as f32;
        let is_50_percent = i == 2; // Second level is 50%

        // Create polygon points for this grid level
        let grid_points = axis_angles.iter()
            .map(|angle| {
                let (x, y) = angle_to_coords(*angle, r);
                format!("{:.1},{:.1}", x, y)
            })
            .collect::<Vec<_>>()
            .join(" ");

        html! {
            <polygon
                points={grid_points}
                class={if is_50_percent { "radar-grid-polygon radar-baseline" } else { "radar-grid-polygon" }}
            />
        }
    }).collect::<Vec<_>>();

    // Create axis lines (8 axes, each 45 degrees apart)
    let axis_lines_html = axis_angles
        .iter()
        .map(|angle| {
            let (x, y) = angle_to_coords(*angle, radius);
            html! {
                <line
                    x1={center.to_string()}
                    y1={center.to_string()}
                    x2={x.to_string()}
                    y2={y.to_string()}
                    class="radar-axis-line"
                />
            }
        })
        .collect::<Vec<_>>();

    // Labels for all 8 dimensions
    let label_offset = radius + 22.0;
    let labels = [
        ("E", angle_to_coords(0.0, label_offset), "radar-label"),
        ("S", angle_to_coords(45.0, label_offset), "radar-label"),
        ("T", angle_to_coords(90.0, label_offset), "radar-label"),
        ("J", angle_to_coords(135.0, label_offset), "radar-label"),
        ("I", angle_to_coords(180.0, label_offset), "radar-label"),
        ("N", angle_to_coords(225.0, label_offset), "radar-label"),
        ("F", angle_to_coords(270.0, label_offset), "radar-label"),
        ("P", angle_to_coords(315.0, label_offset), "radar-label"),
    ];

    html! {
        <div class="mbti-radar-container">
            <svg
                class="mbti-radar-chart"
                width={size.to_string()}
                height={size.to_string()}
                viewBox={format!("0 0 {} {}", size, size)}
            >
                // Background grid (straight line spider web)
                <g class="radar-grid">
                    {for grid_polygons}
                    {for axis_lines_html}
                </g>

                // Baseline polygon (50% reference)
                <polygon
                    points={format!(
                        "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                        angle_to_coords(0.0, radius * 0.5).0, angle_to_coords(0.0, radius * 0.5).1,
                        angle_to_coords(45.0, radius * 0.5).0, angle_to_coords(45.0, radius * 0.5).1,
                        angle_to_coords(90.0, radius * 0.5).0, angle_to_coords(90.0, radius * 0.5).1,
                        angle_to_coords(135.0, radius * 0.5).0, angle_to_coords(135.0, radius * 0.5).1,
                        angle_to_coords(180.0, radius * 0.5).0, angle_to_coords(180.0, radius * 0.5).1,
                        angle_to_coords(225.0, radius * 0.5).0, angle_to_coords(225.0, radius * 0.5).1,
                        angle_to_coords(270.0, radius * 0.5).0, angle_to_coords(270.0, radius * 0.5).1,
                        angle_to_coords(315.0, radius * 0.5).0, angle_to_coords(315.0, radius * 0.5).1,
                    )}
                    class="radar-baseline-polygon"
                />

                // Data polygon (spider web style - lines only, no fill)
                <polygon
                    points={polygon_points.clone()}
                    class="radar-polygon"
                />

                // Data points
                <circle cx={e_x.to_string()} cy={e_y.to_string()} r="4" class="radar-point" />
                <circle cx={s_x.to_string()} cy={s_y.to_string()} r="4" class="radar-point" />
                <circle cx={t_x.to_string()} cy={t_y.to_string()} r="4" class="radar-point" />
                <circle cx={j_x.to_string()} cy={j_y.to_string()} r="4" class="radar-point" />
                <circle cx={i_x.to_string()} cy={i_y.to_string()} r="4" class="radar-point" />
                <circle cx={n_x.to_string()} cy={n_y.to_string()} r="4" class="radar-point" />
                <circle cx={f_x.to_string()} cy={f_y.to_string()} r="4" class="radar-point" />
                <circle cx={p_x.to_string()} cy={p_y.to_string()} r="4" class="radar-point" />

                // Axis labels (highlight if > 50%)
                <g class="radar-labels">
                    {for labels.iter().enumerate().map(|(idx, (text, (x, y), _class))| {
                        let is_highlight = match idx {
                            0 => e_value > 0.5,  // E
                            1 => s_value > 0.5,  // S
                            2 => t_value > 0.5,  // T
                            3 => j_value > 0.5,  // J
                            4 => i_value > 0.5,  // I
                            5 => n_value > 0.5,  // N
                            6 => f_value > 0.5,  // F
                            7 => p_value > 0.5,  // P
                            _ => false,
                        };

                        html! {
                            <text
                                x={x.to_string()}
                                y={y.to_string()}
                                class={if is_highlight { "radar-label radar-label-highlight" } else { "radar-label radar-label-dim" }}
                                text-anchor="middle"
                                dominant-baseline="middle"
                            >
                                {*text}
                            </text>
                        }
                    })}
                </g>
            </svg>

            // Legend (keep original 4 dimension pairs format)
            <div class="radar-legend">
                <div class="radar-legend-item">
                    <span class="radar-legend-label">{"E/I"}</span>
                    <span class="radar-legend-value">{format!("{:.0}%", props.ei_score * 100.0)}</span>
                </div>
                <div class="radar-legend-item">
                    <span class="radar-legend-label">{"S/N"}</span>
                    <span class="radar-legend-value">{format!("{:.0}%", props.sn_score * 100.0)}</span>
                </div>
                <div class="radar-legend-item">
                    <span class="radar-legend-label">{"T/F"}</span>
                    <span class="radar-legend-value">{format!("{:.0}%", props.tf_score * 100.0)}</span>
                </div>
                <div class="radar-legend-item">
                    <span class="radar-legend-label">{"J/P"}</span>
                    <span class="radar-legend-value">{format!("{:.0}%", props.jp_score * 100.0)}</span>
                </div>
            </div>
        </div>
    }
}

/// MBTI analysis component
#[function_component]
pub fn MbtiAnalysis(props: &MbtiAnalysisProps) -> Html {
    // Get MBTI description
    let description = MBTI_DESCRIPTIONS
        .iter()
        .find(|(mbti_type, _)| *mbti_type == props.mbti.mbti_type)
        .map(|(_, desc)| *desc)
        .unwrap_or("Personality type");

    // Dimension scores are used directly in the radar chart component

    // Confidence indicator
    let confidence_class = if props.mbti.confidence >= 0.8 {
        "confidence-high"
    } else if props.mbti.confidence >= 0.6 {
        "confidence-medium"
    } else {
        "confidence-low"
    };

    let confidence_text = if props.mbti.confidence >= 0.8 {
        "High Confidence"
    } else if props.mbti.confidence >= 0.6 {
        "Medium Confidence"
    } else {
        "Low Confidence"
    };

    html! {
        <div class="mbti-analysis">
            <div class="mbti-header">
                <div class="mbti-type-badge">
                    <span class="mbti-type">{&props.mbti.mbti_type}</span>
                    <span class={format!("mbti-confidence {}", confidence_class)}>
                        {confidence_text}
                    </span>
                </div>
                <p class="mbti-description">{description}</p>
            </div>

            <div class="mbti-dimensions">
                <h4>{"Personality Dimensions"}</h4>
                <MbtiRadarChart
                    ei_score={props.mbti.dimensions.ei_score}
                    sn_score={props.mbti.dimensions.sn_score}
                    tf_score={props.mbti.dimensions.tf_score}
                    jp_score={props.mbti.dimensions.jp_score}
                />
            </div>

            if !props.mbti.traits.is_empty() {
                <div class="mbti-traits">
                    <h4>{"Key Traits"}</h4>
                    <div class="traits-list">
                        {for props.mbti.traits.iter().map(|trait_name| {
                            html! {
                                <span class="trait-tag">{trait_name}</span>
                            }
                        })}
                    </div>
                </div>
            }

            <div class="mbti-analysis-text">
                <h4>{"Analysis"}</h4>
                <p>{&props.mbti.analysis}</p>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct SocialAnalysisProps {
    pub social: SocialData,
}

/// Social analysis component
#[function_component]
pub fn SocialAnalysis(props: &SocialAnalysisProps) -> Html {
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
                                <span class="word-tag" style={format!("font-size: {}px", (word.percentage * 20.0 + 12.0).clamp(10.0, 18.0))}>
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
pub struct EndpointViewProps {
    pub endpoint_data: Option<EndpointData>,
    pub is_loading: bool,
    pub error: Option<String>,
    pub ping_results: Vec<(String, Option<f64>)>,
    pub selected_endpoint: Option<String>,
    pub on_select_endpoint: Callback<String>,
    pub custom_endpoints: Vec<String>,
    pub custom_url_input: String,
    pub on_custom_url_input_change: Callback<InputEvent>,
    pub on_add_custom_endpoint: Callback<()>,
    pub custom_endpoint_error: Option<String>,
    pub is_adding_endpoint: bool,
}

#[derive(Properties, PartialEq, Clone)]
pub struct EndpointItemProps {
    pub index: usize,
    pub endpoint: String,
    pub latency: Option<f64>,
    pub is_selected: bool,
    pub on_select: Callback<String>,
    pub ping_attempted: bool, // Whether ping has been attempted (even if failed)
}

/// Endpoint view component
#[function_component]
pub fn EndpointView(props: &EndpointViewProps) -> Html {
    html! {
        <div class="card endpoint-card">
            <div class="card-content">
                if let Some(data) = &props.endpoint_data {
                    <div class="endpoint-header">
                        <h2>{"PolyEndpoint Registry"}</h2>
                        <div class="endpoint-info">
                            <div class="endpoint-info-item">
                                <span class="endpoint-label">{"Contract:"}</span>
                                <span class="endpoint-value">{&data.contract_address}</span>
                            </div>
                            <div class="endpoint-info-item">
                                <span class="endpoint-label">{"Network:"}</span>
                                <span class="endpoint-value">{&data.network}</span>
                            </div>
                            <div class="endpoint-info-item">
                                <span class="endpoint-label">{"Endpoints:"}</span>
                                <span class="endpoint-value">{format!("{}", data.endpoints.len())}</span>
                            </div>
                        </div>
                    </div>

                    <div class="endpoints-list">
                        <h3>{"Registered Endpoints"}</h3>
                        if data.endpoints.is_empty() {
                            <div class="no-endpoints">
                                <p>{"No endpoints registered yet."}</p>
                            </div>
                        } else {
                            <div class="endpoints-container">
                                {for data.endpoints.iter().enumerate().map(|(index, endpoint)| {
                                    let latency = props.ping_results.iter()
                                        .find(|(url, _)| url == endpoint)
                                        .and_then(|(_, latency)| *latency);
                                    let ping_attempted = props.ping_results.iter()
                                        .any(|(url, _)| url == endpoint);
                                    let is_selected = props.selected_endpoint.as_ref()
                                        .map(|s| s == endpoint)
                                        .unwrap_or(false);
                                    html! {
                                        <EndpointItem
                                            index={index}
                                            endpoint={endpoint.clone()}
                                            latency={latency}
                                            is_selected={is_selected}
                                            on_select={props.on_select_endpoint.clone()}
                                            ping_attempted={ping_attempted}
                                        />
                                    }
                                })}
                            </div>
                        }
                    </div>
                } else if props.is_loading {
                    <div class="endpoint-loading">
                        <div class="loading-spinner"></div>
                        <p>{"Loading endpoints..."}</p>
                    </div>
                } else if let Some(error) = &props.error {
                    <div class="error-message">
                        <p>{error}</p>
                    </div>
                } else {
                    <div class="no-endpoint-data">
                        <p>{"Click the link button to load endpoints"}</p>
                    </div>
                }

                // Always show custom endpoints section
                <div class="custom-endpoints-section">
                    <h3>{"Custom Endpoints"}</h3>
                    <div class="custom-endpoint-input">
                        <input
                            type="text"
                            class="custom-url-input"
                            placeholder="Enter custom API URL (e.g., https://api.example.com)"
                            value={props.custom_url_input.clone()}
                            oninput={props.on_custom_url_input_change.clone()}
                            onkeypress={Callback::from({
                                let on_add = props.on_add_custom_endpoint.clone();
                                move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Enter" {
                                        e.prevent_default();
                                        on_add.emit(());
                                    }
                                }
                            })}
                        />
                        <button
                            class="add-custom-endpoint-button"
                            onclick={props.on_add_custom_endpoint.clone().reform(|_| ())}
                            disabled={props.is_adding_endpoint}
                        >
                            if props.is_adding_endpoint {
                                {"Adding..."}
                            } else {
                                {"Add"}
                            }
                        </button>
                    </div>
                    if let Some(error) = &props.custom_endpoint_error {
                        <div class="custom-endpoint-error">
                            <p>{error}</p>
                        </div>
                    }
                    if !props.custom_endpoints.is_empty() {
                        <div class="endpoints-container">
                            {for props.custom_endpoints.iter().enumerate().map(|(index, endpoint)| {
                                let latency = props.ping_results.iter()
                                    .find(|(url, _)| url == endpoint)
                                    .and_then(|(_, latency)| *latency);
                                let is_selected = props.selected_endpoint.as_ref()
                                    .map(|s| s == endpoint)
                                    .unwrap_or(false);
                                let display_index = if let Some(data) = &props.endpoint_data {
                                    data.endpoints.len() + index
                                } else {
                                    index
                                };
                                let ping_attempted = props.ping_results.iter()
                                    .any(|(url, _)| url == endpoint);
                                html! {
                                    <EndpointItem
                                        index={display_index}
                                        endpoint={endpoint.clone()}
                                        latency={latency}
                                        is_selected={is_selected}
                                        on_select={props.on_select_endpoint.clone()}
                                        ping_attempted={ping_attempted}
                                    />
                                }
                            })}
                        </div>
                    }
                </div>
            </div>
        </div>
    }
}

/// Individual endpoint item with ping status
#[function_component]
fn EndpointItem(props: &EndpointItemProps) -> Html {
    let class_name = if props.is_selected {
        "endpoint-item selected"
    } else {
        "endpoint-item"
    };
    let endpoint = props.endpoint.clone();
    let on_select = props.on_select.clone();

    html! {
        <div class={class_name} onclick={Callback::from(move |_| on_select.emit(endpoint.clone()))}>
            <span class="endpoint-index">{format!("{}", props.index + 1)}</span>
            <span class="endpoint-url">{&props.endpoint}</span>
            if props.is_selected {
                <span class="endpoint-selected-indicator">{"✓"} </span>
            }
            if let Some(latency) = props.latency {
                <span class="endpoint-latency">
                    {format!("{:.0}ms", latency)}
                </span>
            } else if props.ping_attempted {
                // Ping was attempted but failed (likely CORS)
                <span class="endpoint-latency failed">{"CORS blocked"}</span>
            } else {
                // Still checking
                <span class="endpoint-latency checking">{"checking..."}</span>
            }
        </div>
    }
}

/// MBTI Skeleton Loading Component
#[derive(Properties, PartialEq, Clone)]
pub struct MbtiSkeletonProps {
    pub message: String,
}

#[function_component]
pub fn MbtiSkeleton(props: &MbtiSkeletonProps) -> Html {
    html! {
        <div class="mbti-analysis skeleton-container">
            <div class="skeleton-overlay"></div>
            <div class="mbti-header">
                <div class="mbti-type-badge">
                    <div class="skeleton-box skeleton-mbti-type"></div>
                    <div class="skeleton-box skeleton-confidence"></div>
                </div>
                <div class="skeleton-box skeleton-description"></div>
            </div>

            <div class="mbti-dimensions">
                <div class="skeleton-box skeleton-title"></div>
                {for (0..4).map(|_| {
                    html! {
                        <div class="dimension-item">
                            <div class="dimension-labels">
                                <div class="skeleton-box skeleton-label"></div>
                                <div class="skeleton-box skeleton-label"></div>
                            </div>
                            <div class="dimension-bar skeleton-bar">
                                <div class="skeleton-fill"></div>
                            </div>
                            <div class="skeleton-box skeleton-result"></div>
                        </div>
                    }
                })}
            </div>

            <div class="mbti-traits">
                <div class="skeleton-box skeleton-title"></div>
                <div class="traits-list">
                    {for (0..6).map(|_| {
                        html! {
                            <div class="skeleton-box skeleton-trait-tag"></div>
                        }
                    })}
                </div>
            </div>

            <div class="mbti-analysis-text">
                <div class="skeleton-box skeleton-title"></div>
                <div class="skeleton-box skeleton-text-line"></div>
                <div class="skeleton-box skeleton-text-line" style="width: 80%;"></div>
                <div class="skeleton-box skeleton-text-line" style="width: 90%;"></div>
            </div>

            <div class="skeleton-loading-message">
                <div class="skeleton-spinner"></div>
                <span>{&props.message}</span>
            </div>
        </div>
    }
}

/// Social Skeleton Loading Component
#[derive(Properties, PartialEq, Clone)]
pub struct SocialSkeletonProps {
    pub message: String,
}

#[function_component]
pub fn SocialSkeleton(props: &SocialSkeletonProps) -> Html {
    html! {
        <div class="social-analysis skeleton-container">
            <div class="skeleton-overlay"></div>

            <div class="social-stats">
                {for (0..3).map(|_| {
                    html! {
                        <div class="stat-item">
                            <div class="skeleton-box skeleton-stat-label"></div>
                            <div class="skeleton-box skeleton-stat-value"></div>
                        </div>
                    }
                })}
            </div>

            <div class="social-circles">
                <div class="skeleton-box skeleton-title"></div>
                {for (0..4).map(|_| {
                    html! {
                        <div class="circle-item">
                            <div class="skeleton-box skeleton-circle-label"></div>
                            <div class="composition-bar skeleton-bar">
                                <div class="skeleton-fill"></div>
                            </div>
                        </div>
                    }
                })}
            </div>

            <div class="interaction-style">
                <div class="skeleton-box skeleton-title"></div>
                <div class="interaction-stats">
                    {for (0..4).map(|_| {
                        html! {
                            <div class="interaction-item">
                                <div class="skeleton-box skeleton-interaction-label"></div>
                                <div class="interaction-bar skeleton-bar">
                                    <div class="skeleton-fill"></div>
                                </div>
                            </div>
                        }
                    })}
                </div>
            </div>

            <div class="mentioned-users">
                <div class="skeleton-box skeleton-title"></div>
                <div class="mentioned-list">
                    {for (0..5).map(|_| {
                        html! {
                            <div class="mentioned-item">
                                <div class="skeleton-box skeleton-mentioned-name"></div>
                                <div class="skeleton-box skeleton-mentioned-count"></div>
                                <div class="skeleton-box skeleton-mentioned-category"></div>
                            </div>
                        }
                    })}
                </div>
            </div>

            <div class="skeleton-loading-message">
                <div class="skeleton-spinner"></div>
                <span>{&props.message}</span>
            </div>
        </div>
    }
}
