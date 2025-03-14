use candid::Principal;
use ic_cdk::{caller, query, update};
use std::path::PathBuf;

use crate::{
    repositories::{ChatId, ChatSessionRepositoryImpl, FilesystemRepositoryImpl},
    services::{
        AccessControlService, AccessControlServiceImpl, ChatSessionService, ChatSessionServiceImpl,
        FilesystemServiceImpl,
    },
};

#[query]
fn get_chat_sessions_count() -> u32 {
    let calling_principal = caller();

    ChatSessionController::default().get_chat_sessions_count(calling_principal)
}

// New update function for automation
#[update]
async fn run_automation(chat_id_value: u64, path: String) -> String {
    let chat_id = ChatId(chat_id_value);
    
    let controller = ChatSessionController::default();
    controller.run_automation_for_path(chat_id, path).await
}

////////////////////
// use ic_cdk::update;

// #[update]
// async fn trigger_webhook(webhook_url: String, message: String) -> String {
//     // Call our test_webhook function
//     crate::test_webhook(webhook_url, message).await
// }
////////////////////


struct ChatSessionController<A: AccessControlService, C: ChatSessionService> {
    access_control_service: A,
    chat_session_service: C,
}

impl Default
    for ChatSessionController<
        AccessControlServiceImpl,
        ChatSessionServiceImpl<
            ChatSessionRepositoryImpl,
            FilesystemServiceImpl<FilesystemRepositoryImpl>,
        >,
    >
{
    fn default() -> Self {
        Self::new(
            AccessControlServiceImpl::default(),
            ChatSessionServiceImpl::default(),
        )
    }
}

impl<A: AccessControlService, C: ChatSessionService> ChatSessionController<A, C> {
    fn new(access_control_service: A, chat_session_service: C) -> Self {
        Self {
            access_control_service,
            chat_session_service,
        }
    }

    fn get_chat_sessions_count(&self, calling_principal: Principal) -> u32 {
        self.access_control_service
            .assert_caller_is_controller(&calling_principal);

        self.chat_session_service.get_chat_sessions_count()
    }
    
    async fn run_automation_for_path(&self, chat_id: ChatId, path: String) -> String {
        // Convert path
        let path_buf = PathBuf::from(path);
        
        // Get filesystem using the getter
        let fs = self.chat_session_service.get_filesystem_service()
            .get_or_create_filesystem(&chat_id);
        
        // Process automation
        match self.chat_session_service.handle_automation_command(&chat_id, &path_buf, &fs).await {
            Ok(_) => "Automation processed successfully".to_string(),
            Err(e) => format!("Error: {}", e),
        }
    }
}