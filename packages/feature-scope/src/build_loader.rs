use anyhow::Result;
use std::{collections::HashMap, fs, path::PathBuf};

pub fn load() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Parse Cargo.toml
    let path = {
        let path = std::env::var("CARGO_MANIFEST_DIR")?;
        let mut path = PathBuf::from(path);
        path.push("Cargo.toml");
        path
    };
    let toml = fs::read_to_string(path)?;

    // Get [package.metadata.feature-scope]
    let toml = toml::de::from_str::<toml::Value>(&toml)?;
    let features = toml
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|p| p.get("feature-scope"))
        .and_then(|f| f.as_table())
        .map(|table| {
            table
                .iter()
                .filter_map(|(key, value)| {
                    value.as_array().map(|arr| {
                        (
                            key.clone(),
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<_>>(),
                        )
                    })
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    // Set rustc-cfg for default features
    if let Some(default_features) = features.get("default") {
        for feature in default_features {
            println!("cargo:rustc-cfg=__scope_{}", feature);
            println!("cargo:rustc-check-cfg=cfg(__scope_{})", feature);
        }
    }

    Ok(())
}
