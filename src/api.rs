use base64::engine::general_purpose;
use base64::Engine as _;
use hmac::Hmac;
use hmac::Mac;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
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

/// Compute SHA256 hash of body
fn compute_body_hash(body: &str) -> String {
    if body.is_empty() {
        return String::new();
    }
    let mut hasher = Sha256::new();
    hasher.update(body.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}

/// Build signature string for authentication
fn build_signature_string(
    method: &str,
    path: &str,
    query: &str,
    body_hash: &str,
    timestamp: i64,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        method, path, query, body_hash, timestamp
    )
}

/// Sign request for authentication
fn sign_request(
    method: &str,
    path: &str,
    query: &str,
    body: Option<&str>,
    secret_hex: &str,
    timestamp: i64,
) -> Result<String, String> {
    // Compute body hash
    let body_hash = body.map(compute_body_hash).unwrap_or_default();

    // Build signature string
    let sig_string = build_signature_string(method, path, query, &body_hash, timestamp);

    // Decode secret from hex
    let secret_bytes =
        hex::decode(secret_hex).map_err(|e| format!("Invalid secret format: {}", e))?;

    // Compute HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_bytes)
        .map_err(|e| format!("Failed to create HMAC: {}", e))?;
    mac.update(sig_string.as_bytes());
    let signature = mac.finalize().into_bytes();

    // Base64 encode
    Ok(general_purpose::STANDARD.encode(signature))
}

/// Add authentication headers to a request if token and secret are configured
pub fn add_auth_headers(
    headers: &web_sys::Headers,
    method: &str,
    url: &str,
    body: Option<&str>,
) -> Result<(), String> {
    if let (Some(token), Some(secret)) = (option_env!("AUTH_TOKEN"), option_env!("AUTH_SECRET")) {
        // Parse URL to get path and query
        let url_obj =
            web_sys::Url::new(url).map_err(|e| format!("Failed to parse URL: {:?}", e))?;
        let mut path = url_obj.pathname();
        // Remove /api or /mcp prefix for signature (server routes are nested, so path doesn't include prefix)
        if path.starts_with("/api/") {
            path = path.strip_prefix("/api").unwrap_or(&path).to_string();
        } else if path.starts_with("/mcp/") {
            path = path.strip_prefix("/mcp").unwrap_or(&path).to_string();
        }
        let query = url_obj.search().trim_start_matches('?').to_string();

        // Get current timestamp
        let timestamp = js_sys::Date::now() as i64 / 1000; // Convert to seconds

        // Sign the request
        match sign_request(method, &path, &query, body, secret, timestamp) {
            Ok(signature) => {
                headers
                    .set("X-Token", token)
                    .map_err(|e| format!("Failed to set X-Token header: {:?}", e))?;
                headers
                    .set("X-Timestamp", &timestamp.to_string())
                    .map_err(|e| format!("Failed to set X-Timestamp header: {:?}", e))?;
                headers
                    .set("X-Signature", &signature)
                    .map_err(|e| format!("Failed to set X-Signature header: {:?}", e))?;
                Ok(())
            }
            Err(e) => Err(format!("Failed to sign request: {}", e)),
        }
    } else {
        Ok(()) // No auth configured, skip
    }
}

pub async fn make_request(
    base_url: &str,
    endpoint: &EndpointInfo,
    body: Option<String>,
    payment_header: Option<String>,
) -> Result<ApiResponse, String> {
    let url = format!("{}{}", base_url, endpoint.path);

    // Save body reference for signing (before it's moved)
    let body_for_signing = body.as_deref();

    let opts = RequestInit::new();
    opts.set_method(&endpoint.method);
    opts.set_mode(RequestMode::Cors);

    // Add body for POST requests
    if endpoint.method == "POST" {
        if let Some(body_str) = &body {
            opts.set_body(&wasm_bindgen::JsValue::from_str(body_str));
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

    // Add authentication headers if token and secret are configured
    if let (Some(token), Some(secret)) = (option_env!("AUTH_TOKEN"), option_env!("AUTH_SECRET")) {
        // Parse URL to get path and query
        let url_obj =
            web_sys::Url::new(&url).map_err(|e| format!("Failed to parse URL: {:?}", e))?;
        let mut path = url_obj.pathname();
        // Remove /api or /mcp prefix for signature (server routes are nested, so path doesn't include prefix)
        if path.starts_with("/api/") {
            path = path.strip_prefix("/api").unwrap_or(&path).to_string();
        } else if path.starts_with("/mcp/") {
            path = path.strip_prefix("/mcp").unwrap_or(&path).to_string();
        }
        let query = url_obj.search().trim_start_matches('?').to_string();

        // Get current timestamp
        let timestamp = js_sys::Date::now() as i64 / 1000; // Convert to seconds

        // Sign the request
        let body_hash = body_for_signing.map(compute_body_hash).unwrap_or_default();
        let sig_string =
            build_signature_string(&endpoint.method, &path, &query, &body_hash, timestamp);
        web_sys::console::log_1(
            &format!(
                "🔐 Signing request - Method: {}, Path: {}, Query: '{}', Body hash: '{}', Timestamp: {}, Sig string: {:?}",
                endpoint.method, path, query, body_hash, timestamp, sig_string
            )
            .into(),
        );

        match sign_request(
            &endpoint.method,
            &path,
            &query,
            body_for_signing,
            secret,
            timestamp,
        ) {
            Ok(signature) => {
                web_sys::console::log_1(&"🔐 Setting authentication headers".into());
                web_sys::console::log_1(
                    &format!(
                        "🔐 Signature: {}...",
                        &signature.chars().take(20).collect::<String>()
                    )
                    .into(),
                );
                headers
                    .set("X-Token", token)
                    .map_err(|e| format!("Failed to set X-Token header: {:?}", e))?;
                headers
                    .set("X-Timestamp", &timestamp.to_string())
                    .map_err(|e| format!("Failed to set X-Timestamp header: {:?}", e))?;
                headers
                    .set("X-Signature", &signature)
                    .map_err(|e| format!("Failed to set X-Signature header: {:?}", e))?;
                web_sys::console::log_1(&"✅ Authentication headers set successfully".into());
            }
            Err(e) => {
                web_sys::console::warn_1(&format!("⚠️ Failed to sign request: {}", e).into());
            }
        }
    } else {
        web_sys::console::log_1(
            &"ℹ️ No authentication configured (AUTH_TOKEN/AUTH_SECRET not set)".into(),
        );
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
