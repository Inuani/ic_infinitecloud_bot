use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{query, update};
use crate::services::webhook_service;

// Transform function for HTTP responses
#[query]
fn transform(args: TransformArgs) -> HttpResponse {
    webhook_service::transform_response(args)
}

// Function to send a webhook
#[update]
async fn send_webhook(message: String) -> String {
    webhook_service::send_to_webhook(message).await
}