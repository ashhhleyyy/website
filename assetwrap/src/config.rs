use std::{fs::File, path::PathBuf, io::Read};

use color_eyre::{Result, eyre::eyre};
use globset::Glob;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssetConfig {
    pub asset_paths: Vec<AssetPath>,
}

#[derive(Deserialize)]
pub struct AssetPath {
    pub input: Glob,
    pub output: PathBuf,
    #[serde(default)]
    pub no_hash: bool,
    #[serde(default)]
    pub hash_ignore: Vec<Glob>,
}

pub fn load_config() -> Result<AssetConfig> {
    let mut buf = String::new();
    File::open("assetconfig.json")
        .map_err(|e| eyre!("failed to open config: {e}"))?
        .read_to_string(&mut buf)
        .map_err(|e| eyre!("failed to read config: {e}"))?;
    serde_json::from_str(&buf)
        .map_err(|e| eyre!("failed to parse config: {e}"))
}
