use std::{ffi::OsString, io::Cursor, path::Path};

use color_eyre::{eyre::eyre, Result};
use image::{io::Reader as ImageReader, DynamicImage, ImageOutputFormat};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::Asset;

#[derive(Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Loader {
    Default,
    Css,
    Image,
}

impl Default for Loader {
    fn default() -> Self {
        Self::Default
    }
}

impl Loader {
    pub fn process(&self, input_path: &Path, hashed_name: bool) -> Result<Vec<Asset>> {
        match self {
            Self::Default => Self::load_default(input_path, hashed_name),
            Self::Css => Self::load_css(input_path, hashed_name),
            Self::Image => Self::load_image(input_path, hashed_name),
        }
    }

    fn load_image(input_path: &Path, hashed_name: bool) -> Result<Vec<Asset>> {
        let img = ImageReader::open(input_path)?.decode()?;
        Ok(vec![
            Self::generate_image(
                input_path,
                &img,
                ImageOutputFormat::Avif,
                "avif",
                hashed_name,
            )?,
            Self::generate_image(
                input_path,
                &img,
                ImageOutputFormat::WebP,
                "webp",
                hashed_name,
            )?,
            Self::generate_image(input_path, &img, ImageOutputFormat::Png, "png", hashed_name)?,
        ])
    }

    fn generate_image<F: Into<ImageOutputFormat>>(
        input_path: &Path,
        img: &DynamicImage,
        format: F,
        ext: &str,
        hashed_name: bool,
    ) -> Result<Asset> {
        let mut content = Vec::<u8>::new();
        img.write_to(&mut Cursor::new(&mut content), format)?;
        Self::create_asset(input_path, content, hashed_name, Some(ext))
    }

    fn load_css(input_path: &Path, hashed_name: bool) -> Result<Vec<Asset>> {
        let source_css = std::fs::read_to_string(input_path)?;
        let content = {
            let filename = input_path
                .file_name()
                .expect("file does not have a name")
                .to_string_lossy()
                .to_string();
            Self::minify_css(filename, &source_css)?
        };

        let asset = Self::create_asset(input_path, content, hashed_name, None)?;
        Ok(vec![asset])
    }

    fn minify_css(filename: String, source: &str) -> Result<Vec<u8>> {
        let mut stylesheet = StyleSheet::parse(
            source,
            ParserOptions {
                filename,
                ..ParserOptions::default()
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

    fn load_default(input_path: &Path, hashed_name: bool) -> Result<Vec<Asset>> {
        let content = std::fs::read(input_path)?;
        let asset = Self::create_asset(input_path, content, hashed_name, None)?;
        Ok(vec![asset])
    }

    fn create_asset(
        input_path: &Path,
        content: Vec<u8>,
        hashed_name: bool,
        override_ext: Option<&str>,
    ) -> Result<Asset> {
        let hash = Self::hash_content(input_path, &content)?;
        let output_filename: OsString = {
            if !hashed_name {
                let path = if let Some(ext) = override_ext {
                    input_path.with_extension(ext)
                } else {
                    input_path.to_path_buf()
                };
                path.file_name()
                    .expect("file does not have a filename!")
                    .to_owned()
            } else {
                let mut s = OsString::from(hash[..16].to_owned());
                if let Some(ext) = override_ext {
                    s.push(".");
                    s.push(ext);
                } else if let Some(ext) = input_path.extension() {
                    s.push(".");
                    s.push(ext);
                }
                s
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
