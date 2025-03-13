use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, 
    HttpResponse, TransformArgs, TransformContext
};
use serde::{Deserialize, Serialize};

// Fixed webhook URL for n8n
const N8N_WEBHOOK_URL: &str = "https://oversyn.com/webhook/6b8bc071-50a4-481a-bce0-2e92bd04b171";

// Data structure for our payload
#[derive(Serialize, Deserialize)]
pub struct WebhookPayload {
    pub canister_id: String,
    pub message: String,
    pub timestamp: u64,
}

// Transform function for HTTP responses
pub fn transform_response(args: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    HttpResponse {
        status: args.response.status,
        body: args.response.body,
        headers,
    }
}

// Function to send data to webhook
pub async fn send_to_webhook(message: String) -> String {
    // Create payload with current time
    let payload = WebhookPayload {
        canister_id: ic_cdk::id().to_string(),
        message,
        timestamp: ic_cdk::api::time(),
    };
    
    // Serialize payload to JSON
    let json_payload = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => return format!("Failed to serialize payload: {}", e),
    };
    
    // Prepare request headers
    let request_headers = vec![
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "ic_canister_webhook".to_string(),
        },
        HttpHeader {
            name: "Content-Type".to_string(),
            value: "application/json".to_string(),
        },
    ];
    
    // Configure the HTTP request
    let request = CanisterHttpRequestArgument {
        url: N8N_WEBHOOK_URL.to_string(),
        method: HttpMethod::POST,
        body: Some(json_payload.into_bytes()),
        max_response_bytes: None,
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
        headers: request_headers,
    };
    
    // Make the request with required cycles
    // 200 billion cycles is a common amount for HTTP outcalls
    match http_request(request, 200_000_000_000_u128).await {
        Ok((response,)) => {
            // Parse response body to string
            String::from_utf8(response.body)
                .unwrap_or_else(|_| "Failed to decode response".to_string())
        }
        Err((r, m)) => {
            // Format error message if request fails
            format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}")
        }
    }
}