use crate::blueprint_controller::BlueprintData;
use serde_json::from_reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::Mutex;

pub struct BlueprintCache {
    cache: Mutex<HashMap<String, Vec<BlueprintData>>>,
}

impl BlueprintCache {
    pub fn new() -> Self {
        BlueprintCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_cache_from_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let blueprints: Vec<BlueprintData> = from_reader(file)?;
        let mut cache = self.cache.lock().unwrap();

        for blueprint in blueprints {
            cache
                .entry(blueprint.card_name.clone())
                .or_insert_with(Vec::new)
                .push(blueprint);
        }

        Ok(())
    }

    pub fn get_blueprints_by_name(&self, name: &str) -> Option<Vec<BlueprintData>> {
        let cache = self.cache.lock().unwrap();
        cache.get(name).cloned()
    }

    pub fn get_all_card_names(&self) -> Vec<String> {
        let cache = self.cache.lock().unwrap();
        cache.keys().cloned().collect()
    }
}
