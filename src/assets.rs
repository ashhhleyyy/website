use std::collections::HashMap;

use assetwrap::Asset;
use once_cell::sync::Lazy;

pub struct AssetMap(HashMap<String, String>);

const ASSET_INDEX_STR: &str = include_str!("../assetindex.json");

pub static ASSET_INDEX: Lazy<AssetMap> = Lazy::new(load_asset_map);

impl AssetMap {
    pub fn get<'a>(&'a self, name: &'a str) -> &'a str {
        if let Some(s) = self.0.get(name) {
            s
        } else {
            name
        }
    }
}

#[cfg(not(debug_assertions))]
const ASSET_PREFIX: &str = "https://cdn.ashhhleyyy.dev/file/ashhhleyyy-assets/";
#[cfg(debug_assertions)]
const ASSET_PREFIX: &str = "/assets/";

pub fn load_asset_map() -> AssetMap {
    let assets: Vec<Asset> = serde_json::from_str(ASSET_INDEX_STR).unwrap();
    let mut map = HashMap::new();

    for asset in assets {
        let original_name = asset.input_path.to_string_lossy().replace("./", "/");
        let new_name = asset.output_path.to_string_lossy().replace(
            "./assets-gen/",
            ASSET_PREFIX,
        );
        map.insert(original_name, new_name);
    }

    AssetMap(map)
}
