use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct WorkspacePackage {
    pub name: String,
    pub manifest_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub packages: HashMap<String, WorkspacePackage>,
}

#[derive(Deserialize)]
struct CargoToml {
    workspace: Option<WorkspaceConfig>,
    package: Option<PackageConfig>,
}

#[derive(Deserialize)]
struct WorkspaceConfig {
    members: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct PackageConfig {
    name: String,
}

/// Parse workspace information, returning a mapping of all package names and paths
/// If the current project is not part of a workspace, returns None
pub fn parse_workspace() -> Result<Option<WorkspaceInfo>> {
    // 1. Run cargo locate-project --workspace --message-format=plain to get workspace root directory
    let output = Command::new("cargo")
        .args(&["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .context("Failed to execute cargo locate-project command")?;

    if !output.status.success() {
        // If command fails, it means the current project is not part of a workspace
        return Ok(None);
    }

    let workspace_manifest_path = String::from_utf8(output.stdout)
        .context("Failed to parse cargo locate-project output as UTF-8")?
        .trim()
        .to_string();

    let workspace_manifest_path = PathBuf::from(workspace_manifest_path);
    let workspace_root = workspace_manifest_path
        .parent()
        .context("Failed to get workspace root directory")?
        .to_path_buf();

    // 2. Read and parse workspace Cargo.toml
    let workspace_toml_content = std::fs::read_to_string(&workspace_manifest_path)
        .context("Failed to read workspace Cargo.toml")?;

    let workspace_toml: CargoToml =
        toml::from_str(&workspace_toml_content).context("Failed to parse workspace Cargo.toml")?;

    let workspace_config = workspace_toml
        .workspace
        .context("No workspace configuration found in Cargo.toml")?;

    let members = workspace_config.members.unwrap_or_default();

    // 3. Parse all member packages
    let mut packages = HashMap::new();

    for member_pattern in members {
        let member_paths = resolve_member_pattern(&workspace_root, &member_pattern)?;

        for member_path in member_paths {
            let manifest_path = member_path.join("Cargo.toml");

            if manifest_path.exists() {
                if let Ok(package) = parse_package_info(&manifest_path) {
                    packages.insert(package.name.clone(), package);
                }
            }
        }
    }

    Ok(Some(WorkspaceInfo { packages }))
}

/// Parse member patterns (supports wildcards)
fn resolve_member_pattern(workspace_root: &Path, pattern: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if pattern.contains('*') {
        // Handle wildcard patterns
        let pattern_path = workspace_root.join(pattern);
        let parent_dir = pattern_path
            .parent()
            .context("Failed to get parent directory of pattern")?;

        if parent_dir.exists() {
            for entry in std::fs::read_dir(parent_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.is_dir() {
                    let manifest_path = path.join("Cargo.toml");
                    if manifest_path.exists() {
                        paths.push(path);
                    }
                }
            }
        }
    } else {
        // Direct path
        let member_path = workspace_root.join(pattern);
        if member_path.exists() && member_path.is_dir() {
            paths.push(member_path);
        }
    }

    Ok(paths)
}

/// Parse individual package information
fn parse_package_info(manifest_path: &Path) -> Result<WorkspacePackage> {
    let toml_content =
        std::fs::read_to_string(manifest_path).context("Failed to read package Cargo.toml")?;

    let package_toml: CargoToml =
        toml::from_str(&toml_content).context("Failed to parse package Cargo.toml")?;

    let package_config = package_toml
        .package
        .context("No package configuration found in Cargo.toml")?;

    Ok(WorkspacePackage {
        name: package_config.name,
        manifest_path: manifest_path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_workspace() {
        // This test needs to be run in an actual workspace environment
        match parse_workspace() {
            Ok(Some(workspace)) => {
                println!(
                    "Found workspace with {} packages:",
                    workspace.packages.len()
                );
                for (name, package) in &workspace.packages {
                    println!("  {} -> {:?}", name, package.manifest_path);
                }
            }
            Ok(None) => {
                println!("Not in a workspace");
            }
            Err(e) => {
                eprintln!("Error parsing workspace: {}", e);
            }
        }
    }
}
