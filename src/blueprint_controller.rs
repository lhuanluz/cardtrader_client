use crate::api;
use crate::api::Expansion;
use indicatif::ProgressBar;
use reqwest::{header::HeaderMap, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::BufReader;

#[derive(Serialize, Deserialize, Clone)]
pub struct BlueprintData {
    pub blueprint_id: u32,
    pub card_name: String,
    pub collector_number: String,
    pub expansion_name: String,
}

pub async fn save_all_blueprints_to_json(
    client: &Client,
    headers: &HeaderMap,
    expansions: &Vec<Expansion>,
) -> Result<(), Box<dyn Error>> {
    let mut existing_blueprints = HashSet::new();
    let mut all_blueprints: Vec<BlueprintData> = Vec::new();

    // Carrega blueprints existentes do arquivo JSON
    if let Ok(file) = File::open("all_blueprints.json") {
        let reader = BufReader::new(file);
        if let Ok(existing_data) = serde_json::from_reader(reader) {
            all_blueprints = existing_data;
            for blueprint in &all_blueprints {
                existing_blueprints.insert(blueprint.blueprint_id);
            }
        }
    }

    let total_expansions = expansions.len();
    let bar = ProgressBar::new(total_expansions as u64);

    // Adiciona novos blueprints
    for expansion in expansions {
        let blueprints = api::fetch_blueprints(client, headers.clone(), expansion.id).await?;
        for blueprint in blueprints {
            if !existing_blueprints.contains(&blueprint.id) {
                let blueprint_data = BlueprintData {
                    blueprint_id: blueprint.id,
                    card_name: blueprint.name.clone(),
                    collector_number: blueprint
                        .collector_number
                        .clone()
                        .unwrap_or_else(|| "N/A".to_string()),
                    expansion_name: expansion.name.clone(),
                };
                all_blueprints.push(blueprint_data);
                existing_blueprints.insert(blueprint.id);
            }
        }
        bar.inc(1);
    }

    bar.finish();

    // Salva todos os blueprints no arquivo JSON
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("all_blueprints.json")?;
    serde_json::to_writer_pretty(file, &all_blueprints)?;

    println!("Todos os blueprints foram salvos em all_blueprints.json.");
    Ok(())
}
