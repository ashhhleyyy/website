use std::{
    ffi::OsString,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use clap::Parser;
use color_eyre::Result;
#[cfg(feature = "rust-s3")]
use s3::{creds::Credentials, Bucket, Region};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod config;

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
    pub input_path: PathBuf,
    pub hash: String,
    pub output_path: PathBuf,
}

impl Asset {
    pub fn render(&self) -> Result<u64> {
        if let Some(parent) = self.output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut input_file = File::open(&self.input_path)?;
        let mut output_file = File::create(&self.output_path)?;

        let size = std::io::copy(&mut input_file, &mut output_file)?;

        Ok(size)
    }
}

pub fn generate_asset(path: &Path, output_path: &Path, no_hash: bool) -> Result<Asset> {
    let mut hasher: Sha256 = Sha256::new();
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.update(b"\x00");
    {
        let mut f = File::open(path)?;
        let mut buf = [0; 4096];
        loop {
            let read = f.read(&mut buf)?;
            if read == 0 {
                break;
            }
            hasher.update(&buf[..read]);
        }
    }
    let hash = {
        let result = hasher.finalize();
        hex::encode(result)
    };

    let output_filename: OsString = {
        if no_hash {
            path.file_name()
                .expect("file does not have a filename!")
                .to_owned()
        } else if let Some(ext) = path.extension() {
            let mut s = OsString::from(hash[..16].to_owned());
            s.push(".");
            s.push(ext);
            s
        } else {
            hash[..16].to_owned().into()
        }
    };

    let output_path = output_path.join(output_filename);

    let asset = Asset {
        input_path: path.to_path_buf(),
        hash,
        output_path,
    };
    Ok(asset)
}
