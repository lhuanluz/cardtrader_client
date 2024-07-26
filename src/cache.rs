use crate::blueprint::Blueprint;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;

pub struct BlueprintCache {
    cache: Mutex<HashMap<String, Vec<Blueprint>>>,
}

impl BlueprintCache {
    pub fn new() -> Self {
        BlueprintCache {
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_cache_from_json(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let blueprints: Vec<Blueprint> = serde_json::from_reader(reader)?;

        let mut cache = self.cache.lock().unwrap();
        for blueprint in blueprints {
            cache
                .entry(blueprint.name.clone())
                .or_insert_with(Vec::new)
                .push(blueprint);
        }

        Ok(())
    }

    pub fn get_blueprints_by_name(&self, name: &str) -> Option<Vec<Blueprint>> {
        let cache = self.cache.lock().unwrap();
        cache.get(name).cloned()
    }

    pub fn get_all_card_names(&self) -> Vec<String> {
        let cache = self.cache.lock().unwrap();
        cache.keys().cloned().collect()
    }
}
