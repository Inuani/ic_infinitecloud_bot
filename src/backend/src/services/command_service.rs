use crate::repositories::KeyboardDirectoryBuilder;
use crate::repositories::{ChatId, ChatSession, ChatSessionAction, Command, FileSystem};
use crate::services::{ChatSessionService, ChatSessionServiceImpl};
use crate::utils::messages;
use crate::utils::MessageParams;

pub trait CommandService {
    fn handle_command(
        &self,
        chat_id: &ChatId,
        command: &Command,
        chat_session: &mut ChatSession,
        filesystem: &FileSystem,
    ) -> Result<MessageParams, String>;
}

pub struct CommandServiceImpl {}

impl Default for CommandServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl CommandService for CommandServiceImpl {
    fn handle_command(
        &self,
        chat_id: &ChatId,
        command: &Command,
        chat_session: &mut ChatSession,
        filesystem: &FileSystem,
    ) -> Result<MessageParams, String> {
        let mut send_message_params = MessageParams::new_send(chat_id.clone());

        match command {
            Command::Start => {
                send_message_params.set_text(messages::start_message(None));
                Ok(send_message_params)
            }
            Command::Help => {
                send_message_params.set_text(messages::help_message());
                Ok(send_message_params)
            }
            Command::Info => {
                send_message_params.set_text(messages::info_message());
                Ok(send_message_params)
            }
            Command::MkDir => {
                chat_session.set_action(ChatSessionAction::MkDir(None));

                send_message_params
                    .set_text(messages::mkdir_message(chat_session.current_path_string()));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_current_dir_button()
                        .build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::Explorer => {
                chat_session.set_action(ChatSessionAction::Explorer);

                send_message_params.set_text(messages::explorer_message(
                    chat_session.current_path_string(),
                ));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_files()? // Include files in the keyboard
                        .build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::RenameFile => {
                chat_session.set_action(ChatSessionAction::RenameFile(None));

                send_message_params.set_text(messages::rename_file_message(
                    chat_session.current_path_string(),
                ));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_files()? // Include files in the keyboard
                        .build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::MoveFile => {
                chat_session.set_action(ChatSessionAction::MoveFile(None));
                send_message_params.set_text(messages::move_file_select_file_message(
                    chat_session.current_path_string(),
                ));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_files()?
                        .build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::DeleteDir => {
                chat_session.set_action(ChatSessionAction::DeleteDir);
                send_message_params.set_text(messages::delete_dir_message(
                    chat_session.current_path_string(),
                ));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?.build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::DeleteFile => {
                chat_session.set_action(ChatSessionAction::DeleteFile);
                send_message_params.set_text(messages::delete_file_message(
                    chat_session.current_path_string(),
                ));

                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_files()?
                        .build();

                send_message_params.set_inline_keyboard_markup(keyboard);

                Ok(send_message_params)
            }
            Command::Automation => {
                // Create the initial response message
                send_message_params.set_text(format!(
                    "Preparing to run automation for path:\n{}\n\nProcessing...",
                    chat_session.current_path_string()
                ));

                // Set action to Explorer to keep UI consistent
                chat_session.set_action(ChatSessionAction::Explorer);

                // Add keyboard for navigation
                let keyboard =
                    KeyboardDirectoryBuilder::new(filesystem, chat_session.current_path())?
                        .with_files()?
                        .build();
                send_message_params.set_inline_keyboard_markup(keyboard);

                // Spawn an async task to run the automation
                let chat_id_clone = chat_id.clone();
                let path_clone = chat_session.current_path().to_path_buf();
                let fs_clone = filesystem.clone();

                ic_cdk::spawn(async move {
                    let service = ChatSessionServiceImpl::default();
                    let _ = service
                        .handle_automation_command(&chat_id_clone, &path_clone, &fs_clone)
                        .await;
                });

                Ok(send_message_params)
            }
        }
    }
}
