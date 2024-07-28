use crate::cardtrader_controller::fetch_card_price;
use crate::cardtrader_controller::fetch_multiple_prices_fantoccini;
use crate::error::CustomError;
use crate::telegram;
use dotenv::dotenv;
use futures::future::join_all;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Error as IOError};
use std::sync::Arc;
use teloxide::types::ChatId;
use tokio::sync::Semaphore;
use tokio::task;

const MAX_CONCURRENT_CHECKS: usize = 10;

#[derive(Serialize, Deserialize, Clone)]
pub struct WishlistItem {
    pub card_name: String,
    pub expansion_name: String,
    pub version: String,
    pub price: f64,
    pub collector_number: String,
}

pub fn add_to_wishlist(item: WishlistItem) -> Result<(), IOError> {
    let mut wishlist = load_wishlist()?;
    println!("Adding {} to wishlist", item.card_name);
    wishlist.push(item);
    save_wishlist(&wishlist)
}

fn load_wishlist() -> Result<Vec<WishlistItem>, IOError> {
    let file = File::open("wishlist.json");
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let wishlist = serde_json::from_reader(reader)?;
            Ok(wishlist)
        }
        Err(_) => Ok(Vec::new()), // If the file doesn't exist, return an empty vector
    }
}

fn save_wishlist(wishlist: &Vec<WishlistItem>) -> Result<(), IOError> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("wishlist.json")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, wishlist)?;
    Ok(())
}

pub async fn check_wishlist_prices() -> Result<(), CustomError> {
    dotenv().ok();
    let telegram_token =
        env::var("TELEGRAM_TOKEN").map_err(|e| CustomError::new(&e.to_string()))?;
    let telegram_chat_id: i64 = env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID must be set")
        .parse()
        .expect("TELEGRAM_CHAT_ID must be a valid i64");
    let mut alert_messages = Vec::new();

    let mut wishlist = load_wishlist().map_err(|e| CustomError::new(&e.to_string()))?;
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));
    let mut tasks = Vec::new();

    let pb = ProgressBar::new(wishlist.len() as u64);
    for item in &mut wishlist {
        let semaphore_clone = Arc::clone(&semaphore);
        let item_clone = item.clone();

        let pb_clone = pb.clone();
        let task = task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap();
            let current_price = fetch_card_price(
                &item_clone.card_name,
                &item_clone.expansion_name,
                &item_clone.version,
            )
            .await
            .map_err(|e| CustomError::new(&e.to_string()))?;

            pb_clone.inc(1);
            if current_price < item_clone.price {
                let alert_message = format!(
                    "*{} \\({}\\) \\[{}\\]*\nPreço Desejado: *R$ {}*\nPreço Atual: *R$ {}*",
                    escape_markdown(&item_clone.card_name),
                    escape_markdown(&item_clone.collector_number),
                    escape_markdown(&item_clone.expansion_name),
                    escape_markdown(&item_clone.price.to_string()),
                    escape_markdown(&current_price.to_string())
                );

                Ok((item_clone, Some(alert_message)))
                    as Result<(WishlistItem, Option<String>), CustomError>
            } else {
                Ok((item_clone, None)) as Result<(WishlistItem, Option<String>), CustomError>
            }
        });

        tasks.push(task);
    }

    let results = join_all(tasks).await;

    pb.finish_with_message("Finished checking prices");
    for result in results {
        match result {
            Ok(Ok((item, Some(alert_message)))) => {
                alert_messages.push(alert_message);
                wishlist.iter_mut().for_each(|w_item| {
                    if w_item.card_name == item.card_name
                        && w_item.expansion_name == item.expansion_name
                        && w_item.version == item.version
                    {
                        w_item.price = item.price;
                    }
                });
            }
            Ok(Ok((_item, None))) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(CustomError::new(&format!("Task failed: {}", e))),
        }
    }

    if !alert_messages.is_empty() {
        let chat_id = ChatId(telegram_chat_id);
        for chunk in split_message(&alert_messages.join("\n\n"), 4000) {
            let consolidated_message = format!("*Alerta de Preço Baixo\\!*\n\n{}", chunk);
            telegram::send_message(&telegram_token, chat_id, &consolidated_message)
                .await
                .map_err(|e| CustomError::new(&e.to_string()))?;
        }
    }

    save_wishlist(&wishlist).map_err(|e| CustomError::new(&e.to_string()))?;
    Ok(())
}

// Função auxiliar para escapar caracteres especiais no MarkdownV2
fn escape_markdown(text: &str) -> String {
    let mut escaped = String::new();
    for c in text.chars() {
        match c {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' | '\\' => {
                escaped.push('\\');
                escaped.push(c);
            }
            _ => escaped.push(c),
        }
    }
    escaped
}
fn split_message(message: &str, max_length: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_chunk = String::new();

    for line in message.lines() {
        if current_chunk.len() + line.len() + 1 > max_length {
            result.push(current_chunk.clone());
            current_chunk.clear();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }

    if !current_chunk.is_empty() {
        result.push(current_chunk);
    }

    result
}

pub async fn continuous_check_prices() -> Result<(), Box<dyn Error>> {
    loop {
        // faça que o timeout seja de 10 segundos após o fim da execução da função
        check_wishlist_prices().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
}

pub async fn sync_prices() -> Result<(), Box<dyn Error>> {
    let mut wishlist = load_wishlist()?;
    let mut urls = Vec::new();
    for item in &mut wishlist {
        let card_name = &item.card_name;
        let expansion_name = &item.expansion_name;
        let version = &item.version;

        let clean_card_name = card_name
            .replace(' ', "-")
            .replace(",", "")
            .replace("'", "-")
            .replace(".", "")
            .replace(":", "")
            .to_lowercase();

        let clean_expansion_name = expansion_name
            .replace(' ', "-")
            .replace(",", "")
            .replace("'", "-")
            .replace(".", "")
            .replace(":", "")
            .to_lowercase();

        let clean_version = version
            .replace(' ', "-")
            .replace(",", "")
            .replace("'", "-")
            .replace(".", "")
            .replace(":", "")
            .to_lowercase();
        let url = if item.version.is_empty() {
            format!(
                "https://www.cardtrader.com/cards/{}-{}",
                clean_card_name, clean_expansion_name
            )
        } else {
            format!(
                "https://www.cardtrader.com/cards/{}-{}-{}",
                clean_card_name, clean_version, clean_expansion_name
            )
        };
        urls.push(url);
    }
    let prices = fetch_multiple_prices_fantoccini(urls).await?;

    for (item, price) in wishlist.iter_mut().zip(prices.iter()) {
        item.price = *price;
    }

    save_wishlist(&wishlist)?;
    Ok(())
}
