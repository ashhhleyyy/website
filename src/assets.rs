use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct AssetMap(HashMap<String, Vec<String>>);

const ASSET_INDEX_STR: &str = include_str!("../assetindex.json");

pub static ASSET_INDEX: Lazy<AssetMap> = Lazy::new(load_asset_map);

impl AssetMap {
    pub fn get<'a>(&'a self, name: &'a str) -> &'a str {
        if let Some(s) = self.0.get(name) {
            if let Some(s) = s.get(0) {
                s
            } else {
                name
            }
        } else {
            name
        }
    }

    pub fn get_all<'a>(&'a self, name: &'a str) -> Option<Vec<&str>> {
        self.0
            .get(name)
            .map(|s| s.iter().map(|s| s.as_str()).collect::<Vec<_>>())
    }
}

#[cfg(not(debug_assertions))]
const ASSET_PREFIX: &str = "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/";
#[cfg(debug_assertions)]
const ASSET_PREFIX: &str = "/assets/";

pub fn load_asset_map() -> AssetMap {
    let assets: HashMap<String, Vec<String>> = serde_json::from_str(ASSET_INDEX_STR).unwrap();
    let mut map = HashMap::new();

    for (original_name, new_names) in assets {
        let new_names = new_names
            .iter()
            .map(|new_name| format!("{ASSET_PREFIX}{new_name}"))
            .collect::<Vec<_>>();
        map.insert(original_name, new_names);
    }

    AssetMap(map)
}
