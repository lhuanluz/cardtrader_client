use crate::api;
use crate::api::Product;
use crate::telegram;
use dotenv::dotenv;
use reqwest::{header::HeaderMap, Client};
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;
use teloxide::types::ChatId;
use tokio::time::sleep;

pub fn save_blueprint_price(blueprint_id: u32, price_cents: u32) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("prices.txt")?;
    writeln!(file, "{} {}", blueprint_id, price_cents)?;
    Ok(())
}

pub async fn check_prices(client: &Client, headers: &HeaderMap) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let telegram_token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set");
    let telegram_chat_id: i64 = env::var("TELEGRAM_CHAT_ID")
        .expect("TELEGRAM_CHAT_ID must be set")
        .parse()
        .expect("TELEGRAM_CHAT_ID must be a valid i64");

    let file = OpenOptions::new().read(true).open("prices.txt")?;
    let reader = BufReader::new(file);

    let mut updates = Vec::new();
    let mut alert_messages = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }
        let blueprint_id: u32 = parts[0].parse()?;
        let saved_price: u32 = parts[1].parse()?;

        let products = fetch_products_with_retry(client, headers, blueprint_id).await?;

        if !products.is_empty() {
            let current_min_price = products[0].price_cents;
            let product_name = products[0].name_en.clone();
            println!(
                "[{}] {} | Valor desejado: R$ {:.2} | Valor na CT: R$ {:.2}",
                blueprint_id,
                product_name,
                saved_price as f64 / 100.0,
                current_min_price as f64 / 100.0
            );
            if current_min_price < saved_price {
                println!(
                    "Alert: [{}] {}: Current price R$ {:.2} is lower than saved price R$ {:.2}",
                    blueprint_id,
                    product_name,
                    current_min_price as f64 / 100.0,
                    saved_price as f64 / 100.0
                );

                let alert_message = format!(
                    "*{}*\nQueda: _R$ {}_\nPreço Atual: *R$ {}*",
                    escape_markdown(&product_name),
                    escape_markdown(&format!(
                        "{:.2}",
                        (saved_price as f64 - current_min_price as f64) / 100.0
                    )),
                    escape_markdown(&format!("{:.2}", current_min_price as f64 / 100.0))
                );
                alert_messages.push(alert_message);

                updates.push((blueprint_id, current_min_price));
            } else {
                updates.push((blueprint_id, saved_price));
            }
        } else {
            println!(
                "[{}] No products found for {}. Valor desejado: R$ {:.2}",
                blueprint_id,
                blueprint_id,
                saved_price as f64 / 100.0
            );
            updates.push((blueprint_id, saved_price));
        }
    }

    // Envia todas as mensagens de alerta de uma vez
    if !alert_messages.is_empty() {
        let chat_id = ChatId(telegram_chat_id);
        let consolidated_message = format!(
            "*Alerta de Preço Baixo\\!*\n\n{}",
            alert_messages.join("\n\n")
        );
        telegram::send_message(&telegram_token, chat_id, &consolidated_message).await?;
    }

    update_prices_file(updates)?;

    Ok(())
}

async fn fetch_products_with_retry(
    client: &Client,
    headers: &HeaderMap,
    blueprint_id: u32,
) -> Result<Vec<Product>, Box<dyn Error>> {
    let mut attempts = 0;
    let mut products = Vec::new();
    loop {
        match api::fetch_products(client, headers.clone(), blueprint_id).await {
            Ok(p) => {
                products = p;
                break;
            }
            Err(_) => {
                attempts += 1;
                if attempts >= 5 {
                    println!(
                        "Failed to fetch products for Blueprint ID {} after 5 attempts",
                        blueprint_id
                    );
                    break;
                }
                sleep(Duration::from_secs(2u64.pow(attempts))).await;
            }
        }
    }
    Ok(products)
}

fn update_prices_file(updates: Vec<(u32, u32)>) -> Result<(), Box<dyn Error>> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("prices.txt")?;

    for (blueprint_id, price) in updates {
        writeln!(file, "{} {}", blueprint_id, price)?;
    }

    Ok(())
}

pub async fn continuous_check_prices(
    client: &Client,
    headers: &HeaderMap,
) -> Result<(), Box<dyn Error>> {
    loop {
        println!("Checking prices...");
        check_prices(client, headers).await?;
        println!("Press Ctrl+C to stop continuous price check.");
        sleep(Duration::from_secs(60)).await;
    }
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

pub async fn update_local_prices(
    client: &Client,
    headers: &HeaderMap,
) -> Result<(), Box<dyn Error>> {
    println!("Updating prices...");
    let file = OpenOptions::new().read(true).open("prices.txt")?;
    let reader = BufReader::new(file);

    let mut updates = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }
        let blueprint_id: u32 = parts[0].parse()?;
        let _saved_price: u32 = parts[1].parse()?;

        let products = fetch_products_with_retry(client, headers, blueprint_id).await?;

        if !products.is_empty() {
            let current_min_price = products[0].price_cents;
            updates.push((blueprint_id, current_min_price));
        } else {
            updates.push((blueprint_id, 0)); // Add a placeholder if no product is found
        }
    }

    // Atualiza o arquivo prices.txt com os novos valores
    update_prices_file(updates)?;

    println!("Prices updated successfully!");

    Ok(())
}
