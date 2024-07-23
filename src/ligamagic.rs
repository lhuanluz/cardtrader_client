use crate::api;
use dotenv::dotenv;
use reqwest::{header::HeaderMap, Client};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use tokio::time::{sleep, Duration};

pub async fn check_ligamagic_prices(
    client: &Client,
    headers: &HeaderMap,
    _user_name: &str,
) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let file = OpenOptions::new().read(true).open("prices.txt")?;
    let reader = BufReader::new(file);

    // Mapa para armazenar o menor preço de cada nome de carta
    let mut card_prices: HashMap<String, u32> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }
        let blueprint_id: u32 = parts[0].parse()?;
        let _saved_price: u32 = parts[1].parse()?;

        let products = api::fetch_products(client, headers.clone(), blueprint_id).await?;
        if products.is_empty() {
            continue;
        }

        let product = &products[0];
        let card_name = &product.name_en;
        let card_price = product.price_cents;

        // Atualiza o menor preço para o nome da carta
        card_prices
            .entry(card_name.clone())
            .and_modify(|price| {
                if card_price < *price {
                    *price = card_price;
                }
            })
            .or_insert(card_price);
    }

    for (card_name, card_price) in card_prices {
        let encoded_card_name = encode_ligamagic_card_name(&card_name);
        let url = format!(
            "https://www.ligamagic.com.br/?view=cards/card&card={}",
            encoded_card_name
        );

        if let Some(liga_price) = buscar_valor_min_no_ligamagic(client, &url).await? {
            println!(
                "{} | CardTrader: R$ {:.2} | LigaMagic: R$ {:.2}",
                card_name,
                card_price as f64 / 100.0,
                liga_price
            );
        } else {
            println!("Could not find price for {} on LigaMagic", card_name);
        }
    }

    Ok(())
}

async fn buscar_valor_min_no_ligamagic(
    client: &Client,
    url: &str,
) -> Result<Option<f64>, Box<dyn Error>> {
    let res = fetch_page_with_retry(client, url).await?;
    let document = Html::parse_document(&res);
    let link_selector = Selector::parse("div.e-col7 a.goto").unwrap();

    // Encontra o link "Ir à loja"
    let mut shop_url = None;
    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            shop_url = Some(href.to_string());
            break;
        }
    }

    if let Some(shop_url) = shop_url {
        let full_url = format!("https://www.ligamagic.com.br/{}", shop_url);
        let res = fetch_page_with_retry(client, &full_url).await?;
        let document = Html::parse_document(&res);
        let redirect_selector = Selector::parse("h5 a.color-1").unwrap();

        let mut final_url = None;
        for element in document.select(&redirect_selector) {
            if let Some(href) = element.value().attr("href") {
                final_url = Some(href.to_string());
                break;
            }
        }

        if let Some(final_url) = final_url {
            let res = fetch_page_with_retry(client, &final_url).await?;
            let document = Html::parse_document(&res);
            let row_selector = Selector::parse("div.table-cards-row").unwrap();
            let price_selector = Selector::parse("div.card-preco").unwrap();
            let discount_price_selector =
                Selector::parse("div.preco_com_desconto font[color='red']").unwrap();
            let stock_selector = Selector::parse("div.table-cards-body-cell:nth-child(5)").unwrap();

            let mut min_price: Option<f64> = None;

            for row in document.select(&row_selector) {
                let stock_element = row.select(&stock_selector).next();
                let price_element = row.select(&price_selector).next();
                let discount_price_element = row.select(&discount_price_selector).next();

                if let Some(stock_element) = stock_element {
                    let stock_text = stock_element
                        .text()
                        .collect::<String>()
                        .trim()
                        .replace("Estoque\n", "")
                        .replace("unid.", "")
                        .trim()
                        .to_string();

                    if let Ok(stock) = stock_text.parse::<u32>() {
                        if stock > 0 {
                            let price_text =
                                if let Some(discount_price_element) = discount_price_element {
                                    discount_price_element
                                        .text()
                                        .collect::<String>()
                                        .trim()
                                        .replace("Preço\n", "")
                                        .replace("R$", "")
                                        .replace(",", ".")
                                        .trim()
                                        .to_string()
                                } else if let Some(price_element) = price_element {
                                    price_element
                                        .text()
                                        .collect::<String>()
                                        .trim()
                                        .replace("Preço\n", "")
                                        .replace("R$", "")
                                        .replace(",", ".")
                                        .trim()
                                        .to_string()
                                } else {
                                    continue;
                                };

                            if let Ok(price) = price_text.parse::<f64>() {
                                if price > 0.0 {
                                    min_price = match min_price {
                                        Some(current_min) => Some(current_min.min(price)),
                                        None => Some(price),
                                    };
                                }
                            }
                        }
                    }
                }
            }
            Ok(min_price)
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

async fn fetch_page_with_retry(client: &Client, url: &str) -> Result<String, Box<dyn Error>> {
    let mut attempts = 0;
    loop {
        match client
            .get(url)
            .timeout(Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => match response.text().await {
                Ok(text) => return Ok(text),
                Err(err) => {
                    println!("Error reading response text: {:?}", err);
                    attempts += 1;
                }
            },
            Err(err) => {
                println!("Error fetching URL: {:?}", err);
                attempts += 1;
            }
        }
        if attempts >= 5 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Failed to fetch page after 5 attempts",
            )));
        }
        sleep(Duration::from_secs(2u64.pow(attempts))).await;
    }
}

fn encode_ligamagic_card_name(card_name: &str) -> String {
    let mut encoded_name = String::new();

    for c in card_name.chars() {
        match c {
            ' ' => encoded_name.push('+'),
            '\'' => encoded_name.push_str("%27"),
            ',' => encoded_name.push_str("%2C"),
            '/' => encoded_name.push_str("%2F"),
            '!' => encoded_name.push_str("%21"),
            '?' => encoded_name.push_str("%3F"),
            // Adicione outros caracteres especiais aqui se necessário
            _ => encoded_name.push(c),
        }
    }

    encoded_name
}
