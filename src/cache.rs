use crate::api::Blueprint;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
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

    pub fn load_cache(&self, file_path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let re = Regex::new(r"Blueprint ID: (\d+), Card Name: (.+), Collector Number: (.*)")?;
        let mut cache = self.cache.lock().unwrap();

        for line in reader.lines() {
            let line = line?;
            if let Some(caps) = re.captures(&line) {
                let id: u32 = caps.get(1).unwrap().as_str().parse()?;
                let name = caps.get(2).unwrap().as_str().to_string();
                let collector_number = caps.get(3).map_or(None, |m| Some(m.as_str().to_string()));

                let blueprint = Blueprint {
                    id,
                    name: name.clone(),
                    collector_number,
                };

                cache.entry(name).or_insert_with(Vec::new).push(blueprint);
            }
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
