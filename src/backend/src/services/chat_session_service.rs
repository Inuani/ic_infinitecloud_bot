use frankenstein::{CallbackQuery, MaybeInaccessibleMessage, Message};
use std::path::PathBuf;

use crate::{
    custom_print,
    repositories::{
        with_clear_action_on_error, ChatId, ChatSession, ChatSessionAction, ChatSessionRepository,
        ChatSessionRepositoryImpl, ChatSessionWaitReply, Command, FileSystem,
        FilesystemRepositoryImpl, KeyboardDirectoryBuilder
    },
    services::{
        CommandService, CommandServiceImpl, FileOperationService, FileOperationServiceImpl,
    },
    utils::{
        filesystem::root_path,
        MessageParams, TG_FILE_MIME_TYPE_PREFIX,
    },
};

use super::{FilesystemService, FilesystemServiceImpl};

pub trait ChatSessionService {
    fn get_or_create_chat_session(&self, chat_id: &ChatId) -> ChatSession;

    fn update_chat_session(&self, chat_id: ChatId, chat_session: ChatSession);

    fn get_chat_sessions_count(&self) -> u32;

    fn handle_update_content_message(
        &self,
        chat_id: ChatId,
        msg: Message,
    ) -> Result<MessageParams, String>;

    fn handle_update_content_callback_query(
        &self,
        chat_id: ChatId,
        query: CallbackQuery,
    ) -> Result<MessageParams, String>;

    async fn handle_automation_command(
        &self,
        chat_id: &ChatId,
        current_path: &PathBuf,
        fs: &FileSystem,
    ) -> Result<MessageParams, String>;

    fn get_filesystem_service(&self) -> &dyn FilesystemService;
}

pub struct ChatSessionServiceImpl<T: ChatSessionRepository, F: FilesystemService> {
    chat_session_repository: T,
    filesystem_service: F,
    command_service: CommandServiceImpl,
    file_operation_service: FileOperationServiceImpl,
}

impl Default
    for ChatSessionServiceImpl<
        ChatSessionRepositoryImpl,
        FilesystemServiceImpl<FilesystemRepositoryImpl>,
    >
{
    fn default() -> Self {
        Self::new(
            ChatSessionRepositoryImpl::default(),
            FilesystemServiceImpl::default(),
            CommandServiceImpl::default(),
            FileOperationServiceImpl::default(),
        )
    }
}

impl<T: ChatSessionRepository, F: FilesystemService> ChatSessionService
    for ChatSessionServiceImpl<T, F>
{
    fn get_filesystem_service(&self) -> &dyn FilesystemService {
        &self.filesystem_service
    }

    fn get_or_create_chat_session(&self, chat_id: &ChatId) -> ChatSession {
        match self
            .chat_session_repository
            .get_chat_session_by_chat_id(chat_id)
        {
            Some(chat_session) => chat_session,
            None => {
                let chat_session = ChatSession::default();
                self.chat_session_repository
                    .set_chat_session_by_chat_id(chat_id.clone(), chat_session.clone());
                chat_session
            }
        }
    }

    fn update_chat_session(&self, chat_id: ChatId, chat_session: ChatSession) {
        self.chat_session_repository
            .set_chat_session_by_chat_id(chat_id, chat_session);
    }

    fn get_chat_sessions_count(&self) -> u32 {
        self.chat_session_repository.get_chat_session_count() as u32
    }

    fn handle_update_content_message(
        &self,
        chat_id: ChatId,
        msg: Message,
    ) -> Result<MessageParams, String> {
        let mut fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
        let mut chat_session = self.get_or_create_chat_session(&chat_id);

        let res = with_clear_action_on_error(&mut chat_session, |cs| {
            let current_path = cs.current_path().clone();
            custom_print!(
                "UpdateContent::Message: chat_id: {:?}, current_path: {:?}, current_action: {:?}, message.text: {:?}",
                chat_id, current_path, cs.action(), msg.text
            );

            match Command::try_from(msg.clone()) {
                Ok(command) => {     
                    let current_path = cs.current_path().clone();
                    // when receiving a command, we want to reset the chat session
                    cs.reset();

                    match command {
                        Command::DeleteDir | Command::DeleteFile | Command::Automation => {
                            // These commands need to preserve the current path
                            cs.set_current_path(current_path.clone());
                        },
                        _ => {}
                    }
                    return self.command_service.handle_command(&chat_id, &command, cs, &fs);
                }
                Err(_) => {
                    if let Some(text) = msg.text {
                        return match cs.action() {
                            Some(current_action) => match current_action {
                                ChatSessionAction::MkDir(Some(
                                    ChatSessionWaitReply::DirectoryName,
                                )) => {
                                    self.file_operation_service.handle_mkdir_action(&chat_id, cs, &mut fs, text)
                                }
                                ChatSessionAction::SaveFile(
                                    Some(file_node),
                                    Some(ChatSessionWaitReply::FileName),
                                ) => {
                                    self.file_operation_service.handle_save_file_action(&chat_id, cs, &mut fs, file_node, text)
                                }
                                ChatSessionAction::RenameFile(Some(
                                    ChatSessionWaitReply::FileName,
                                )) => {
                                    self.file_operation_service.handle_rename_file_action(&chat_id, cs, &mut fs, text)
                                }
                                _ => Ok(MessageParams::generic_error(chat_id.clone())),
                            },
                            None => self.file_operation_service.process_file_message(
                                cs,
                                &fs,
                                chat_id.clone(),
                                msg.message_id,
                                Some(text.len().try_into().unwrap()),
                                Some(format!("{TG_FILE_MIME_TYPE_PREFIX}text")),
                            ),
                        };
                    }

                    if let Some(document) = msg.document {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            document.file_size,
                            document.mime_type,
                        );
                    }

                    if let Some(photos) = msg.photo {
                        let photo = photos.first().unwrap();
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            photo.file_size,
                            Some("jpeg".to_string()),
                        );
                    }

                    if let Some(video) = msg.video {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            video.file_size,
                            video.mime_type,
                        );
                    }

                    if let Some(video_note) = msg.video_note {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            video_note.file_size,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}video_note")),
                        );
                    }

                    if let Some(audio) = msg.audio {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            audio.file_size,
                            audio.mime_type,
                        );
                    }

                    if let Some(voice) = msg.voice {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            voice.file_size,
                            voice.mime_type,
                        );
                    }

                    if let Some(sticker) = msg.sticker {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            sticker.file_size,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}sticker")),
                        );
                    }

                    if msg.contact.is_some() {
                        return self.file_operation_service.process_file_message(
                            cs,
                            &fs,
                            chat_id.clone(),
                            msg.message_id,
                            None,
                            Some(format!("{TG_FILE_MIME_TYPE_PREFIX}contact")),
                        );
                    }

                    Ok(MessageParams::generic_error(chat_id.clone()))
                }
            }
        });

        self.save_chat_session_and_filesystem(chat_id, chat_session, fs);

        res
    }

    fn handle_update_content_callback_query(
        &self,
        chat_id: ChatId,
        query: CallbackQuery,
    ) -> Result<MessageParams, String> {
        let mut fs = self.filesystem_service.get_or_create_filesystem(&chat_id);
        let mut chat_session = self.get_or_create_chat_session(&chat_id);

        let res = with_clear_action_on_error(&mut chat_session, |cs| {
            let action = query
                .data
                .ok_or_else(|| "Data not found in callback query".to_string())?
                .into();
            let message_id = match query
                .message
                .ok_or_else(|| "Message not found in callback query".to_string())?
            {
                MaybeInaccessibleMessage::Message(msg) => msg.message_id,
                MaybeInaccessibleMessage::InaccessibleMessage(msg) => msg.message_id,
            };

            custom_print!(
                "UpdateContent::CallbackQuery: chat_id: {:?}, current_path: {:?}, current_action: {:?}, action: {:?}",
                chat_id,
                cs.current_path(),
                cs.action(),
                action
            );

            let mut edit_message_params = MessageParams::new_edit(chat_id.clone(), message_id);
            let current_action = cs.action().ok_or_else(|| {
                "UpdateContent::CallbackQuery: No action in chat session".to_string()
            })?;

            match action {
                ChatSessionAction::CurrentDir => match current_action {
                    ChatSessionAction::MkDir(None) => {
                        cs.set_action(ChatSessionAction::MkDir(Some(
                            ChatSessionWaitReply::DirectoryName,
                        )));
                        edit_message_params.set_text(crate::utils::messages::ask_directory_name_message(cs.current_path_string()));
                        edit_message_params.set_inline_keyboard_markup(crate::utils::messages::back_inline_keyboard());

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(file_node), None) => {
                        cs.set_action(ChatSessionAction::SaveFile(
                            Some(file_node),
                            Some(ChatSessionWaitReply::FileName),
                        ));
                        edit_message_params.set_text(crate::utils::messages::ask_file_name_message(cs.current_path_string()));
                        edit_message_params.set_inline_keyboard_markup(crate::utils::messages::back_inline_keyboard());

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::MoveFile(Some(from_path)) => {
                        let to_path = cs.current_path().clone();
                        self.file_operation_service.handle_move_file_action(&chat_id, &from_path, &to_path, &mut fs)
                            .map(|params| {
                                // Transfer message text to edit message
                                edit_message_params.set_text(params.json_value().unwrap()["text"].as_str().unwrap().to_string());
                                edit_message_params
                            })
                    }
                    _ => crate::services::file_operation_service::action_not_supported_error(),
                },
                ChatSessionAction::ParentDir => {
                    let current_path = cs.current_path().clone();
                    let root_path = root_path();
                    let parent_path = current_path.parent().unwrap_or(root_path.as_ref());

                    match current_action {
                        ChatSessionAction::Explorer => {
                            let node = fs.get_node(parent_path)?;

                            if node.is_directory() {
                                cs.set_current_path(parent_path.to_path_buf());
                                edit_message_params.set_text(crate::utils::messages::explorer_message(cs.current_path_string()));

                                let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                    .with_files()?
                                    .build();
                                edit_message_params.set_inline_keyboard_markup(keyboard);
                            } else {
                                // should never happen
                                return Err("Parent is not a directory".to_string());
                            }

                            Ok(edit_message_params)
                        }
                        ChatSessionAction::MkDir(_) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params.set_text(crate::utils::messages::mkdir_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_current_dir_button()
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::SaveFile(Some(_), None) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params.set_text(crate::utils::messages::create_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_current_dir_button()
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::RenameFile(_) => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params.set_text(crate::utils::messages::rename_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        ChatSessionAction::MoveFile(from_path) => {
                            cs.set_current_path(parent_path.to_path_buf());

                            let (message_text, keyboard) = match from_path {
                                Some(from_path) => {
                                    let msg = crate::utils::messages::move_file_select_destination_message(
                                        from_path.to_string_lossy().to_string(),
                                    );
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_current_dir_button()
                                            .build();
                                    (msg, keyboard)
                                }
                                None => {
                                    let msg =
                                    crate::utils::messages::move_file_select_file_message(cs.current_path_string());
                                    let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                        .with_files()?
                                        .build();
                                    (msg, keyboard)
                                }
                            };
                            edit_message_params.set_text(message_text);
                            edit_message_params.set_inline_keyboard_markup(keyboard);

                            Ok(edit_message_params)
                        }
                        ChatSessionAction::DeleteFile => {
                            cs.set_current_path(parent_path.to_path_buf());
                            edit_message_params.set_text(crate::utils::messages::delete_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, parent_path)?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        }
                        _ => crate::services::file_operation_service::action_not_supported_error(),
                    }
                },
                ChatSessionAction::FileOrDir(path) => match current_action {
                    ChatSessionAction::Explorer => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());
                            edit_message_params.set_text(crate::utils::messages::explorer_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        } else {
                            // Use the FileOperationService to handle the file explorer
                            self.file_operation_service.handle_explorer_action(&chat_id, &path, &fs)
                        }
                    }
                    ChatSessionAction::MkDir(_) => {
                        cs.set_current_path(path.clone());
                        edit_message_params.set_text(crate::utils::messages::mkdir_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);
                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(_), None) => {
                        cs.set_current_path(path.clone());
                        edit_message_params.set_text(crate::utils::messages::create_file_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, &path)?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);
                        Ok(edit_message_params)
                    }
                    ChatSessionAction::RenameFile(None) => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());
                            edit_message_params.set_text(crate::utils::messages::rename_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        } else {
                            // Handle rename file operation
                            let message_id = node
                                .file_message_id()
                                .ok_or_else(|| "Message id not found".to_string())?;
                            let file_name = path
                                .file_name()
                                .ok_or_else(|| "File name not found".to_string())?
                                .to_string_lossy()
                                .to_string();

                            let mut send_message_params = MessageParams::new_send(chat_id.clone());
                            send_message_params.set_text(crate::utils::messages::ask_rename_file_message(
                                file_name,
                                cs.current_path_string(),
                            ));
                            send_message_params.set_reply_to_message_id(message_id)?;

                            cs.set_current_path(path);
                            cs.set_action(ChatSessionAction::RenameFile(Some(
                                ChatSessionWaitReply::FileName,
                            )));

                            return Ok(send_message_params);
                        }
                    }
                    ChatSessionAction::MoveFile(from_path) => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());

                            let (message_text, keyboard) = match from_path {
                                Some(from_path) => {
                                    let msg = crate::utils::messages::move_file_select_destination_message(
                                        from_path.to_string_lossy().to_string(),
                                    );
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_current_dir_button()
                                            .build();
                                    (msg, keyboard)
                                }
                                None => {
                                    let msg =
                                    crate::utils::messages::move_file_select_file_message(cs.current_path_string());
                                    let keyboard =
                                        KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                            .with_files()?
                                            .build();
                                    (msg, keyboard)
                                }
                            };
                            edit_message_params.set_text(message_text);
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        } else {
                            // Handle move file selection
                            let message_id = node
                                .file_message_id()
                                .ok_or_else(|| "Message id not found".to_string())?;
                            let from_path = path.clone();

                            cs.set_current_path(root_path());

                            let mut send_message_params = MessageParams::new_send(chat_id.clone());
                            send_message_params.set_text(crate::utils::messages::move_file_select_destination_message(
                                from_path.to_string_lossy().to_string(),
                            ));
                            send_message_params.set_reply_to_message_id(message_id)?;
                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_current_dir_button()
                                .build();
                            send_message_params.set_inline_keyboard_markup(keyboard);

                            cs.set_action(ChatSessionAction::MoveFile(Some(from_path)));

                            return Ok(send_message_params);
                        }
                    }
                    ChatSessionAction::DeleteFile => {
                        let node = fs.get_node(&path)?;

                        if node.is_directory() {
                            cs.set_current_path(path.clone());
                            edit_message_params.set_text(crate::utils::messages::delete_file_message(cs.current_path_string()));

                            let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                                .with_files()?
                                .build();
                            edit_message_params.set_inline_keyboard_markup(keyboard);
                            Ok(edit_message_params)
                        } else {
                            // Delete the file using FileOperationService
                            let result = self.file_operation_service.handle_delete_file_action(
                                &chat_id, 
                                &path, 
                                cs.current_path(),
                                &mut fs
                            )?;
                            
                            // Convert send message to edit message format
                            if let Ok(value) = result.json_value() {
                                edit_message_params.set_text(value["text"].as_str().unwrap().to_string());
                            }
                            
                            cs.reset();
                            
                            Ok(edit_message_params)
                        }
                    }
                    ChatSessionAction::DeleteDir => {
                        // Use FileOperationService to handle delete directory
                        let result = self.file_operation_service.handle_delete_dir_action(
                            &chat_id,
                            &path,
                            cs.current_path(),
                            &mut fs
                        )?;
                        
                        // Convert send message to edit message format
                        if let Ok(value) = result.json_value() {
                            edit_message_params.set_text(value["text"].as_str().unwrap().to_string());
                            
                            // If keyboard is set in the result, copy it
                            if let Some(reply_markup) = value.get("reply_markup") {
                                if let Some(keyboard) = reply_markup.get("inline_keyboard") {
                                    // Parse the keyboard and set it
                                    let keyboard_str = keyboard.to_string();
                                    let keyboard: frankenstein::InlineKeyboardMarkup = serde_json::from_str(&keyboard_str).unwrap();
                                    edit_message_params.set_inline_keyboard_markup(keyboard);
                                }
                            }
                        }
                        
                        // Only reset if directory was successfully deleted
                        if !edit_message_params.json_value().unwrap()["text"].as_str().unwrap().contains("cannot") {
                            cs.reset();
                        }
                        
                        Ok(edit_message_params)
                    }
                    _ => crate::services::file_operation_service::action_not_supported_error(),
                },
                ChatSessionAction::Back => match current_action {
                    ChatSessionAction::MkDir(Some(_)) => {
                        cs.set_action(ChatSessionAction::MkDir(None));

                        edit_message_params.set_text(crate::utils::messages::mkdir_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);

                        Ok(edit_message_params)
                    }
                    ChatSessionAction::SaveFile(Some(file_node), Some(_)) => {
                        cs.set_action(ChatSessionAction::SaveFile(Some(file_node), None));

                        edit_message_params.set_text(crate::utils::messages::create_file_message(cs.current_path_string()));

                        let keyboard = KeyboardDirectoryBuilder::new(&fs, cs.current_path())?
                            .with_current_dir_button()
                            .build();
                        edit_message_params.set_inline_keyboard_markup(keyboard);

                        Ok(edit_message_params)
                    }
                    _ => crate::services::file_operation_service::action_not_supported_error(),
                },
                ChatSessionAction::DeleteDir
                | ChatSessionAction::Explorer
                | ChatSessionAction::MoveFile(_)
                | ChatSessionAction::DeleteFile
                | ChatSessionAction::SaveFile(_, _)
                | ChatSessionAction::RenameFile(_)
                | ChatSessionAction::MkDir(_) => Err("invalid action".to_string()),
            }
        });

        self.save_chat_session_and_filesystem(chat_id, chat_session, fs);

        res
    }

    async fn handle_automation_command(
        &self,
        chat_id: &ChatId,
        current_path: &PathBuf,
        fs: &FileSystem,
    ) -> Result<MessageParams, String> {
        self.file_operation_service.handle_file_automation(chat_id, current_path, fs).await
    }
}

impl<T: ChatSessionRepository, F: FilesystemService> ChatSessionServiceImpl<T, F> {
    fn new(
        chat_session_repository: T, 
        filesystem_service: F, 
        command_service: CommandServiceImpl,
        file_operation_service: FileOperationServiceImpl,
    ) -> Self {
        Self {
            chat_session_repository,
            filesystem_service,
            command_service,
            file_operation_service,
        }
    }

    fn save_chat_session_and_filesystem(
        &self,
        chat_id: ChatId,
        chat_session: ChatSession,
        filesystem: FileSystem,
    ) {
        self.update_chat_session(chat_id.clone(), chat_session);
        self.filesystem_service
            .update_filesystem(&chat_id, filesystem);
    }
}