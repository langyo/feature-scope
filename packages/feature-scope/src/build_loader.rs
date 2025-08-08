use anyhow::{anyhow, Context, Result};
use std::{collections::HashSet, path::PathBuf};

use crate::utils::{
    manifest_parser::{parse_manifest, FeatureScopeDecl},
    workspace_parser::parse_workspace,
};

pub fn load() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let current_manifest_path = {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .context("CARGO_MANIFEST_DIR environment variable not found")?;
        let mut path = PathBuf::from(manifest_dir);
        path.push("Cargo.toml");
        path
    };

    let current_metadata = parse_manifest(&current_manifest_path)
        .context("Failed to parse current package manifest")?;
    let workspace_info = parse_workspace().context("Failed to parse workspace information")?;

    let mut available_features = HashSet::new();
    if let Some(decl) = &current_metadata.feature_scope_decl {
        for (feature, _) in &decl.features {
            available_features.insert(feature.clone());
        }
    }

    // TODO: Is it possible to get the name of the final package being compiled?
    //       Maybe I can write a CLI program as a cargo component to parse and provide extra environment variables,
    //       then I can use those variables in build.rs to avoid the limitations
    //       that I cannot get the final package's name during pre-compilation.

    let mut used_features = HashSet::new();
    if let Some(workspace_info) = &workspace_info {
        for feature_ref in &current_metadata.feature_scope_refs {
            // Collect features from referenced packages
            let toml_path = workspace_info
                .packages
                .get(&feature_ref.package)
                .context(format!(
                    "Package '{}' not found in workspace",
                    feature_ref.package
                ))?
                .manifest_path
                .clone();
            let metadata = parse_manifest(&toml_path).context(format!(
                "Failed to parse manifest for package '{}'",
                feature_ref.package
            ))?;

            if let Some(feature) = &metadata.feature_scope_decl {
                available_features.extend(feature.features.keys().cloned());
            }

            fn dfs(features_decl: &FeatureScopeDecl, name: &str) -> Result<HashSet<String>> {
                if let Some(features) = features_decl.features.get(name) {
                    let mut ret = HashSet::new();
                    ret.extend(features.iter().cloned());
                    for feature in features {
                        if feature != "default" {
                            ret.extend(dfs(features_decl, feature)?);
                        }
                    }
                    Ok(ret)
                } else {
                    Err(anyhow!("Feature '{}' not found", name))
                }
            }

            // Check if the other features are used
            for feature in &feature_ref.features {
                if feature != "default" {
                    // Check the source declaration
                    if let Some(features_decl) = &metadata.feature_scope_decl {
                        let features_decl = dfs(features_decl, feature)
                            .context(format!("Failed to resolve feature '{}'", feature))?;
                        used_features.extend(features_decl);
                    } else {
                        return Err(anyhow!(
                            "Feature '{}' is not declared in package '{}'",
                            feature,
                            feature_ref.package
                        ));
                    }

                    used_features.insert(feature.clone());
                } else {
                    // If "default" is used, include all features in the declaration
                    if let Some(features_decl) = &metadata.feature_scope_decl {
                        for (feature, _) in &features_decl.features {
                            let features_decl = dfs(features_decl, feature)
                                .context(format!("Failed to resolve feature '{}'", feature))?;
                            used_features.extend(features_decl);
                            used_features.insert(feature.clone());
                        }
                    } else {
                        return Err(anyhow!(
                            "Feature 'default' is not declared in package '{}'",
                            feature_ref.package
                        ));
                    }
                }
            }
        }
    }

    // TODO: Split the crates into different namespace prefixes, like `__scope_{crate}_{feature}`
    println!("cargo:warning=Available features: {:?}", available_features);
    println!("cargo:warning=Used features: {:?}", used_features);

    // Print the rustc params
    if workspace_info.is_some() {
        // Write rustc-cfg attributes only if in a workspace to avoid conflicts
        // FIXME: rustc-cfg was not correctly passed to types, but actually types' build.rs should also output __scope_b,
        //        otherwise compilation cannot be distinguished
        for feature in &used_features {
            println!("cargo:rustc-cfg=__scope_{}", feature);
        }
    }
    println!("cargo:rustc-cfg=__scope_default");
    for feature in &available_features {
        println!("cargo:rustc-check-cfg=cfg(__scope_{})", feature);
    }

    Ok(())
}
