use std::{path::{Path, PathBuf}, fs::File, io::Read, ffi::OsString};

use color_eyre::Result;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

pub mod config;

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
            path.file_name().expect("file does not have a filename!").to_owned()
        } else {
            if let Some(ext) = path.extension() {
                let mut s = OsString::from((&hash[..16]).clone());
                s.push(".");
                s.push(ext);
                s
            } else {
                (&hash[..16]).clone().into()
            }
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
