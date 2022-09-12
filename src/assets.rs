use std::collections::HashMap;

use assetwrap::Asset;
use once_cell::sync::Lazy;

pub struct AssetMap(HashMap<String, String>);

const ASSET_INDEX_STR: &str = include_str!("../assetindex.json");

pub static ASSET_INDEX: Lazy<AssetMap> = Lazy::new(load_asset_map);

impl AssetMap {
    pub fn get<'a>(&'a self, name: &'a str) -> &'a str {
        println!("{} {:?}", name, self.0);
        if let Some(s) = self.0.get(name) {
            s
        } else {
            name
        }
    }
}

pub fn load_asset_map() -> AssetMap {
    let assets: Vec<Asset> = serde_json::from_str(ASSET_INDEX_STR).unwrap();
    let mut map = HashMap::new();

    for asset in assets {
        let original_name = asset.input_path.to_string_lossy().replace("./", "/");
        let new_name = asset.output_path.to_string_lossy().replace("./assets-gen/", "/assets/");
        map.insert(original_name, new_name);
    }

    AssetMap(map)
}
