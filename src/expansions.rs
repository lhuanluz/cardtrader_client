use crate::api;
use crate::api::Expansion;
use crate::blueprints::show_blueprints;
use inquire::{InquireError, Select};
use reqwest::{header::HeaderMap, Client};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

pub async fn show_expansions(
    client: &Client,
    headers: &HeaderMap,
    expansions: &Vec<Expansion>,
) -> Result<(), Box<dyn Error>> {
    let expansion_names: Vec<String> = expansions.iter().map(|exp| exp.name.clone()).collect();

    let select_ans: Result<String, InquireError> =
        Select::new("Select an expansion to view details:", expansion_names).prompt();

    match select_ans {
        Ok(name) => {
            if let Some(expansion) = expansions.iter().find(|exp| exp.name == name) {
                let blueprints =
                    api::fetch_blueprints(client, headers.clone(), expansion.id).await?;
                show_blueprints(client, headers, &blueprints).await?;
            } else {
                println!("Expansion not found.");
            }
        }
        Err(_) => println!("Failed to select an expansion."),
    }

    Ok(())
}

pub async fn save_all_blueprints(
    client: &Client,
    headers: &HeaderMap,
    expansions: &Vec<Expansion>,
) -> Result<(), Box<dyn Error>> {
    let mut existing_blueprints = HashSet::new();

    if let Ok(file) = File::open("all_blueprints.txt") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line) = line {
                existing_blueprints.insert(line);
            }
        }
    }

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("all_blueprints.txt")?;

    for expansion in expansions {
        let blueprints = api::fetch_blueprints(client, headers.clone(), expansion.id).await?;
        for blueprint in blueprints {
            let line = format!(
                "Blueprint ID: {}, Card Name: {}, Collector Number: {}",
                blueprint.id,
                blueprint.name,
                blueprint
                    .collector_number
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string())
            );
            if !existing_blueprints.contains(&line) {
                file.write_all(line.as_bytes())?;
                file.write_all(b"\n")?;
                existing_blueprints.insert(line);
            }
        }
    }

    println!("Todos os blueprints foram salvos em all_blueprints.txt.");
    Ok(())
}
