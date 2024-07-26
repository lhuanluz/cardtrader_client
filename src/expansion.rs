use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Expansion {
    pub id: u32,
    pub name: String,
}
