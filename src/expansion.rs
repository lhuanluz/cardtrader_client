use serde::Deserialize;

#[derive(Deserialize)]
pub struct Expansion {
    pub id: u32,
    pub name: String,
}
