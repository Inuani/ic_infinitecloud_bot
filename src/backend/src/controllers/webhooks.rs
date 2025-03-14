use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{query, update};
use crate::{
    repositories::{ChatId, ChatSessionRepositoryImpl, FilesystemRepositoryImpl},
    services::{
        webhook_service, ChatSessionService, ChatSessionServiceImpl, FilesystemServiceImpl
    },
};

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

#[update]
async fn process_file_automation(chat_id: u64, path: String) -> String {
    // Initialize services - using IMPLEMENTATION types, not traits
    let chat_session_service = ChatSessionServiceImpl::<ChatSessionRepositoryImpl, FilesystemServiceImpl<FilesystemRepositoryImpl>>::default();
    
    // Convert parameters
    let chat_id = ChatId(chat_id);
    let path_buf = std::path::PathBuf::from(path);
    
    // Get filesystem using the getter
    let fs = chat_session_service.get_filesystem_service()
        .get_or_create_filesystem(&chat_id);
    
    // Process the automation request
    match chat_session_service.handle_automation_command(&chat_id, &path_buf, &fs).await {
        Ok(_) => "Automation processed successfully".to_string(),
        Err(e) => format!("Error processing automation: {}", e),
    }
}