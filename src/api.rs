use crate::blueprint::Blueprint;
use crate::blueprint::BlueprintApiResponse;
use crate::expansion::Expansion;
use reqwest::{header::HeaderMap, Client};
use std::error::Error;

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
    let url = format!(
        "https://api.cardtrader.com/api/v2/blueprints/export?expansion_id={}",
        expansion_id
    );
    let request = client.request(reqwest::Method::GET, &url).headers(headers);
    let response = request.send().await?;
    let body = response.text().await?;
    let blueprints_response: Vec<BlueprintApiResponse> = serde_json::from_str(&body)?;
    println!(
        "Fetched {} blueprints for {}",
        blueprints_response.len(),
        expansion_name
    );
    let blueprints: Vec<Blueprint> = blueprints_response
        .into_iter()
        .map(|bp| Blueprint {
            id: bp.id,
            name: bp.name,
            version: bp.version,
            collector_number: bp.collector_number,
            expansion_name: expansion_name.clone(),
        })
        .collect();
    Ok(blueprints)
}
