use crate::cache::BlueprintCache;
use crate::cardtrader_controller::fetch_card_price;
use inquire::{InquireError, Select};
use std::error::Error;

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
                            "{} ({}) [{}]",
                            bp.card_name, bp.collector_number, bp.expansion_name
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
                            println!("You selected to add all versions of {}:", card_name);
                            for version in &versions {
                                let price =
                                    fetch_card_price(&version.card_name, &version.expansion_name)
                                        .await?;
                                println!(
                                    "{} ({}) [{}] - Price: R$ {:.2}",
                                    version.card_name,
                                    version.collector_number,
                                    version.expansion_name,
                                    price
                                );
                            }
                        } else {
                            let selected_version = versions
                                .iter()
                                .find(|v| {
                                    format!(
                                        "{} ({}) [{}]",
                                        v.card_name, v.collector_number, v.expansion_name
                                    ) == version
                                })
                                .unwrap();

                            let price = fetch_card_price(
                                &selected_version.card_name,
                                &selected_version.expansion_name,
                            )
                            .await?;

                            println!("You selected: {} - Price: R$ {:.2}", version, price);
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
