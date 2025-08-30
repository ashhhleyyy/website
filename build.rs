fn main() {
    println!("cargo::rerun-if-env-changed=ASSET_INDEX");
    if std::env::var("ASSET_INDEX").is_err() {
        println!("cargo::rustc-env=ASSET_INDEX=../assetindex.json")
    }
}
