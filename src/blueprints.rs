use crate::api;
use crate::api::Blueprint;
use crate::api::Product;
use crate::prices::save_blueprint_price;
use inquire::{InquireError, Select};
use regex::Regex;
use reqwest::{header::HeaderMap, Client};
use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::time::Duration;
use tokio::time::sleep;

pub async fn show_blueprints(
    client: &Client,
    headers: &HeaderMap,
    blueprints: &Vec<Blueprint>,
) -> Result<(), Box<dyn Error>> {
    let blueprint_names: Vec<String> = blueprints
        .iter()
        .map(|bp| {
            format!(
                "{} ({})",
                bp.name,
                bp.collector_number
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string())
            )
        })
        .collect();

    let select_ans: Result<String, InquireError> =
        Select::new("Select a blueprint to view details:", blueprint_names).prompt();

    match select_ans {
        Ok(name) => {
            if let Some(blueprint) = blueprints.iter().find(|bp| {
                format!(
                    "{} ({})",
                    bp.name,
                    bp.collector_number
                        .clone()
                        .unwrap_or_else(|| "N/A".to_string())
                ) == name
            }) {
                let products = fetch_products_with_retry(client, headers, blueprint.id).await?;
                show_products(&products)?;

                if !products.is_empty() {
                    let min_price = products[0].price_cents;
                    save_blueprint_price(blueprint.id, min_price)?;
                } else {
                    save_blueprint_price(blueprint.id, 0)?;
                }
            } else {
                println!("Blueprint not found.");
            }
        }
        Err(_) => println!("Failed to select a blueprint."),
    }

    Ok(())
}

fn show_products(products: &Vec<Product>) -> Result<(), Box<dyn Error>> {
    println!("Products:");
    for product in products {
        println!(
            "ID: {}, Name: {}, Price: R$ {:.2}, Quantity: {}",
            product.id,
            product.name_en,
            product.price_cents as f64 / 100.0,
            product.quantity
        );
    }
    Ok(())
}

pub async fn search_and_select_blueprints(
    client: &Client,
    headers: &HeaderMap,
) -> Result<(), Box<dyn Error>> {
    let file = OpenOptions::new().read(true).open("all_blueprints.txt")?;
    let reader = BufReader::new(file);
    println!("Searching for cards, please wait...");
    let re = Regex::new(r"Blueprint ID: (\d+), Card Name: (.+), Collector Number: (.*)").unwrap();

    let mut blueprints: Vec<Blueprint> = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re.captures(&line) {
            let id: u32 = caps.get(1).unwrap().as_str().parse()?;
            let name = caps.get(2).unwrap().as_str().to_string();
            let collector_number = caps.get(3).map_or(None, |m| Some(m.as_str().to_string()));

            blueprints.push(Blueprint {
                id,
                name,
                collector_number,
            });
        }
    }

    let card_names: Vec<String> = blueprints.iter().map(|bp| bp.name.clone()).collect();
    let unique_card_names: HashSet<_> = card_names.iter().cloned().collect();
    let unique_card_names: Vec<_> = unique_card_names.into_iter().collect();

    let card_name_ans: Result<String, InquireError> =
        Select::new("Select a card name to view versions:", unique_card_names).prompt();

    match card_name_ans {
        Ok(card_name) => {
            let mut versions: Vec<String> = blueprints
                .iter()
                .filter(|bp| bp.name == card_name)
                .map(|bp| {
                    format!(
                        "{} ({})",
                        bp.name,
                        bp.collector_number
                            .clone()
                            .unwrap_or_else(|| "N/A".to_string())
                    )
                })
                .collect();

            versions.push("Add all versions".to_string());

            let version_ans: Result<String, InquireError> =
                Select::new("Select a version to add to prices:", versions).prompt();

            match version_ans {
                Ok(version) => {
                    if version == "Add all versions" {
                        for bp in blueprints.iter().filter(|bp| bp.name == card_name) {
                            let products =
                                fetch_products_with_retry(client, headers, bp.id).await?;
                            let min_price =
                                products.iter().map(|p| p.price_cents).min().unwrap_or(0);
                            save_blueprint_price(bp.id, min_price)?;
                        }
                    } else {
                        if let Some(bp) = blueprints.iter().find(|bp| {
                            format!(
                                "{} ({})",
                                bp.name,
                                bp.collector_number
                                    .clone()
                                    .unwrap_or_else(|| "N/A".to_string())
                            ) == version
                        }) {
                            let products =
                                fetch_products_with_retry(client, headers, bp.id).await?;
                            let min_price =
                                products.iter().map(|p| p.price_cents).min().unwrap_or(0);
                            save_blueprint_price(bp.id, min_price)?;
                        }
                    }
                }
                Err(_) => println!("Failed to select a version."),
            }
        }
        Err(_) => println!("Failed to select a card name."),
    }

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
