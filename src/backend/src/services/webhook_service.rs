// use ic_cdk::api::management_canister::http_request::{
//     http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, 
//     HttpResponse, TransformArgs, TransformContext
// };
// use serde::{Deserialize, Serialize};

// // Fixed webhook URL for n8n
// const N8N_WEBHOOK_URL: &str = "https://oversyn.com/webhook/6b8bc071-50a4-481a-bce0-2e92bd04b171";

// // Data structure for our payload
// #[derive(Serialize, Deserialize)]
// pub struct WebhookPayload {
//     pub canister_id: String,
//     pub message: String,
//     pub timestamp: u64,
// }

// // Transform function for HTTP responses
// pub fn transform_response(args: TransformArgs) -> HttpResponse {
//     let headers = vec![
//         HttpHeader {
//             name: "Content-Security-Policy".to_string(),
//             value: "default-src 'self'".to_string(),
//         },
//         HttpHeader {
//             name: "X-Content-Type-Options".to_string(),
//             value: "nosniff".to_string(),
//         },
//     ];

//     HttpResponse {
//         status: args.response.status,
//         body: args.response.body,
//         headers,
//     }
// }

// // Function to send data to webhook
// pub async fn send_to_webhook(message: String) -> String {
//     // Create payload with current time
//     let payload = WebhookPayload {
//         canister_id: ic_cdk::id().to_string(),
//         message,
//         timestamp: ic_cdk::api::time(),
//     };
    
//     // Serialize payload to JSON
//     let json_payload = match serde_json::to_string(&payload) {
//         Ok(json) => json,
//         Err(e) => return format!("Failed to serialize payload: {}", e),
//     };
    
//     // Prepare request headers
//     let request_headers = vec![
//         HttpHeader {
//             name: "User-Agent".to_string(),
//             value: "ic_canister_webhook".to_string(),
//         },
//         HttpHeader {
//             name: "Content-Type".to_string(),
//             value: "application/json".to_string(),
//         },
//     ];
    
//     // Configure the HTTP request
//     let request = CanisterHttpRequestArgument {
//         url: N8N_WEBHOOK_URL.to_string(),
//         method: HttpMethod::POST,
//         body: Some(json_payload.into_bytes()),
//         max_response_bytes: None,
//         transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
//         headers: request_headers,
//     };
    
//     // Make the request with required cycles
//     // 200 billion cycles is a common amount for HTTP outcalls
//     match http_request(request, 200_000_000_000_u128).await {
//         Ok((response,)) => {
//             // Parse response body to string
//             String::from_utf8(response.body)
//                 .unwrap_or_else(|_| "Failed to decode response".to_string())
//         }
//         Err((r, m)) => {
//             // Format error message if request fails
//             format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}")
//         }
//     }
// }


use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, 
    HttpResponse, TransformArgs, TransformContext
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::custom_print;

// Fixed webhook URL for n8n
const N8N_WEBHOOK_URL: &str = "https://oversyn.com/webhook-test/6b8bc071-50a4-481a-bce0-2e92bd04b171";

// Basic data structure for our payload
#[derive(Serialize, Deserialize)]
pub struct WebhookPayload {
    pub canister_id: String,
    pub message: String,
    pub timestamp: u64,
}

// Enhanced payload for file automation
#[derive(Serialize, Deserialize)]
pub struct FileAutomationPayload {
    pub canister_id: String,
    pub chat_id: u64,
    pub message_id: Option<i32>,
    pub current_path: String,
    pub file_info: Option<FileInfo>,
    pub timestamp: u64,
}

// Structure to hold file information
#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String, 
    pub message_id: i32,
    pub mime_type: Option<String>,
    pub size: Option<u64>,
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
    
    send_payload_to_webhook(json_payload).await
}

// New function for file automation
pub async fn send_file_automation(
    chat_id: u64,
    message_id: Option<i32>,
    current_path: PathBuf,
    file_info: Option<FileInfo>
) -> String {
    // Create payload with file information
    let payload = FileAutomationPayload {
        canister_id: ic_cdk::id().to_string(),
        chat_id,
        message_id,
        current_path: current_path.to_string_lossy().to_string(),
        file_info,
        timestamp: ic_cdk::api::time(),
    };
    
    // Serialize payload to JSON
    let json_payload = match serde_json::to_string(&payload) {
        Ok(json) => json,
        Err(e) => return format!("Failed to serialize payload: {}", e),
    };
    
    custom_print!("Sending automation payload: {}", json_payload);
    send_payload_to_webhook(json_payload).await
}

// Helper function to send payload to webhook
async fn send_payload_to_webhook(json_payload: String) -> String {
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
    match http_request(request, 20_900_000_000_u128).await {
        Ok((response,)) => {
            // Parse response body to string
            let result = String::from_utf8(response.body)
                .unwrap_or_else(|_| "Failed to decode response".to_string());
            custom_print!("Received response from webhook: {}", result);  // Log the response
            result
        }
        Err((r, m)) => {
            // Format error message if request fails
            let error_msg = format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            custom_print!("Error sending to webhook: {}", error_msg);  // Log the error
            error_msg
        }
    }
}