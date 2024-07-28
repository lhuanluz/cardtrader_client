use dotenv::dotenv;
use reqwest::Client;
use std::error::Error;
use teloxide::prelude::*;
use teloxide::types::{ChatId, ParseMode};
use teloxide::Bot;
pub async fn send_message(
    token: &str,
    chat_id: ChatId,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()?;
    let bot = Bot::with_client(token, client).parse_mode(ParseMode::MarkdownV2);
    bot.send_message(chat_id, message).send().await?;
    Ok(())
}
