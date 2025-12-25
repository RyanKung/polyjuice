use serde_json::json;
use wasm_bindgen::JsCast;
use web_sys;

/// Update Farcaster Mini App embed meta tags based on current route
/// This ensures each shareable URL has proper embed metadata
pub fn update_embed_meta_tags(
    route: &str,
    custom_url: Option<&str>,
    custom_title: Option<&str>,
    custom_image: Option<&str>,
) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let head = document.head().unwrap();

    // Get current URL
    let current_url = custom_url
        .map(|s| s.to_string())
        .or_else(|| window.location().href().ok())
        .unwrap_or_else(|| "https://miniapp.polyjuice.io".to_string());

    // Determine embed configuration based on route
    let (image_url, button_title, target_url) = match route {
        route if route.starts_with("/annual-report/") => {
            // Annual Report page
            let image = custom_image.unwrap_or("https://miniapp.polyjuice.io/imgs/preview.png");
            let title = custom_title.unwrap_or("View Annual Report");
            (image.to_string(), title.to_string(), current_url.clone())
        }
        route if route.starts_with("/profile/") => {
            // Profile page
            let image = custom_image.unwrap_or("https://miniapp.polyjuice.io/imgs/preview.png");
            let title = custom_title.unwrap_or("View Profile");
            (image.to_string(), title.to_string(), current_url.clone())
        }
        route if route.starts_with("/chat/") => {
            // Chat page
            let image = custom_image.unwrap_or("https://miniapp.polyjuice.io/imgs/preview.png");
            let title = custom_title.unwrap_or("Start Chat");
            (image.to_string(), title.to_string(), current_url.clone())
        }
        _ => {
            // Default/Home page
            let image = custom_image.unwrap_or("https://miniapp.polyjuice.io/imgs/preview.png");
            let title = custom_title.unwrap_or("Open Polyjuice");
            (image.to_string(), title.to_string(), current_url.clone())
        }
    };

    // Create embed JSON
    let embed_json = json!({
        "version": "1",
        "imageUrl": image_url,
        "button": {
            "title": button_title,
            "action": {
                "type": "launch_miniapp",
                "url": target_url,
                "name": "polyjuice",
                "splashImageUrl": "https://miniapp.polyjuice.io/imgs/splash.png",
                "splashBackgroundColor": "#667eea"
            }
        }
    });

    let embed_json_str = embed_json.to_string();

    // Update or create fc:miniapp meta tag
    update_or_create_meta_tag(&head, "fc:miniapp", &embed_json_str);

    // Update or create fc:frame meta tag (for backward compatibility)
    let frame_json = json!({
        "version": "1",
        "imageUrl": image_url,
        "button": {
            "title": button_title,
            "action": {
                "type": "launch_frame",
                "url": target_url,
                "name": "polyjuice",
                "splashImageUrl": "https://miniapp.polyjuice.io/imgs/splash.png",
                "splashBackgroundColor": "#667eea"
            }
        }
    });
    let frame_json_str = frame_json.to_string();
    update_or_create_meta_tag(&head, "fc:frame", &frame_json_str);
}

/// Helper function to update or create a meta tag
fn update_or_create_meta_tag(head: &web_sys::HtmlHeadElement, name: &str, content: &str) {
    let document = head.owner_document().unwrap();

    // Try to find existing meta tag
    let existing = head
        .query_selector(&format!("meta[name=\"{}\"]", name))
        .ok()
        .flatten();

    if let Some(meta) = existing {
        // Update existing meta tag
        if let Ok(meta_element) = meta.dyn_into::<web_sys::HtmlMetaElement>() {
            meta_element.set_content(content);
        }
    } else {
        // Create new meta tag
        let meta = document
            .create_element("meta")
            .unwrap()
            .dyn_into::<web_sys::HtmlMetaElement>()
            .unwrap();
        meta.set_name(name);
        meta.set_content(content);
        head.append_child(&meta).unwrap();
    }
}

/// Initialize embed meta tags for the current route
pub fn init_embed_meta_tags() {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let pathname = location.pathname().unwrap_or_default();
    
    update_embed_meta_tags(&pathname, None, None, None);
}

