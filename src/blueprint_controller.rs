use crate::api;
use crate::blueprint::BlueprintData;
use crate::error::CustomError;
use crate::expansion::Expansion;
use indicatif::ProgressBar;
use reqwest::{header::HeaderMap, Client};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use tokio::sync::Semaphore;
use tokio::task;

const MAX_CONCURRENT_REQUESTS: usize = 50; // Limite de tarefas paralelas

pub async fn save_all_blueprints_to_json(
    client: &Client,
    headers: HeaderMap,
    expansions: Vec<Expansion>,
) -> Result<(), CustomError> {
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
    let semaphore = std::sync::Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
    let tasks: Vec<_> = expansions
        .into_iter()
        .map(|expansion| {
            let client = client.clone();
            let headers = headers.clone();
            let expansion_name = expansion.name.clone();
            let existing_blueprints = existing_blueprints.clone();
            let semaphore = semaphore.clone();

            task::spawn(async move {
                let _permit = semaphore.acquire().await;
                let blueprints = api::fetch_blueprints(&client, headers, expansion.id)
                    .await
                    .map_err(|e| CustomError::new(&e.to_string()))?;
                let new_blueprints: Vec<_> = blueprints
                    .into_iter()
                    .filter_map(|blueprint| {
                        if !existing_blueprints.contains(&blueprint.id) {
                            Some(BlueprintData {
                                blueprint_id: blueprint.id,
                                card_name: blueprint.name,
                                collector_number: blueprint
                                    .collector_number
                                    .unwrap_or_else(|| "N/A".to_string()),
                                expansion_name: expansion_name.clone(),
                                version: blueprint.version.clone(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<BlueprintData>>();
                Ok::<Vec<BlueprintData>, CustomError>(new_blueprints)
            })
        })
        .collect();

    for task in tasks {
        match task.await {
            Ok(Ok(new_blueprints)) => {
                all_blueprints.extend(new_blueprints);
                bar.inc(1);
            }
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(CustomError::new(&e.to_string())),
        }
    }

    bar.finish();

    // Salva todos os blueprints no arquivo JSON
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("all_blueprints.json")
        .map_err(|e| CustomError::new(&e.to_string()))?;
    serde_json::to_writer_pretty(file, &all_blueprints)
        .map_err(|e| CustomError::new(&e.to_string()))?;

    println!("Todos os blueprints foram salvos em all_blueprints.json.");
    Ok(())
}
