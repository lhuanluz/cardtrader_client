use crate::api;
use crate::api::Blueprint;
use crate::api::Product;
use crate::cache::BlueprintCache;
use crate::prices::save_blueprint_price;
use inquire::{InquireError, Select};
use reqwest::{header::HeaderMap, Client};
use std::error::Error;
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
    cache: &BlueprintCache,
) -> Result<(), Box<dyn Error>> {
    let card_names = cache.get_all_card_names();
    let card_name_ans: Result<String, InquireError> =
        Select::new("Select a card name to view versions:", card_names).prompt();

    match card_name_ans {
        Ok(card_name) => {
            if let Some(blueprints) = cache.get_blueprints_by_name(&card_name) {
                let mut versions: Vec<String> = blueprints
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

                versions.push("Add all versions".to_string());

                let version_ans: Result<String, InquireError> =
                    Select::new("Select a version to add to prices:", versions).prompt();

                match version_ans {
                    Ok(version) => {
                        if version == "Add all versions" {
                            for bp in blueprints {
                                let products =
                                    fetch_products_with_retry(client, headers, bp.id).await?;
                                let min_price =
                                    products.iter().map(|p| p.price_cents).min().unwrap_or(0);
                                save_blueprint_price(bp.id, min_price)?; // Save with the current price or 0 if not available
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
                                save_blueprint_price(bp.id, min_price)?; // Save with the current price or 0 if not available
                            }
                        }
                    }
                    Err(_) => println!("Failed to select a version."),
                }
            } else {
                println!("No blueprints found for the selected card name.");
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
