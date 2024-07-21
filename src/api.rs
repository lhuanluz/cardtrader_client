use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Expansion {
    pub id: u32,
    pub game_id: u32,
    pub code: String,
    pub name: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Blueprint {
    pub id: u32,
    pub name: String,
    pub collector_number: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Product {
    pub id: u32,
    pub name_en: String,
    pub price_cents: u32,
    pub price_currency: String,
    pub quantity: u32,
    // outros campos
}

#[derive(Deserialize)]
struct FixedProperties {
    collector_number: Option<String>,
}

#[derive(Deserialize)]
struct BlueprintData {
    id: u32,
    name: String,
    fixed_properties: FixedProperties,
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
) -> Result<Vec<Blueprint>, Box<dyn Error>> {
    let url = format!(
        "https://api.cardtrader.com/api/v2/blueprints/export?expansion_id={}",
        expansion_id
    );
    let request = client.request(reqwest::Method::GET, &url).headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;
    let blueprint_data: Vec<BlueprintData> = serde_json::from_str(&body)?;
    let blueprints: Vec<Blueprint> = blueprint_data
        .into_iter()
        .map(|data| Blueprint {
            id: data.id,
            name: data.name,
            collector_number: data.fixed_properties.collector_number,
        })
        .collect();
    Ok(blueprints)
}

#[derive(Deserialize)]
struct ProductsWrapper {
    #[serde(flatten)]
    products: HashMap<String, Vec<Product>>,
}

pub async fn fetch_products(
    client: &Client,
    headers: HeaderMap,
    blueprint_id: u32,
) -> Result<Vec<Product>, Box<dyn Error>> {
    let url = format!(
        "https://api.cardtrader.com/api/v2/marketplace/products?blueprint_id={}",
        blueprint_id
    );
    let request = client.request(reqwest::Method::GET, &url).headers(headers);

    let response = request.send().await?;
    let body = response.text().await?;
    let products_wrapper: ProductsWrapper = serde_json::from_str(&body)?;

    let products = products_wrapper
        .products
        .values()
        .flatten()
        .cloned()
        .collect();

    Ok(products)
}
