use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::ChatId;

pub async fn send_message(
    token: &str,
    chat_id: ChatId,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let bot = Bot::new(token);
    bot.send_message(chat_id, message).send().await?;
    Ok(())
}
