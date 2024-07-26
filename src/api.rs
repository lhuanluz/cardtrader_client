use crate::blueprint::{Blueprint, BlueprintApiResponse};
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
) -> Result<Vec<Blueprint>, Box<dyn Error>> {
    let request = client
        .request(
            reqwest::Method::GET,
            format!(
                "https://api.cardtrader.com/api/v2/blueprints/export?expansion_id={}",
                expansion_id
            ),
        )
        .headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;
    let api_response: Vec<BlueprintApiResponse> = serde_json::from_str(&body)?;

    let blueprints: Vec<Blueprint> = api_response
        .into_iter()
        .map(|resp| Blueprint {
            id: resp.id,
            name: resp.name,
            version: resp.version,
            collector_number: resp.fixed_properties.collector_number,
            expansion_name: String::new(), // Placeholder, will be set in the controller
        })
        .collect();

    Ok(blueprints)
}
