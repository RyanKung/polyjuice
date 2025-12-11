use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirements {
    pub scheme: String,
    pub network: String,
    pub max_amount_required: String,
    pub asset: String,
    pub pay_to: String,
    pub resource: String,
    pub description: String,
    pub mime_type: Option<String>,
    pub max_timeout_seconds: Option<u64>,
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequirementsResponse {
    pub x402_version: u32,
    pub error: String,
    pub accepts: Vec<PaymentRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPayload {
    #[serde(rename = "x402Version")]
    pub x402_version: u32,
    pub scheme: String,
    pub network: String,
    pub resource: String,
    pub payload: ExactPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactPayload {
    pub signature: String,
    pub authorization: Authorization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorization {
    /// Payer's wallet address
    pub from: String,
    /// Recipient's wallet address
    pub to: String,
    /// Payment amount in atomic units
    pub value: String,
    /// Unix timestamp when authorization becomes valid
    #[serde(rename = "validAfter")]
    pub valid_after: String,
    /// Unix timestamp when authorization expires
    #[serde(rename = "validBefore")]
    pub valid_before: String,
    /// 32-byte random nonce to prevent replay attacks
    pub nonce: String,
}

impl PaymentPayload {
    pub fn to_base64(&self) -> Result<String, String> {
        let json = serde_json::to_string(self)
            .map_err(|e| format!("Failed to serialize payment: {}", e))?;
        Ok(base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            json.as_bytes(),
        ))
    }
}

/// Create EIP-712 typed data for EIP-3009 TransferWithAuthorization
pub fn create_eip712_typed_data(
    requirements: &PaymentRequirements,
    payer: &str,
    nonce: &str,
    timestamp: u64,
) -> Result<String, String> {
    // Determine chain ID from network
    let chain_id = match requirements.network.as_str() {
        "base-sepolia" => 84532,
        "base-mainnet" | "base" => 8453,
        "ethereum-mainnet" | "ethereum" => 1,
        "ethereum-sepolia" | "sepolia" => 11155111,
        _ => return Err(format!("Unsupported network: {}", requirements.network)),
    };

    // Normalize addresses (lowercase, with 0x prefix)
    let payer_normalized = normalize_address(payer);
    let payee_normalized = normalize_address(&requirements.pay_to);

    // Calculate valid_after and valid_before
    let valid_after = timestamp;
    let valid_before = timestamp + requirements.max_timeout_seconds.unwrap_or(60);

    // Determine the verifying contract based on the asset (USDC)
    let verifying_contract = normalize_address(&requirements.asset);

    let typed_data = serde_json::json!({
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "TransferWithAuthorization": [
                {"name": "from", "type": "address"},
                {"name": "to", "type": "address"},
                {"name": "value", "type": "uint256"},
                {"name": "validAfter", "type": "uint256"},
                {"name": "validBefore", "type": "uint256"},
                {"name": "nonce", "type": "bytes32"}
            ]
        },
        "primaryType": "TransferWithAuthorization",
        "domain": {
            "name": requirements.extra.as_ref()
                .and_then(|e| e.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("USD Coin"),
            "version": requirements.extra.as_ref()
                .and_then(|e| e.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("2"),
            "chainId": chain_id,
            "verifyingContract": verifying_contract
        },
        "message": {
            "from": payer_normalized,
            "to": payee_normalized,
            "value": requirements.max_amount_required,
            "validAfter": valid_after,
            "validBefore": valid_before,
            "nonce": nonce
        }
    });

    // Serialize to JSON string with proper formatting
    let json_string = serde_json::to_string_pretty(&typed_data)
        .map_err(|e| format!("Failed to serialize typed data: {}", e))?;

    // Validate the serialized JSON by parsing it back
    let _validation: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| format!("Serialized JSON is invalid: {}", e))?;

    // Additional validation: ensure all required fields are present
    let parsed_validation: serde_json::Value = serde_json::from_str(&json_string)
        .map_err(|e| format!("Failed to parse for validation: {}", e))?;

    // Check domain (EIP-3009 TransferWithAuthorization)
    if !parsed_validation["domain"]["name"].is_string()
        || !parsed_validation["domain"]["version"].is_string()
        || (!parsed_validation["domain"]["chainId"].is_number()
            && !parsed_validation["domain"]["chainId"].is_string())
        || !parsed_validation["domain"]["verifyingContract"].is_string()
    {
        return Err("Invalid domain structure in typed data".to_string());
    }

    // Check message (EIP-3009 TransferWithAuthorization)
    if !parsed_validation["message"]["from"].is_string()
        || !parsed_validation["message"]["to"].is_string()
        || !parsed_validation["message"]["value"].is_string()
        || (!parsed_validation["message"]["validAfter"].is_number()
            && !parsed_validation["message"]["validAfter"].is_string())
        || (!parsed_validation["message"]["validBefore"].is_number()
            && !parsed_validation["message"]["validBefore"].is_string())
        || !parsed_validation["message"]["nonce"].is_string()
    {
        return Err("Invalid message structure in typed data".to_string());
    }

    Ok(json_string)
}

/// Normalize Ethereum address (lowercase with 0x prefix)
fn normalize_address(addr: &str) -> String {
    let addr = addr.trim();
    if addr.starts_with("0x") || addr.starts_with("0X") {
        format!("0x{}", addr[2..].to_lowercase())
    } else {
        format!("0x{}", addr.to_lowercase())
    }
}

/// Generate a random nonce (32 bytes for bytes32)
pub fn generate_nonce() -> String {
    // Generate a 32-byte random nonce
    // Since we're in WASM, we'll use the available random functionality
    // to generate a hex string that represents a 32-byte value
    let mut bytes: Vec<u8> = Vec::with_capacity(32);

    for _ in 0..32 {
        // Generate random byte (0-255)
        let random = js_sys::Math::random() * 256.0;
        bytes.push(random as u8);
    }

    // Convert to hex string with 0x prefix
    format!("0x{}", hex::encode(bytes))
}

/// Get current Unix timestamp in seconds
pub fn get_timestamp() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

/// Create payment payload from requirements and signature
pub fn create_payment_payload(
    requirements: &PaymentRequirements,
    payer: &str,
    signature: &str,
    nonce: &str,
    timestamp: u64,
) -> PaymentPayload {
    // Calculate valid_after and valid_before timestamps
    let valid_after = timestamp.to_string();
    let valid_before = (timestamp + requirements.max_timeout_seconds.unwrap_or(60)).to_string();

    PaymentPayload {
        x402_version: 1,
        scheme: requirements.scheme.clone(),
        network: requirements.network.clone(),
        resource: requirements.resource.clone(),
        payload: ExactPayload {
            signature: signature.to_string(),
            authorization: Authorization {
                from: normalize_address(payer),
                to: normalize_address(&requirements.pay_to),
                value: requirements.max_amount_required.clone(),
                valid_after,
                valid_before,
                nonce: nonce.to_string(),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_address() {
        assert_eq!(normalize_address("0xABCD1234"), "0xabcd1234");
        assert_eq!(normalize_address("ABCD1234"), "0xabcd1234");
        assert_eq!(normalize_address("0X1234"), "0x1234");
    }
}
