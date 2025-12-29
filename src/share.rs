use js_sys;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::farcaster;
use crate::icons;

/// Share options component
#[derive(Properties, PartialEq, Clone)]
pub struct ShareButtonProps {
    #[prop_or_default]
    pub url: Option<String>,
    #[prop_or_default]
    pub text: Option<String>,
    pub is_farcaster_env: bool,
}

#[function_component]
pub fn ShareButton(props: &ShareButtonProps) -> Html {
    let show_share_menu = use_state(|| false);
    let menu_ref = use_node_ref();
    let button_ref = use_node_ref();

    let url = props.url.clone().unwrap_or_else(|| {
        web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default()
    });

    let text = props
        .text
        .clone()
        .unwrap_or_else(|| "Check out Polyjuice: Discover & Chat with Farcaster Users".to_string());

    // Close menu when clicking outside
    {
        let show_share_menu = show_share_menu.clone();
        let menu_ref = menu_ref.clone();
        let button_ref = button_ref.clone();
        use_effect_with(show_share_menu.clone(), move |is_open| {
            if !**is_open {
                return;
            }

            let closure = Closure::<dyn FnMut(web_sys::MouseEvent)>::new({
                let show_share_menu = show_share_menu.clone();
                let menu_ref = menu_ref.clone();
                let button_ref = button_ref.clone();
                move |e: web_sys::MouseEvent| {
                    let target = e.target();
                    if let Some(target) = target {
                        let target_element = target.dyn_ref::<web_sys::Element>();
                        let menu_element = menu_ref.get();
                        let button_element = button_ref.get();

                        // Check if click is outside both menu and button
                        let is_outside = if let (Some(target_el), Some(menu_el)) =
                            (target_element, menu_element.as_ref())
                        {
                            !menu_el.contains(Some(target_el))
                        } else {
                            true
                        };

                        let is_outside_button = if let (Some(target_el), Some(button_el)) =
                            (target_element, button_element.as_ref())
                        {
                            !button_el.contains(Some(target_el))
                        } else {
                            true
                        };

                        if is_outside && is_outside_button {
                            show_share_menu.set(false);
                        }
                    }
                }
            });

            let document = web_sys::window().and_then(|w| w.document()).unwrap();

            if document
                .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .is_ok()
            {
                closure.forget();
            }
        });
    }

    let on_share_click = {
        let show_share_menu = show_share_menu.clone();
        Callback::from(move |e: yew::MouseEvent| {
            e.stop_propagation();
            show_share_menu.set(!*show_share_menu);
        })
    };

    let on_farcaster_share = {
        let url = url.clone();
        let text = text.clone();
        let show_share_menu = show_share_menu.clone();
        Callback::from(move |_| {
            let url_clone = url.clone();
            let text_clone = text.clone();
            let show_share_menu_clone = show_share_menu.clone();
            spawn_local(async move {
                // Include URL in the text
                let text_with_url = format!("{}\n\n{}", text_clone, url_clone);
                // Also pass URL as embed for rich preview
                if let Err(e) =
                    farcaster::compose_cast(&text_with_url, Some(vec![url_clone.clone()])).await
                {
                    web_sys::console::error_1(&format!("Failed to compose cast: {}", e).into());
                }
                show_share_menu_clone.set(false);
            });
        })
    };

    let on_twitter_share = {
        let url = url.clone();
        let text = text.clone();
        let show_share_menu = show_share_menu.clone();
        Callback::from(move |_| {
            // Use JavaScript's encodeURIComponent
            let encoded_text = js_sys::encode_uri_component(&text);
            let encoded_url = js_sys::encode_uri_component(&url);
            let twitter_url = format!(
                "https://twitter.com/intent/tweet?text={}&url={}",
                encoded_text.as_string().unwrap_or_default(),
                encoded_url.as_string().unwrap_or_default()
            );
            if let Some(window) = web_sys::window() {
                if let Err(e) = window.open_with_url_and_target(&twitter_url, "_blank") {
                    web_sys::console::error_1(&format!("Failed to open Twitter: {:?}", e).into());
                }
            }
            show_share_menu.set(false);
        })
    };

    let on_copy_link = {
        let url = url.clone();
        let show_share_menu = show_share_menu.clone();
        Callback::from(move |_| {
            let url_clone = url.clone();
            let show_share_menu_clone = show_share_menu.clone();
            spawn_local(async move {
                let window = web_sys::window().unwrap();
                // Try modern Clipboard API using js_sys::Reflect
                if let Ok(navigator_val) = js_sys::Reflect::get(&window, &"navigator".into()) {
                    if !navigator_val.is_null() && !navigator_val.is_undefined() {
                        if let Ok(clipboard_val) =
                            js_sys::Reflect::get(&navigator_val, &"clipboard".into())
                        {
                            if !clipboard_val.is_null() && !clipboard_val.is_undefined() {
                                if let Ok(write_text_fn) =
                                    js_sys::Reflect::get(&clipboard_val, &"writeText".into())
                                {
                                    if let Some(write_fn) =
                                        write_text_fn.dyn_ref::<js_sys::Function>()
                                    {
                                        if let Ok(promise_val) =
                                            write_fn.call1(&clipboard_val, &url_clone.into())
                                        {
                                            if let Ok(promise) =
                                                promise_val.dyn_into::<js_sys::Promise>()
                                            {
                                                match wasm_bindgen_futures::JsFuture::from(promise)
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        web_sys::console::log_1(
                                                            &"URL copied to clipboard".into(),
                                                        );
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                show_share_menu_clone.set(false);
            });
        })
    };

    html! {
        <div style="position: relative; display: inline-block;">
            <button
                ref={button_ref.clone()}
                class="share-button"
                onclick={on_share_click}
                style="background: none; border: none; font-size: 20px; cursor: pointer; padding: 4px 8px; color: white; display: flex; align-items: center; justify-content: center;"
            >
                {icons::share()}
            </button>
            {
                if *show_share_menu {
                    html! {
                        <div
                            ref={menu_ref.clone()}
                            onclick={Callback::from(|e: yew::MouseEvent| {
                                e.stop_propagation();
                            })}
                            style="
                                position: absolute;
                                top: 100%;
                                right: 0;
                                margin-top: 8px;
                                background: rgba(255, 255, 255, 0.95);
                                backdrop-filter: blur(20px);
                                -webkit-backdrop-filter: blur(20px);
                                border-radius: 12px;
                                padding: 8px;
                                box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
                                z-index: 1000;
                                min-width: 180px;
                                border: 1px solid rgba(255, 255, 255, 0.3);
                            ">
                            {
                                if props.is_farcaster_env {
                                    html! {
                                        <button
                                            onclick={on_farcaster_share}
                                            style="
                                                width: 100%;
                                                padding: 12px 16px;
                                                background: none;
                                                border: none;
                                                border-radius: 8px;
                                                cursor: pointer;
                                                display: flex;
                                                align-items: center;
                                                gap: 12px;
                                                color: #333;
                                                font-size: 14px;
                                                font-weight: 500;
                                                text-align: left;
                                                transition: background 0.2s;
                                            "
                                        >
                                            <i class="fab fa-farcaster" style="color: #8A63D2; font-size: 18px;"></i>
                                            <span>{"Share on Farcaster"}</span>
                                        </button>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            <button
                                onclick={on_twitter_share}
                                style="
                                    width: 100%;
                                    padding: 12px 16px;
                                    background: none;
                                    border: none;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    display: flex;
                                    align-items: center;
                                    gap: 12px;
                                    color: #333;
                                    font-size: 14px;
                                    font-weight: 500;
                                    text-align: left;
                                    transition: background 0.2s;
                                "
                                        >
                                <i class="fab fa-twitter" style="color: #1DA1F2; font-size: 18px;"></i>
                                <span>{"Share on Twitter"}</span>
                            </button>
                            <div style="height: 1px; background: rgba(0, 0, 0, 0.1); margin: 4px 0;"></div>
                            <button
                                onclick={on_copy_link}
                                style="
                                    width: 100%;
                                    padding: 12px 16px;
                                    background: none;
                                    border: none;
                                    border-radius: 8px;
                                    cursor: pointer;
                                    display: flex;
                                    align-items: center;
                                    gap: 12px;
                                    color: #333;
                                    font-size: 14px;
                                    font-weight: 500;
                                    text-align: left;
                                    transition: background 0.2s;
                                "
                                        >
                                <i class="fas fa-link" style="color: #666; font-size: 18px;"></i>
                                <span>{"Copy Link"}</span>
                            </button>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
