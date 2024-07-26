use crate::blueprint::Blueprint;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
pub struct Expansion {
    pub id: u32,
    pub name: String,
}

pub async fn fetch_expansions(
    client: &Client,
    headers: HeaderMap,
) -> Result<Vec<Expansion>, Box<dyn Error>> {
    let request = client
        .request(
            reqwest::Method::GET,
            "https://api.cardtrader.com/api/v2/expansions",
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;
    let expansions: Vec<Expansion> = serde_json::from_str(&body)?;
    Ok(expansions)
}

pub async fn fetch_blueprints(
    client: &Client,
    headers: HeaderMap,
    expansion_id: u32,
    expansion_name: &String,
) -> Result<Vec<Blueprint>, Box<dyn Error>> {
    let requets = client
        .request(
            reqwest::Method::GET,
            format!(
                "https://api.cardtrader.com/api/v2/blueprints/export?expansion_id={}",
                expansion_id
            ),
        )
        .headers(headers);
    let response = requets.send().await?;
    let body = response.text().await?;
    let blueprints: Vec<Blueprint> = serde_json::from_str(&body)
        .into_iter()
        .map(|bp: Blueprint| Blueprint {
            id: bp.id,
            name: bp.name,
            version: bp.version,
            collector_number: bp.collector_number,
            expansion_name: expansion_name.clone(),
        })
        .collect();
    Ok(blueprints)
}
