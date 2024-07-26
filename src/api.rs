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
    let response = client
        .get("https://api.cardtrader.com/v1/expansions")
        .headers(headers)
        .send()
        .await?;
    let expansions: Vec<Expansion> = response.json().await?;
    Ok(expansions)
}

pub async fn fetch_blueprints(
    client: &Client,
    headers: HeaderMap,
    expansion_id: u32,
) -> Result<Vec<Blueprint>, Box<dyn Error>> {
    let url = format!(
        "https://api.cardtrader.com/v1/expansions/{}/blueprints",
        expansion_id
    );
    let response = client.get(&url).headers(headers).send().await?;
    let blueprints: Vec<Blueprint> = response.json().await?;
    Ok(blueprints)
}
