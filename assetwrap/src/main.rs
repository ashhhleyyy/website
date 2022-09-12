use std::fs::File;

use bytesize::ByteSize;
use color_eyre::Result;
use globset::GlobSetBuilder;
use walkdir::WalkDir;

fn main() -> Result<()> {
    color_eyre::install()?;

    let bucket = assetwrap::create_s3_client();

    let mut assets = vec![];

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
            if glob.is_match(entry.path()) {
                let no_hash = asset_path.no_hash || no_hash_matchers.is_match(entry.path());
                assets.push(assetwrap::generate_asset(
                    entry.path(),
                    &asset_path.output,
                    no_hash,
                )?);
            }
        }
    }

    let mut index = File::create("assetindex.json")?;
    serde_json::to_writer_pretty(&mut index, &assets)?;

    let mut total_size = ByteSize::b(0);
    for asset in &assets {
        let size = asset.render()?;
        let size = ByteSize::b(size);
        println!("Rendered {:?} ({})", asset.output_path, size);
        total_size += size;
    }

    println!("Rendered {} assets ({})", assets.len(), total_size);

    if let Some(bucket) = bucket {
        println!("Uploading {} assets to S3...", assets.len());
        for asset in &assets {
            // TODO: Don't hardcode this prefix, lol
            let path = asset.output_path.to_string_lossy().replace("./assets-gen/", "");
            if matches!(bucket.head_object(&path)?, (_, 404)) { // Doesn't exist, we need to upload
                let mut reader = File::open(&asset.output_path)?;
                bucket.put_object_stream(&mut reader, &path)?;
                println!("Uploaded {path} to bucket!");
            }
        }
    }

    Ok(())
}
