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
    let client = Client::builder().build()?;
    let bot = Bot::with_client(token, client).parse_mode(ParseMode::MarkdownV2);
    bot.send_message(chat_id, message).send().await?;
    Ok(())
}

pub async fn run() {
    let token = std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not found");
    let bot = Bot::new(token);
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        // now handle the message as if had args, considering the /add /helo commands +
        // whatever is next
        match msg.text() {
            Some(text) => {
                if text.starts_with("/add") {
                    let chat_id = msg.chat.id;
                    let message = text.replace("/add", "");
                    bot.send_message(chat_id, message).send().await?;
                } else if text.starts_with("/hello") {
                    let chat_id = msg.chat.id;
                    bot.send_message(chat_id, "Hello!").send().await?;
                }
            }
            None => {}
        }
        Ok(())
    })
    .await;
}
