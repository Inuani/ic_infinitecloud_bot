use frankenstein::{Message, MessageEntityType};

use crate::custom_print;

#[derive(Debug)]
pub enum Command {
    Start,
    Help,
    Info,
    MkDir,
    Explorer,
    RenameFile,
    MoveFile,
    DeleteDir,
    DeleteFile,
    Automation,
}

impl TryFrom<Message> for Command {
    type Error = String;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let text_command = message
            .text
            .ok_or_else(|| "No text in message".to_string())?;

        custom_print!("Checking for command in message: {}", text_command);

        let entity = message
            .entities
            .and_then(|e| e.first().cloned())
            .ok_or_else(|| "No entities in message".to_string())?;

        custom_print!("Entity type: {:?}", entity.type_field);

        if entity.type_field != MessageEntityType::BotCommand {
            return Err("No bot command in message".to_string());
        }

        let offset = entity.offset as usize;
        let length = entity.length as usize;
        let command = &text_command[offset..offset + length];

        custom_print!("Extracted command: {}", command);

        match command {
            "/start" => Ok(Command::Start),
            "/help" => Ok(Command::Help),
            "/info" => Ok(Command::Info),
            "/mkdir" => Ok(Command::MkDir),
            "/explorer" => Ok(Command::Explorer),
            "/rename_file" => Ok(Command::RenameFile),
            "/move_file" => Ok(Command::MoveFile),
            "/delete_dir" => Ok(Command::DeleteDir),
            "/delete_file" => Ok(Command::DeleteFile),
            "/automation" => Ok(Command::Automation),
            _ => Err("Unknown command".to_string()),
        }
    }
}
