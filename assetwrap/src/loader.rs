use std::{ffi::OsString, path::Path};

use color_eyre::{eyre::eyre, Result};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::Asset;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Loader {
    Default,
    Css,
}

impl Default for Loader {
    fn default() -> Self {
        Self::Default
    }
}

impl Loader {
    pub fn process(&self, input_path: &Path, hashed_name: bool) -> Result<Asset> {
        match self {
            Self::Default => Self::load_default(input_path, hashed_name),
            Self::Css => Self::load_css(input_path, hashed_name),
        }
    }

    fn load_css(input_path: &Path, hashed_name: bool) -> Result<Asset> {
        let source_css = std::fs::read_to_string(input_path)?;
        let content = {
            let filename = input_path
                .file_name()
                .expect("file does not have a name")
                .to_string_lossy()
                .to_string();
            Self::minify_css(filename, &source_css)?
        };

        Self::create_asset(input_path, content, hashed_name)
    }

    fn minify_css(filename: String, source: &str) -> Result<Vec<u8>> {
        let mut stylesheet = StyleSheet::parse(
            source,
            ParserOptions {
                filename,
                ..Default::default()
            },
        )
        .map_err(|e| eyre!("failed to parse: {e}"))?;
        let minify_options = MinifyOptions::default();
        stylesheet.minify(minify_options)?;
        Ok(stylesheet
            .to_css(PrinterOptions {
                minify: true,
                ..Default::default()
            })?
            .code
            .into_bytes())
    }

    fn load_default(input_path: &Path, hashed_name: bool) -> Result<Asset> {
        let content = std::fs::read(input_path)?;
        Self::create_asset(input_path, content, hashed_name)
    }

    fn create_asset(input_path: &Path, content: Vec<u8>, hashed_name: bool) -> Result<Asset> {
        let hash = Self::hash_content(input_path, &content)?;
        let output_filename: OsString = {
            if !hashed_name {
                input_path
                    .file_name()
                    .expect("file does not have a filename!")
                    .to_owned()
            } else if let Some(ext) = input_path.extension() {
                let mut s = OsString::from(hash[..16].to_owned());
                s.push(".");
                s.push(ext);
                s
            } else {
                hash[..16].to_owned().into()
            }
        };
        Ok(Asset {
            content,
            hash,
            output_filename,
        })
    }

    fn hash_content(input_path: &Path, content: &[u8]) -> Result<String> {
        let mut hasher: Sha256 = Sha256::new();
        hasher.update(input_path.to_string_lossy().as_bytes());
        hasher.update(b"\x00");
        hasher.update(content);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }
}
