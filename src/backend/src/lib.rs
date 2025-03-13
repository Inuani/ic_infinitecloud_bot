// mod controllers;
// mod repositories;
// mod services;
// mod utils;


// use ic_cdk::api::management_canister::http_request::{
//     http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
//     TransformContext
// };

// use crate::repositories::{HttpRequest, HttpUpdateRequest};

// use ic_cdk_macros::query;
// use serde::{Deserialize, Serialize};

// mod controllers;
// mod repositories;
// mod services;
// mod utils;

// // Add this transform function for HTTP responses
// #[query]
// fn transform(args: TransformArgs) -> HttpResponse {
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

// // Add a test webhook function to verify HTTP outcalls work
// #[ic_cdk::update]
// async fn test_webhook(webhook_url: String, message: String) -> String {
//     // Simple payload for testing
//     #[derive(Serialize, Deserialize)]
//     struct TestPayload {
//         canister_id: String,
//         message: String,
//         timestamp: u64,
//     }

//     let payload = TestPayload {
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
//         url: webhook_url,
//         method: HttpMethod::POST,
//         body: Some(json_payload.into_bytes()),
//         max_response_bytes: None,
//         // Use TransformContext.from_name as shown in outcall.txt
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


// ic_cdk::export_candid!();


use crate::repositories::{HttpRequest, HttpUpdateRequest};
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};

mod controllers;
mod repositories;
mod services;
mod utils;

ic_cdk::export_candid!();
