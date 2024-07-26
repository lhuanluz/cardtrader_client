use crate::cardtrader_controller::fetch_card_price;
use crate::telegram;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Error as IOError};
use teloxide::types::ChatId;

#[derive(Serialize, Deserialize)]
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

pub async fn check_wishlist_prices() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let telegram_token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set");
    let telegram_chat_id: i64 = env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID must be set")
        .parse()
        .expect("TELEGRAM_CHAT_ID must be a valid i64");
    let mut alert_messages = Vec::new();

    let mut wishlist = load_wishlist()?;
    for item in &mut wishlist {
        println!(
            "Checking price of {} ({}) [{}] - {}",
            item.card_name, item.collector_number, item.expansion_name, item.version
        );
        let current_price =
            fetch_card_price(&item.card_name, &item.expansion_name, &item.version).await?;
        if current_price < item.price {
            println!(
                "Alert: The price of {} ({}) [{}] - {} has dropped from R$ {} to R$ {}",
                item.card_name,
                item.collector_number,
                item.expansion_name,
                item.version,
                item.price,
                current_price
            );
            let alert_message = format!(
                "*{}*\nQueda: _R$ {:.2} \\({:.2}%\\)_\nPreço Atual: *R$ {}*",
                escape_markdown(&item.card_name),
                escape_markdown(&(item.price - current_price).to_string()),
                escape_markdown(&(((item.price - current_price) / item.price) * 100.0).to_string()),
                escape_markdown(&(current_price).to_string())
            );
            alert_messages.push(alert_message);

            item.price = current_price;
        }
    }
    if !alert_messages.is_empty() {
        let chat_id = ChatId(telegram_chat_id);
        for chunk in split_message(&alert_messages.join("\n\n"), 4000) {
            let consolidated_message = format!("*Alerta de Preço Baixo\\!*\n\n{}", chunk);
            telegram::send_message(&telegram_token, chat_id, &consolidated_message).await?;
        }
    }

    save_wishlist(&wishlist)?;
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
