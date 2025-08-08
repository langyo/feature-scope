//! # feature-scope CLI
//!
//! A helper CLI tool that enables workspace crates to independently control their required
//! features without cross-package interference.
//!
//! ## Overview
//!
//! This CLI tool provides the correct compiler arguments for the `feature-scope` library.
//! You need to use `cargo feature-scope` instead of regular `cargo` commands when building
//! or running your project.
//!
//! ## Usage
//!
//! ```bash
//! # Build your project
//! cargo feature-scope build
//!
//! # Run your project
//! cargo feature-scope run
//!
//! # Run a specific package in workspace
//! cargo feature-scope run -p your-package-name
//!
//! # Run tests
//! cargo feature-scope test
//! ```
//!
//! ## Installation
//!
//! ```bash
//! cargo install cargo-feature-scope
//! ```
//!
//! ## Configuration
//!
//! This tool works with a two-step configuration in your `Cargo.toml`:
//!
//! 1. **Declare features** in library crates:
//! ```toml
//! [package.metadata.feature-scope-decl]
//! default = ["a"]
//! a = []
//! b = []
//! ```
//!
//! 2. **Configure feature usage** in consumer crates:
//! ```toml
//! [[package.metadata.feature-scope]]
//! package = "your-library-name"
//! features = ["b"]
//! default-features = false
//! ```

use anyhow::{Context, Result};
use clap::{Arg, ArgMatches, Command};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    env,
    path::PathBuf,
    process,
};

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<Package>,
    workspace: Option<Workspace>,
}

#[derive(Debug, Deserialize)]
struct Workspace {
    members: Option<Vec<String>>,
    #[serde(rename = "default-members")]
    default_members: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "feature-scope-decl")]
    feature_scope_decl: Option<FeatureScopeDecl>,
    #[serde(rename = "feature-scope")]
    feature_scope: Option<Vec<FeatureScope>>,
}

#[derive(Debug, Deserialize)]
struct FeatureScopeDecl {
    default: Option<Vec<String>>,
    #[serde(flatten)]
    features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FeatureScope {
    package: String,
    features: Vec<String>,
    #[serde(rename = "default-features")]
    default_features: Option<bool>,
}

fn main() -> Result<()> {
    let app = Command::new("cargo-feature-scope")
        .bin_name("cargo")
        .subcommand_required(true)
        .subcommand(
            Command::new("feature-scope")
                .about("Cargo feature scope helper")
                .arg(
                    Arg::new("command")
                        .help("Cargo command to run (build, check, run, test, etc.)")
                        .required(true)
                        .value_name("COMMAND"),
                )
                .arg(
                    Arg::new("package")
                        .short('p')
                        .long("package")
                        .help("Package to build")
                        .value_name("SPEC"),
                )
                .arg(
                    Arg::new("args")
                        .help("Additional arguments to pass to cargo")
                        .action(clap::ArgAction::Append)
                        .trailing_var_arg(true),
                ),
        );

    let matches = app.get_matches();

    if let Some(feature_scope_matches) = matches.subcommand_matches("feature-scope") {
        run_feature_scope(feature_scope_matches)?;
    }

    Ok(())
}

fn run_feature_scope(matches: &ArgMatches) -> Result<()> {
    let command = matches.get_one::<String>("command").unwrap();
    let package = matches.get_one::<String>("package");
    let additional_args: Vec<&String> = matches.get_many("args").unwrap_or_default().collect();

    // Get current directory and root Cargo.toml
    let current_dir = env::current_dir()?;
    let root_manifest_path = find_root_manifest(&current_dir)?;

    // Parse root Cargo.toml
    let root_content = std::fs::read_to_string(&root_manifest_path)
        .with_context(|| format!("Failed to read {}", root_manifest_path.display()))?;
    let root_cargo_toml: CargoToml =
        toml::from_str(&root_content).with_context(|| "Failed to parse root Cargo.toml")?;

    // Determine target package
    let target_package_name = if let Some(pkg) = package {
        pkg.clone()
    } else {
        // If no package is specified, determine the default package
        determine_default_package(&root_cargo_toml, &root_manifest_path)?
    };

    // Check if it's a workspace
    let (cfg_args, check_cfg_args) = if root_cargo_toml.workspace.is_some() {
        // Workspace mode
        handle_workspace_package(&root_cargo_toml, &root_manifest_path, &target_package_name)?
    } else {
        // Single package mode
        handle_single_package(&root_cargo_toml)?
    };

    // Build and execute cargo command
    execute_cargo_command(
        command,
        package,
        &cfg_args,
        &check_cfg_args,
        &additional_args,
    )
}

fn find_root_manifest(start_dir: &PathBuf) -> Result<PathBuf> {
    let mut current_dir = start_dir.clone();

    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    anyhow::bail!("Could not find Cargo.toml in current directory or parent directories")
}

fn determine_default_package(
    root_cargo_toml: &CargoToml,
    root_manifest_path: &PathBuf,
) -> Result<String> {
    if let Some(workspace) = &root_cargo_toml.workspace {
        // Workspace mode: use the first default-members or first members
        if let Some(default_members) = &workspace.default_members {
            if let Some(first_default) = default_members.first() {
                return Ok(extract_package_name_from_path(
                    first_default,
                    root_manifest_path,
                )?);
            }
        }

        if let Some(members) = &workspace.members {
            if let Some(first_member) = members.first() {
                return Ok(extract_package_name_from_path(
                    first_member,
                    root_manifest_path,
                )?);
            }
        }

        anyhow::bail!("No members found in workspace")
    } else {
        // Single package mode: use current package name
        if let Some(package) = &root_cargo_toml.package {
            Ok(package.name.clone())
        } else {
            anyhow::bail!("No package found in root Cargo.toml")
        }
    }
}

fn extract_package_name_from_path(
    member_path: &str,
    root_manifest_path: &PathBuf,
) -> Result<String> {
    let root_dir = root_manifest_path.parent().unwrap();
    let member_manifest = root_dir.join(member_path).join("Cargo.toml");

    let content = std::fs::read_to_string(&member_manifest)
        .with_context(|| format!("Failed to read {}", member_manifest.display()))?;
    let cargo_toml: CargoToml =
        toml::from_str(&content).with_context(|| "Failed to parse member Cargo.toml")?;

    if let Some(package) = cargo_toml.package {
        Ok(package.name)
    } else {
        anyhow::bail!("No package found in {}", member_manifest.display())
    }
}

fn handle_single_package(cargo_toml: &CargoToml) -> Result<(Vec<String>, Vec<String>)> {
    let mut cfg_args = vec![String::from("--cfg"), String::from("__scope_default")];
    let mut all_scope_features = HashSet::new();

    // Always add default scope
    all_scope_features.insert("__scope_default".to_string());

    if let Some(package) = &cargo_toml.package {
        if let Some(metadata) = &package.metadata {
            // Single package mode: feature-scope-decl and feature-scope are in the same file
            if let Some(feature_scope_decl) = &metadata.feature_scope_decl {
                // Collect all declared feature scopes
                for feature_name in feature_scope_decl.features.keys() {
                    all_scope_features.insert(format!("__scope_{}", feature_name));
                }

                // Iteratively parse default features and their dependencies
                let mut enabled_features = HashSet::new();
                if let Some(defaults) = &feature_scope_decl.default {
                    for default_feature in defaults {
                        resolve_feature_dependencies(
                            default_feature,
                            &feature_scope_decl.features,
                            &mut enabled_features,
                        );
                    }
                }

                // Add cfg parameters for enabled features
                for feature in &enabled_features {
                    all_scope_features.insert(format!("__scope_{}", feature));
                    cfg_args.push(String::from("--cfg"));
                    cfg_args.push(format!("__scope_{}", feature));
                }

                if let Some(feature_scope) = &metadata.feature_scope {
                    // Cross-validate and apply feature-scope configuration
                    for scope in feature_scope {
                        for feature in &scope.features {
                            if feature_scope_decl.features.contains_key(feature) {
                                // Parse dependencies of this feature
                                let mut scope_enabled_features = HashSet::new();
                                resolve_feature_dependencies(
                                    feature,
                                    &feature_scope_decl.features,
                                    &mut scope_enabled_features,
                                );

                                for enabled_feature in scope_enabled_features {
                                    cfg_args.push(String::from("--cfg"));
                                    cfg_args.push(format!("__scope_{}", enabled_feature));
                                }
                            } else {
                                eprintln!(
                                    "Warning: feature '{}' not declared in feature-scope-decl",
                                    feature
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // Build check-cfg parameters
    let mut check_cfg_args = Vec::new();
    for scope_feature in all_scope_features {
        check_cfg_args.push(String::from("--check-cfg"));
        check_cfg_args.push(format!("cfg({})", scope_feature));
    }

    Ok((cfg_args, check_cfg_args))
}

// Helper function to iteratively parse feature dependencies
fn resolve_feature_dependencies(
    feature: &str,
    feature_map: &HashMap<String, Vec<String>>,
    enabled_features: &mut HashSet<String>,
) {
    // Avoid circular dependencies
    if enabled_features.contains(feature) {
        return;
    }

    enabled_features.insert(feature.to_string());

    // Recursively parse dependencies
    if let Some(dependencies) = feature_map.get(feature) {
        for dep in dependencies {
            resolve_feature_dependencies(dep, feature_map, enabled_features);
        }
    }
}

fn handle_workspace_package(
    root_cargo_toml: &CargoToml,
    root_manifest_path: &PathBuf,
    target_package: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut cfg_args = Vec::new();
    let mut all_scope_features = HashSet::new();
    let root_dir = root_manifest_path.parent().unwrap();

    // Always add default scope to check-cfg
    all_scope_features.insert("__scope_default".to_string());

    // Default enable __scope_default
    let mut enable_scope_default = true;

    // First collect information of all packages in the workspace
    let mut workspace_packages = HashMap::new();

    if let Some(workspace) = &root_cargo_toml.workspace {
        if let Some(members) = &workspace.members {
            for member_path in members {
                let member_manifest = root_dir.join(member_path).join("Cargo.toml");
                if member_manifest.exists() {
                    let content = std::fs::read_to_string(&member_manifest)?;
                    let member_cargo_toml: CargoToml = toml::from_str(&content)?;

                    if let Some(package) = member_cargo_toml.package {
                        workspace_packages.insert(package.name.clone(), (member_manifest, package));
                    }
                }
            }
        }
    }

    // Collect feature scopes defined in feature-scope-decl of all packages
    for (_, (_, package)) in &workspace_packages {
        if let Some(metadata) = &package.metadata {
            if let Some(feature_scope_decl) = &metadata.feature_scope_decl {
                // Collect all declared feature scopes
                for feature_name in feature_scope_decl.features.keys() {
                    all_scope_features.insert(format!("__scope_{}", feature_name));
                }

                if let Some(defaults) = &feature_scope_decl.default {
                    for feature in defaults {
                        all_scope_features.insert(format!("__scope_{}", feature));
                    }
                }
            }
        }
    }

    // Find target package
    let (_target_manifest_path, target_package_info) = workspace_packages
        .get(target_package)
        .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in workspace", target_package))?;

    // Process feature-scope configuration of target package
    if let Some(metadata) = &target_package_info.metadata {
        if let Some(feature_scope) = &metadata.feature_scope {
            for scope in feature_scope {
                // Find feature-scope-decl of dependency package
                if let Some((_, dep_package)) = workspace_packages.get(&scope.package) {
                    if let Some(dep_metadata) = &dep_package.metadata {
                        if let Some(dep_feature_scope_decl) = &dep_metadata.feature_scope_decl {
                            // Check if default features are disabled
                            let scope_enable_default_features =
                                scope.default_features.unwrap_or(true);
                            if !scope_enable_default_features {
                                enable_scope_default = false;
                            }

                            // Cross-validate and parse explicitly specified feature dependencies
                            for feature in &scope.features {
                                if dep_feature_scope_decl.features.contains_key(feature)
                                    || dep_feature_scope_decl
                                        .default
                                        .as_ref()
                                        .map_or(false, |d| d.contains(feature))
                                {
                                    // Iteratively parse feature dependencies
                                    let mut enabled_features = HashSet::new();
                                    resolve_feature_dependencies(
                                        feature,
                                        &dep_feature_scope_decl.features,
                                        &mut enabled_features,
                                    );

                                    for enabled_feature in enabled_features {
                                        cfg_args.push(String::from("--cfg"));
                                        cfg_args.push(format!("__scope_{}", enabled_feature));
                                    }
                                } else {
                                    eprintln!(
                                        "Warning: feature '{}' not declared in package '{}'",
                                        feature, scope.package
                                    );
                                }
                            }

                            // If default features are enabled and no features are explicitly specified, handle default features
                            if scope_enable_default_features && scope.features.is_empty() {
                                if let Some(defaults) = &dep_feature_scope_decl.default {
                                    for default_feature in defaults {
                                        let mut enabled_features = HashSet::new();
                                        resolve_feature_dependencies(
                                            default_feature,
                                            &dep_feature_scope_decl.features,
                                            &mut enabled_features,
                                        );

                                        for enabled_feature in enabled_features {
                                            cfg_args.push(String::from("--cfg"));
                                            cfg_args.push(format!("__scope_{}", enabled_feature));
                                        }
                                    }
                                }
                            }
                        } else {
                            eprintln!(
                                "Warning: package '{}' does not have feature-scope-decl",
                                scope.package
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "Warning: dependency package '{}' not found in workspace",
                        scope.package
                    );
                }
            }
        }
    }

    // Finally decide whether to add __scope_default
    if enable_scope_default {
        cfg_args.insert(0, String::from("__scope_default"));
        cfg_args.insert(0, String::from("--cfg"));
    }

    // Build check-cfg parameters
    let mut check_cfg_args = Vec::new();
    for scope_feature in all_scope_features {
        check_cfg_args.push(String::from("--check-cfg"));
        check_cfg_args.push(format!("cfg({})", scope_feature));
    }

    Ok((cfg_args, check_cfg_args))
}

fn execute_cargo_command(
    command: &str,
    package: Option<&String>,
    cfg_args: &[String],
    check_cfg_args: &[String],
    additional_args: &[&String],
) -> Result<()> {
    let mut cargo_cmd = process::Command::new("cargo");
    cargo_cmd.arg(command);

    // Add package arguments
    if let Some(pkg) = package {
        cargo_cmd.arg("-p").arg(pkg);
    }

    // Pass cfg and check-cfg parameters through RUSTFLAGS environment variable
    if !cfg_args.is_empty() || !check_cfg_args.is_empty() {
        let mut rustflags = env::var("RUSTFLAGS").unwrap_or_default();

        // Add cfg parameters
        for cfg_arg in cfg_args {
            if !rustflags.is_empty() {
                rustflags.push(' ');
            }
            rustflags.push_str(cfg_arg);
        }

        // Add check-cfg parameters
        for check_cfg_arg in check_cfg_args {
            if !rustflags.is_empty() {
                rustflags.push(' ');
            }
            rustflags.push_str(check_cfg_arg);
        }

        cargo_cmd.env("RUSTFLAGS", rustflags);
    }

    // Add additional arguments
    for arg in additional_args {
        cargo_cmd.arg(arg);
    }

    println!("Running: {:?}", cargo_cmd);
    if !cfg_args.is_empty() {
        println!("cfg_args: {:?}", cfg_args);
    }
    if !check_cfg_args.is_empty() {
        println!("check_cfg_args: {:?}", check_cfg_args);
    }

    // Execute cargo command
    let status = cargo_cmd
        .status()
        .with_context(|| "Failed to execute cargo command")?;

    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
