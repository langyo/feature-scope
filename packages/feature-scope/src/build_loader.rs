use crate::utils::{
    manifest_parser::{parse_manifest, ManifestMetadata},
    workspace_parser::{parse_workspace, WorkspaceInfo},
};
use anyhow::{Context, Result};
use std::{collections::HashSet, path::PathBuf};

pub fn load() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // 1. Get current package's Cargo.toml path
    let current_manifest_path = {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .context("CARGO_MANIFEST_DIR environment variable not found")?;
        let mut path = PathBuf::from(manifest_dir);
        path.push("Cargo.toml");
        path
    };

    // 2. Parse current package's Cargo.toml
    let current_metadata = parse_manifest(&current_manifest_path)
        .context("Failed to parse current package manifest")?;

    // 3. Try to get workspace information
    let workspace_info = parse_workspace().context("Failed to parse workspace information")?;

    // 4. Process feature-scope configuration
    if let Some(workspace) = workspace_info {
        process_feature_scope_with_workspace(&current_metadata, &workspace)?;
    } else {
        // If no workspace, only process current package's feature-scope-decl
        process_feature_scope_standalone(&current_metadata)?;
    }

    Ok(())
}

/// Process feature-scope configuration in workspace environment
fn process_feature_scope_with_workspace(
    current_metadata: &ManifestMetadata,
    workspace: &WorkspaceInfo,
) -> Result<()> {
    let mut activated_features = HashSet::new();
    let mut all_possible_features = HashSet::new();
    let mut has_non_empty_features = false;

    // Add all possible features declared by current package
    if let Some(decl) = &current_metadata.feature_scope_decl {
        for feature_list in decl.features.values() {
            for feature in feature_list {
                all_possible_features.insert(feature.clone());
            }
        }
    }

    // Iterate through current package's feature-scope references
    for scope_ref in &current_metadata.feature_scope_refs {
        // Check if there are non-empty features
        if !scope_ref.features.is_empty() {
            has_non_empty_features = true;
        }

        // 1. Find corresponding package in workspace
        let target_package = workspace
            .packages
            .get(&scope_ref.package)
            .with_context(|| {
                format!(
                    "Referenced package '{}' not found in workspace. Available packages: {}",
                    scope_ref.package,
                    workspace
                        .packages
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;

        // 2. Parse target package's Cargo.toml
        let target_metadata = parse_manifest(&target_package.manifest_path).with_context(|| {
            format!(
                "Failed to parse manifest for package '{}'",
                scope_ref.package
            )
        })?;

        // 3. Check if target package has feature-scope-decl
        let target_decl = target_metadata
            .feature_scope_decl
            .as_ref()
            .with_context(|| {
                format!(
                    "Package '{}' does not have feature-scope-decl metadata",
                    scope_ref.package
                )
            })?;

        // Add all possible features declared by target package
        for feature_list in target_decl.features.values() {
            for feature in feature_list {
                all_possible_features.insert(feature.clone());
            }
        }

        // 4. Verify that requested features exist in target package
        for requested_feature in &scope_ref.features {
            if !target_decl.features.contains_key(requested_feature) {
                return Err(anyhow::anyhow!(
                    "Package '{}' does not declare feature '{}'. Available features: {}",
                    scope_ref.package,
                    requested_feature,
                    target_decl
                        .features
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }

            // 5. Activate corresponding features
            if let Some(feature_list) = target_decl.features.get(requested_feature) {
                for feature in feature_list {
                    activated_features.insert(feature.clone());
                }
            }
        }
    }

    // If there are no feature-scope references at all, or all features lists are empty, activate default
    if current_metadata.feature_scope_refs.is_empty() || !has_non_empty_features {
        // Process default features from current package's feature-scope-decl
        if let Some(decl) = &current_metadata.feature_scope_decl {
            if let Some(default_features) = decl.features.get("default") {
                for feature in default_features {
                    activated_features.insert(feature.clone());
                }
            }
        }
        // If no feature-scope-decl is declared or default is empty, output __scope_default
        if activated_features.is_empty() {
            activated_features.insert("default".to_string());
        }
    }

    // Ensure default is always in possible features
    all_possible_features.insert("default".to_string());

    // 6. Output all activated features as rustc-cfg
    for feature in &activated_features {
        println!("cargo:rustc-cfg=__scope_{}", feature);
    }

    // Output all possible features' check-cfg (deduplicated)
    for feature in &all_possible_features {
        println!("cargo:rustc-check-cfg=cfg(__scope_{})", feature);
    }

    Ok(())
}

/// Process feature-scope configuration in non-workspace environment
fn process_feature_scope_standalone(current_metadata: &ManifestMetadata) -> Result<()> {
    let mut activated_features = HashSet::new();
    let mut all_possible_features = HashSet::new();
    let mut has_non_empty_features = false;

    // Add all possible features declared by current package
    if let Some(decl) = &current_metadata.feature_scope_decl {
        for feature_list in decl.features.values() {
            for feature in feature_list {
                all_possible_features.insert(feature.clone());
            }
        }
    }

    // Check if there are non-empty feature-scope references
    for scope_ref in &current_metadata.feature_scope_refs {
        if !scope_ref.features.is_empty() {
            has_non_empty_features = true;
            break;
        }
    }

    // In non-workspace environment, issue warning if there are feature-scope references
    if !current_metadata.feature_scope_refs.is_empty() {
        eprintln!(
            "cargo:warning=Found {} feature-scope references but not in a workspace. These will be ignored.",
            current_metadata.feature_scope_refs.len()
        );
        for scope_ref in &current_metadata.feature_scope_refs {
            eprintln!(
                "cargo:warning=  - {} -> {:?}",
                scope_ref.package, scope_ref.features
            );
        }
    }

    // If there are no feature-scope references at all, or all features lists are empty, activate default
    if current_metadata.feature_scope_refs.is_empty() || !has_non_empty_features {
        // Process default features from current package's feature-scope-decl
        if let Some(decl) = &current_metadata.feature_scope_decl {
            if let Some(default_features) = decl.features.get("default") {
                for feature in default_features {
                    activated_features.insert(feature.clone());
                }
            }
        }
        // If no feature-scope-decl is declared or default is empty, output __scope_default
        if activated_features.is_empty() {
            activated_features.insert("default".to_string());
        }
    }

    // Ensure default is always in possible features
    all_possible_features.insert("default".to_string());

    // Output activated features
    for feature in &activated_features {
        println!("cargo:rustc-cfg=__scope_{}", feature);
    }

    // Output all possible features' check-cfg (deduplicated)
    for feature in &all_possible_features {
        println!("cargo:rustc-check-cfg=cfg(__scope_{})", feature);
    }

    Ok(())
}
