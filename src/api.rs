use serde::Deserialize;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Request;
use web_sys::RequestInit;
use web_sys::RequestMode;
use web_sys::Response;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: String,
    pub name: String,
    pub description: String,
    pub tier: String,
    pub requires_payment: bool,
    pub default_body: Option<String>,
}

pub async fn make_request(
    base_url: &str,
    endpoint: &EndpointInfo,
    body: Option<String>,
    payment_header: Option<String>,
) -> Result<ApiResponse, String> {
    let url = format!("{}{}", base_url, endpoint.path);

    let opts = RequestInit::new();
    opts.set_method(&endpoint.method);
    opts.set_mode(RequestMode::Cors);

    // Add body for POST requests
    if endpoint.method == "POST" {
        if let Some(body_str) = body {
            opts.set_body(&wasm_bindgen::JsValue::from_str(&body_str));
        }
    }

    let request = Request::new_with_str_and_init(&url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    // Set headers
    let headers = request.headers();
    headers
        .set("Content-Type", "application/json")
        .map_err(|e| format!("Failed to set Content-Type: {:?}", e))?;

    // Add payment header if provided
    if let Some(ref payment) = payment_header {
        web_sys::console::log_1(
            &format!("🔐 Setting X-PAYMENT header (length: {})", payment.len()).into(),
        );
        headers
            .set("X-PAYMENT", payment)
            .map_err(|e| format!("Failed to set X-PAYMENT header: {:?}", e))?;
        web_sys::console::log_1(&"✅ X-PAYMENT header set successfully".into());
    } else {
        web_sys::console::log_1(&"ℹ️ No payment header provided".into());
    }

    let window = web_sys::window().ok_or("No window object")?;
    web_sys::console::log_1(&format!("🌐 Making request: {} {}", endpoint.method, url).into());
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Fetch failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;

    let status = resp.status();
    let status_text = resp.status_text();

    // Debug: Log request details
    web_sys::console::log_1(&format!("API Request: {} {}", endpoint.method, url).into());
    if let Some(payment) = payment_header {
        web_sys::console::log_1(&format!("X-PAYMENT header: {}", payment).into());
    }
    web_sys::console::log_1(&format!("Response status: {} {}", status, status_text).into());

    // Get response headers
    let mut response_headers = Vec::new();
    let headers_iter = resp.headers().entries();

    if let Some(iterator) = js_sys::try_iter(&headers_iter).ok().flatten() {
        for entry in iterator.flatten() {
            if let Ok(array) = entry.dyn_into::<js_sys::Array>() {
                if array.length() == 2 {
                    let key = array.get(0).as_string().unwrap_or_default();
                    let value = array.get(1).as_string().unwrap_or_default();
                    response_headers.push((key, value));
                }
            }
        }
    }

    // Get response body
    let text = JsFuture::from(
        resp.text()
            .map_err(|e| format!("No text method: {:?}", e))?,
    )
    .await
    .map_err(|e| format!("Failed to get text: {:?}", e))?;

    let body = text.as_string().unwrap_or_default();

    // Debug: Log response body
    web_sys::console::log_1(&format!("Response body: {}", body).into());

    Ok(ApiResponse {
        status,
        status_text,
        headers: response_headers,
        body,
    })
}
