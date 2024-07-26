use crate::api;
use crate::blueprint::BlueprintData;
use indicatif::ProgressBar;
use reqwest::{header::HeaderMap, Client};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};

pub async fn save_all_blueprints_to_json(
    client: &Client,
    headers: &HeaderMap,
    expansions: &Vec<api::Expansion>,
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
        let blueprints =
            api::fetch_blueprints(client, headers.clone(), expansion.id, &expansion.name).await?;
        for blueprint in blueprints {
            if !existing_blueprints.contains(&blueprint.id) {
                let blueprint_data = BlueprintData {
                    blueprint_id: blueprint.id,
                    card_name: blueprint.name.clone(),
                    version: blueprint.version.clone(),
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
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &all_blueprints)?;

    println!("Todos os blueprints foram salvos em all_blueprints.json.");
    Ok(())
}
