// use frankenstein::Message;
use std::path::{Path, PathBuf};

use crate::{
    custom_print,
    repositories::{
        ChatId, ChatSession, ChatSessionAction,
        FileSystem, FileSystemNode, KeyboardDirectoryBuilder, MessageId,
    },
    services::webhook_service::{send_directory_automation, send_file_automation, FileInfo},
    utils::{
        messages::{
            self,
            back_inline_keyboard, cannot_delete_non_empty_dir_message, create_file_message,
            created_directory_success_message, created_file_success_message,
            deleted_dir_success_message, deleted_file_success_message,
            explorer_file_message, explorer_message,
            moved_file_success_message, renamed_file_success_message,
        },
        MessageParams
    },
};

pub trait FileOperationService {
    fn process_file_message(
        &self,
        chat_session: &mut ChatSession,
        fs: &FileSystem,
        chat_id: ChatId,
        message_id: MessageId,
        file_size: Option<u64>,
        mime_type: Option<String>,
    ) -> Result<MessageParams, String>;

    fn handle_mkdir_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        text: String,
    ) -> Result<MessageParams, String>;

    fn handle_save_file_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        file_node: FileSystemNode,
        file_name: String,
    ) -> Result<MessageParams, String>;

    fn handle_rename_file_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        new_file_name: String,
    ) -> Result<MessageParams, String>;

    fn handle_move_file_action(
        &self,
        chat_id: &ChatId,
        from_path: &Path,
        to_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String>;

    fn handle_delete_file_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        current_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String>;

    fn handle_delete_dir_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        current_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String>;

    fn handle_explorer_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        fs: &FileSystem,
    ) -> Result<MessageParams, String>;

    async fn handle_file_automation(
        &self,
        chat_id: &ChatId,
        current_path: &PathBuf,
        fs: &FileSystem,
    ) -> Result<MessageParams, String>;
}

pub struct FileOperationServiceImpl {}

impl Default for FileOperationServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperationServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileOperationService for FileOperationServiceImpl {
    fn process_file_message(
        &self,
        chat_session: &mut ChatSession,
        fs: &FileSystem,
        chat_id: ChatId,
        message_id: MessageId,
        file_size: Option<u64>,
        mime_type: Option<String>,
    ) -> Result<MessageParams, String> {
        // We reset the chat session to start the flow of saving a new file
        chat_session.reset();

        let file_node = FileSystemNode::new_file(message_id, file_size.unwrap_or(0), mime_type);
        chat_session.set_action(ChatSessionAction::SaveFile(Some(file_node), None));

        let mut send_message_params = MessageParams::new_send(chat_id.clone());
        send_message_params.set_text(create_file_message(chat_session.current_path_string()));
        let keyboard = KeyboardDirectoryBuilder::new(fs, chat_session.current_path())?
            .with_current_dir_button()
            .build();
        send_message_params.set_inline_keyboard_markup(keyboard);

        Ok(send_message_params)
    }

    fn handle_mkdir_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        dir_name: String,
    ) -> Result<MessageParams, String> {
        let dir_path = chat_session.current_path().join(&dir_name);
        fs.mkdir(&dir_path)?;
        chat_session.reset();

        let mut send_message_params = MessageParams::new_send(chat_id.clone());
        send_message_params.set_text(created_directory_success_message(
            dir_name,
            dir_path.to_string_lossy().to_string(),
        ));
        Ok(send_message_params)
    }

    fn handle_save_file_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        file_node: FileSystemNode,
        file_name: String,
    ) -> Result<MessageParams, String> {
        let dir_path = chat_session.current_path();
        let file_path = dir_path.join(file_name);
        let final_file_path = fs.create_file_from_node(&file_path, file_node)?;
        let mut send_message_params = MessageParams::new_send(chat_id.clone());
        send_message_params.set_text(created_file_success_message(
            final_file_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            dir_path.to_string_lossy().to_string(),
        ));
        Ok(send_message_params)
    }

    fn handle_rename_file_action(
        &self,
        chat_id: &ChatId,
        chat_session: &mut ChatSession,
        fs: &mut FileSystem,
        new_file_name: String,
    ) -> Result<MessageParams, String> {
        let from_path = chat_session.current_path();
        let mut to_path = from_path.clone();
        to_path.set_file_name(&new_file_name);
        fs.mv(from_path, &to_path)?;
        let mut send_message_params = MessageParams::new_send(chat_id.clone());
        send_message_params.set_text(renamed_file_success_message(
            from_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            new_file_name,
            from_path.parent().unwrap().to_string_lossy().to_string(),
        ));
        Ok(send_message_params)
    }

    fn handle_move_file_action(
        &self,
        chat_id: &ChatId,
        from_path: &Path,
        to_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String> {
        let file_name = from_path.file_name().unwrap().to_string_lossy().to_string();
        let full_to_path = to_path.join(&file_name);
        fs.mv(from_path, &full_to_path)?;

        let mut message_params = MessageParams::new_send(chat_id.clone());
        message_params.set_text(moved_file_success_message(
            file_name,
            from_path.to_string_lossy().to_string(),
            full_to_path.to_string_lossy().to_string(),
        ));
        Ok(message_params)
    }

    fn handle_delete_file_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        current_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String> {
        fs.remove_node(path)?;

        let file_name = path
            .file_name()
            .ok_or_else(|| "File name not found".to_string())?
            .to_string_lossy()
            .to_string();

        let mut send_message_params = MessageParams::new_send(chat_id.clone());
        send_message_params.set_text(deleted_file_success_message(
            file_name,
            current_path.to_string_lossy().to_string(),
        ));
        Ok(send_message_params)
    }

    fn handle_delete_dir_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        current_path: &Path,
        fs: &mut FileSystem,
    ) -> Result<MessageParams, String> {
        let node = fs.get_node(path)?;

        let mut message_params = MessageParams::new_send(chat_id.clone());

        if node.is_directory() {
            let is_empty = match node {
                FileSystemNode::Directory { nodes, .. } => nodes.is_empty(),
                _ => false, // This shouldn't happen
            };

            if is_empty {
                // If directory is empty, delete it
                fs.remove_node(path)?;

                let dir_name = path
                    .file_name()
                    .ok_or_else(|| "Directory name not found".to_string())?
                    .to_string_lossy()
                    .to_string();

                message_params.set_text(deleted_dir_success_message(
                    dir_name,
                    current_path.to_string_lossy().to_string(),
                ));
            } else {
                // If directory is not empty, show error
                message_params.set_text(cannot_delete_non_empty_dir_message(
                    path.file_name()
                        .ok_or_else(|| "Directory name not found".to_string())?
                        .to_string_lossy()
                        .to_string(),
                ));

                // Keep the action and path so user can navigate back
                let keyboard = back_inline_keyboard();
                message_params.set_inline_keyboard_markup(keyboard);
            }
        } else {
            // If it's not a directory, show error
            message_params.set_text("This is not a directory.".to_string());
            let keyboard = back_inline_keyboard();
            message_params.set_inline_keyboard_markup(keyboard);
        }

        Ok(message_params)
    }

    fn handle_explorer_action(
        &self,
        chat_id: &ChatId,
        path: &Path,
        fs: &FileSystem,
    ) -> Result<MessageParams, String> {
        let node = fs.get_node(path)?;

        if node.is_directory() {
            let mut message_params = MessageParams::new_send(chat_id.clone());
            message_params.set_text(explorer_message(path.to_string_lossy().to_string()));

            let keyboard = KeyboardDirectoryBuilder::new(fs, path)?
                .with_files()?
                .build();
            message_params.set_inline_keyboard_markup(keyboard);
            Ok(message_params)
        } else {
            // Get file info
            let message_id = node
                .file_message_id()
                .ok_or_else(|| "Message id not found".to_string())?;
            let file_name = path
                .file_name()
                .ok_or_else(|| "File name not found".to_string())?
                .to_string_lossy()
                .to_string();
            let parent_path = path
                .parent()
                .ok_or_else(|| "Parent path not found".to_string())?
                .to_string_lossy()
                .to_string();

            let mut send_message_params = MessageParams::new_send(chat_id.clone());
            send_message_params.set_text(explorer_file_message(file_name, parent_path));
            send_message_params.set_reply_to_message_id(message_id)?;

            Ok(send_message_params)
        }
    }

    async fn handle_file_automation(
        &self,
        chat_id: &ChatId,
        current_path: &PathBuf,
        fs: &FileSystem,
    ) -> Result<MessageParams, String> {
        custom_print!(
            "Starting file automation for chat_id: {}, path: {:?}",
            chat_id,
            current_path
        );

        let node = fs.get_node(current_path)?;

        let mut send_message_params = MessageParams::new_send(chat_id.clone());

        match node {
            FileSystemNode::File {
                message_id,
                size,
                mime_type,
                ..
            } => {
                custom_print!("Processing automation for file at path: {:?}", current_path);
                let file_name = current_path
                    .file_name()
                    .ok_or_else(|| "File name not found".to_string())?
                    .to_string_lossy()
                    .to_string();

                let file_info = FileInfo {
                    name: file_name.clone(),
                    path: current_path.to_string_lossy().to_string(),
                    message_id: *message_id,
                    mime_type: mime_type.clone(),
                    size: Some(*size),
                };

                // Pass the file_info to send_file_automation
                let result = send_file_automation(
                    chat_id.0,
                    Some(*message_id),
                    current_path.clone(),
                    Some(file_info),
                )
                .await;

                send_message_params.set_text(
                    if result.contains("error") || result.contains("Failed") {
                        messages::automation_error_message(result)
                    } else {
                        messages::automation_file_message(
                            file_name,
                            current_path.to_string_lossy().to_string(),
                        )
                    },
                );
            }
            FileSystemNode::Directory { .. } => {
                custom_print!(
                    "Processing automation for directory at path: {:?}",
                    current_path
                );

                // Get all files in directory
                let files_result = if let FileSystemNode::Directory { nodes, .. } = node {
                    // Collect file information for all files in directory
                    let mut file_infos = Vec::new();

                    for (path, node) in nodes {
                        if let FileSystemNode::File {
                            message_id,
                            size,
                            mime_type,
                            ..
                        } = node
                        {
                            let full_path = current_path.join(path);
                            let file_name = path.to_string_lossy().to_string();

                            file_infos.push(FileInfo {
                                name: file_name,
                                path: full_path.to_string_lossy().to_string(),
                                message_id: *message_id,
                                mime_type: mime_type.clone(),
                                size: Some(*size),
                            });
                        }
                    }

                    // Log the JSON separately (not in a match expression)
                    if let Ok(json) = serde_json::to_string(&file_infos) {
                        custom_print!("Found files: {}", json);
                    } else {
                        custom_print!("Error serializing file_infos");
                    }

                    // Use the special send_directory_automation function
                    send_directory_automation(chat_id.0, current_path.clone(), file_infos).await
                } else {
                    "Error: Not a directory".to_string()
                };

                send_message_params.set_text(
                    if files_result.contains("error") || files_result.contains("Failed") {
                        messages::automation_error_message(files_result)
                    } else {
                        messages::automation_message(current_path.to_string_lossy().to_string())
                    },
                );
            }
        }

        Ok(send_message_params)
    }
}

pub fn action_not_supported_error() -> Result<MessageParams, String> {
    Err("current action not supported by this action".to_string())
}