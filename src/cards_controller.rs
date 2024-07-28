use crate::cache::BlueprintCache;
use crate::cardtrader_controller::fetch_card_price;
use crate::error::CustomError;
use crate::wishlist_controller::{add_to_wishlist, WishlistItem};
use futures::future::join_all;
use inquire::{InquireError, Select};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

const MAX_CONCURRENT_CHECKS: usize = 10;

pub async fn list_and_select_cards(cache: &BlueprintCache) -> Result<(), Box<dyn Error>> {
    // Lista os nomes das cartas a partir do cache
    let card_names: Vec<String> = cache.get_all_card_names();

    // Usuário seleciona um nome de carta
    let select_card_name: Result<String, InquireError> =
        Select::new("Select a card name:", card_names).prompt();

    match select_card_name {
        Ok(card_name) => {
            if let Some(versions) = cache.get_blueprints_by_name(&card_name) {
                let mut version_descriptions: Vec<String> = versions
                    .iter()
                    .map(|bp| {
                        format!(
                            "{} ({}) - {}",
                            bp.card_name, bp.collector_number, bp.expansion_name,
                        )
                    })
                    .collect();
                version_descriptions.push("Add all versions".to_string());

                // Usuário seleciona uma versão da carta
                let select_version: Result<String, InquireError> =
                    Select::new("Select a card version:", version_descriptions).prompt();

                match select_version {
                    Ok(version) => {
                        if version == "Add all versions" {
                            let mut tasks = Vec::new();
                            let pb = indicatif::ProgressBar::new(versions.len() as u64);
                            let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));
                            println!("You selected to add all versions of {}:", card_name);
                            for version in &versions {
                                let semaphore_clone = Arc::clone(&semaphore);
                                let version_clone = version.clone();
                                let pb_clone = pb.clone();
                                let task = task::spawn(async move {
                                    let _permit = semaphore_clone.acquire().await.unwrap();

                                    let price = fetch_card_price(
                                        &version_clone.card_name,
                                        &version_clone.expansion_name,
                                        &version_clone.version.as_deref().unwrap_or(""),
                                    )
                                    .await
                                    .map_err(|e| CustomError::new(&e.to_string()))?;

                                    let item = WishlistItem {
                                        card_name: version_clone.card_name.clone(),
                                        expansion_name: version_clone.expansion_name.clone(),
                                        version: version_clone
                                            .version
                                            .as_deref()
                                            .unwrap_or("")
                                            .to_string(),
                                        price,
                                        collector_number: version_clone.collector_number.clone(),
                                    };
                                    let _ = add_to_wishlist(item);
                                    pb_clone.inc(1);
                                    Ok(()) as Result<(), CustomError>
                                });
                                tasks.push(task);
                            }
                            let results = join_all(tasks).await;
                            pb.finish_with_message("Finished adding all versions to wishlist");
                            for result in results {
                                match result {
                                    Ok(_) => {}
                                    Err(e) => println!("Failed to add a card version: {}", e),
                                }
                            }
                        } else {
                            let selected_version = versions
                                .iter()
                                .find(|v| {
                                    format!(
                                        "{} ({}) - {}",
                                        v.card_name, v.collector_number, v.expansion_name,
                                    ) == version
                                })
                                .unwrap();

                            let price = fetch_card_price(
                                &selected_version.card_name,
                                &selected_version.expansion_name,
                                selected_version.version.as_deref().unwrap_or(""),
                            )
                            .await?;

                            let item = WishlistItem {
                                card_name: selected_version.card_name.clone(),
                                expansion_name: selected_version.expansion_name.clone(),
                                version: selected_version
                                    .version
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_string(),
                                price,
                                collector_number: selected_version.collector_number.clone(),
                            };
                            add_to_wishlist(item)?;
                        }
                    }
                    Err(_) => println!("Failed to select a card version."),
                }
            } else {
                println!("No versions found for the selected card.");
            }
        }
        Err(_) => println!("Failed to select a card name."),
    }

    Ok(())
}
