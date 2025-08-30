use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::eyre, Result};
use globset::Glob;
use serde::Deserialize;

use crate::loader::Loader;

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
    #[serde(default)]
    pub just_copy: Vec<Glob>,
    #[serde(default)]
    pub loader: Loader,
}

pub fn load_config(path: &Path) -> Result<AssetConfig> {
    let mut buf = String::new();
    File::open(path)
        .map_err(|e| eyre!("failed to open config: {e}"))?
        .read_to_string(&mut buf)
        .map_err(|e| eyre!("failed to read config: {e}"))?;
    serde_json::from_str(&buf).map_err(|e| eyre!("failed to parse config: {e}"))
}
