use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Blueprint {
    pub id: u32,
    pub name: String,
    pub version: Option<String>,
    pub collector_number: Option<String>,
    pub expansion_name: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BlueprintData {
    pub blueprint_id: u32,
    pub card_name: String,
    pub version: Option<String>,
    pub collector_number: String,
    pub expansion_name: String,
}

#[derive(Deserialize)]
pub struct FixedProperties {
    pub collector_number: Option<String>,
}

#[derive(Deserialize)]
pub struct BlueprintApiResponse {
    pub id: u32,
    pub name: String,
    pub version: Option<String>,
    pub fixed_properties: FixedProperties,
}
