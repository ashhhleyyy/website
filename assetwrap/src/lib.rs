use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::Result;
use loader::Loader;
#[cfg(feature = "rust-s3")]
use s3::{creds::Credentials, Bucket, Region};
use serde::{Deserialize, Serialize};

pub mod config;
pub mod loader;

#[derive(Parser)]
pub struct Cli {
    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub upload_s3: bool,

    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub s3_endpoint: Option<String>,
    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub s3_access_key: Option<String>,
    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub s3_secret_key: Option<String>,
    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub s3_bucket: Option<String>,
    #[cfg(feature = "rust-s3")]
    #[clap(long)]
    pub s3_region: Option<String>,
}

#[cfg(feature = "rust-s3")]
pub fn create_s3_client() -> Option<Bucket> {
    let cli = Cli::parse();

    if cli.upload_s3 {
        if let Some(s3_access_key) = cli.s3_access_key {
            std::env::set_var("AWS_ACCESS_KEY_ID", s3_access_key);
        }
        if let Some(s3_secret_key) = cli.s3_secret_key {
            std::env::set_var("AWS_SECRET_ACCESS_KEY", s3_secret_key);
        }
        let creds = Credentials::default().expect("failed to load s3 credentials");
        let region = Region::Custom {
            region: cli
                .s3_region
                .or_else(|| std::env::var("S3_REGION").ok())
                .expect("missing s3 region"),
            endpoint: cli
                .s3_endpoint
                .or_else(|| std::env::var("S3_ENDPOINT").ok())
                .expect("missing s3 endpoint"),
        };
        let bucket_name = cli
            .s3_bucket
            .or_else(|| std::env::var("S3_BUCKET").ok())
            .expect("missing s3 bucket");
        Some(
            Bucket::new(&bucket_name, region, creds)
                .expect("failed to load s3 bucket")
                .with_path_style(),
        )
    } else {
        None
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
    pub content: Vec<u8>,
    pub hash: String,
    pub output_filename: OsString,
}

impl Asset {
    pub fn render(&self, output_dir: &Path) -> Result<(PathBuf, u64)> {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }
        let output_path = output_dir.join(&self.output_filename);

        std::fs::write(&output_path, &self.content)?;

        Ok((output_path, self.content.len() as u64))
    }
}

pub fn generate_assets(path: &Path, loader: Loader, no_hash: bool) -> Result<Vec<Asset>> {
    loader.process(path, !no_hash)
}
