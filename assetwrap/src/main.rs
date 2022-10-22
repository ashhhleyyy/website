use std::{collections::HashMap, fs::File, path::PathBuf};

use bytesize::ByteSize;
use color_eyre::Result;
use globset::GlobSetBuilder;
use walkdir::WalkDir;

fn main() -> Result<()> {
    color_eyre::install()?;

    #[cfg(feature = "rust-s3")]
    let bucket = assetwrap::create_s3_client();

    let mut asset_map = HashMap::new();

    let mut total_size = ByteSize::b(0);

    let config = assetwrap::config::load_config()?;
    for asset_path in &config.asset_paths {
        let no_hash_matchers = {
            let mut builder = GlobSetBuilder::new();
            for ignore in &asset_path.hash_ignore {
                builder.add(ignore.clone());
            }
            builder.build()?
        };
        let glob = asset_path.input.compile_matcher();
        for entry in WalkDir::new(".").into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && glob.is_match(entry.path()) {
                let no_hash = asset_path.no_hash || no_hash_matchers.is_match(entry.path());
                let asset = assetwrap::generate_asset(entry.path(), asset_path.loader, no_hash)?;
                let (output_path, size) = asset.render(&asset_path.output)?;
                let original_name = entry.path().to_string_lossy().replace("./", "/");
                let new_name = output_path.to_string_lossy().replace("./assets-gen/", "");
                let size = ByteSize::b(size);
                total_size += size;
                println!("Rendered {} ({})", &new_name, size);
                asset_map.insert(original_name, new_name);
            }
        }
    }

    let mut index = File::create("assetindex.json")?;
    serde_json::to_writer_pretty(&mut index, &asset_map)?;

    println!("Rendered {} assets ({})", asset_map.len(), total_size);

    #[cfg(feature = "rust-s3")]
    if let Some(bucket) = bucket {
        println!("Uploading {} assets to S3...", asset_map.len());
        for output_path in asset_map.values() {
            // TODO: Don't hardcode this prefix, lol
            let res = bucket.head_object(&output_path)?;
            if res.1 == 404 {
                // Doesn't exist, we need to upload
                let fs_path = {
                    let mut path = PathBuf::new();
                    path.push("./assets-gen/");
                    path.push(&output_path);
                    path
                };

                let content = std::fs::read(&fs_path)?;
                let content_type = mime_guess::from_path(&output_path)
                    .first_or_text_plain()
                    .essence_str()
                    .to_owned();
                bucket.put_object_with_content_type(&output_path, &content[..], &content_type)?;
                println!("Uploaded {output_path} to bucket!");
            } else if res.1 != 200 {
                eprintln!("failed to upload {output_path}: got status {}", res.1);
            } else {
                println!("{output_path} already exists in bucket!");
            }
        }
    }

    Ok(())
}
