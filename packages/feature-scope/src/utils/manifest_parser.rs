use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureScopeRef {
    pub package: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FeatureScopeDecl {
    pub features: HashMap<String, Vec<String>>,
}

impl Default for FeatureScopeDecl {
    fn default() -> Self {
        let mut features = HashMap::new();
        features.insert("default".to_string(), Vec::new());
        Self { features }
    }
}

#[derive(Debug, Clone)]
pub struct ManifestMetadata {
    pub feature_scope_decl: Option<FeatureScopeDecl>,
    pub feature_scope_refs: Vec<FeatureScopeRef>,
}

/// Parse feature-scope related metadata from the specified Cargo.toml file path
///
/// # Arguments
///
/// * `manifest_path` - Path to the Cargo.toml file
///
/// # Returns
///
/// Returns parsed metadata including:
/// - `feature_scope_decl`: Content that this package declares as acceptable conditional compilation flags
/// - `feature_scope_refs`: Other packages referenced by this package and their required features
pub fn parse_manifest(manifest_path: &Path) -> Result<ManifestMetadata> {
    let toml_content = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read manifest file: {}", manifest_path.display()))?;

    let toml_value: toml::Value = toml::from_str(&toml_content)
        .with_context(|| format!("Failed to parse TOML in: {}", manifest_path.display()))?;

    let metadata = toml_value.get("package").and_then(|p| p.get("metadata"));

    let feature_scope_decl = parse_feature_scope_decl(metadata)?;
    let feature_scope_refs = parse_feature_scope_refs(metadata)?;

    Ok(ManifestMetadata {
        feature_scope_decl,
        feature_scope_refs,
    })
}

/// Parse metadata.feature-scope-decl field
fn parse_feature_scope_decl(metadata: Option<&toml::Value>) -> Result<Option<FeatureScopeDecl>> {
    let Some(metadata) = metadata else {
        return Ok(None);
    };

    let Some(decl_value) = metadata.get("feature-scope-decl") else {
        return Ok(None);
    };

    let Some(decl_table) = decl_value.as_table() else {
        return Err(anyhow::anyhow!(
            "metadata.feature-scope-decl must be a table"
        ));
    };

    let mut features = HashMap::new();

    for (key, value) in decl_table {
        let feature_list = value
            .as_array()
            .with_context(|| format!("Feature '{}' in feature-scope-decl must be an array", key))?;

        let feature_strings: Result<Vec<String>> = feature_list
            .iter()
            .enumerate()
            .map(|(i, v)| {
                v.as_str().map(String::from).with_context(|| {
                    format!(
                        "Feature '{}' item {} in feature-scope-decl must be a string",
                        key, i
                    )
                })
            })
            .collect();

        features.insert(key.clone(), feature_strings?);
    }

    // If no default field is declared, default to empty array
    if !features.contains_key("default") {
        features.insert("default".to_string(), Vec::new());
    }

    Ok(Some(FeatureScopeDecl { features }))
}

/// Parse metadata.feature-scope field
fn parse_feature_scope_refs(metadata: Option<&toml::Value>) -> Result<Vec<FeatureScopeRef>> {
    let Some(metadata) = metadata else {
        return Ok(Vec::new());
    };

    let Some(refs_value) = metadata.get("feature-scope") else {
        return Ok(Vec::new());
    };

    let Some(refs_array) = refs_value.as_array() else {
        return Err(anyhow::anyhow!("metadata.feature-scope must be an array"));
    };

    let mut refs = Vec::new();

    for (i, ref_value) in refs_array.iter().enumerate() {
        let feature_ref = FeatureScopeRef::deserialize(ref_value.clone())
            .with_context(|| format!("Failed to parse feature-scope reference at index {}", i))?;

        refs.push(feature_ref);
    }

    Ok(refs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest_with_decl() {
        // Create a temporary Cargo.toml content for testing
        let toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"

[package.metadata.feature-scope-decl]
default = ["feature1"]
optional = ["feature2", "feature3"]
extra = []

[[package.metadata.feature-scope]]
package = "other-package"
features = ["feature1", "feature2"]

[[package.metadata.feature-scope]]
package = "another-package"
features = ["feature3"]
"#;

        let toml_value: toml::Value = toml::from_str(toml_content).unwrap();
        let metadata = toml_value.get("package").and_then(|p| p.get("metadata"));

        let decl = parse_feature_scope_decl(metadata).unwrap();
        assert!(decl.is_some());

        let decl = decl.unwrap();
        assert_eq!(
            decl.features.get("default"),
            Some(&vec!["feature1".to_string()])
        );
        assert_eq!(
            decl.features.get("optional"),
            Some(&vec!["feature2".to_string(), "feature3".to_string()])
        );
        assert_eq!(decl.features.get("extra"), Some(&vec![]));

        let refs = parse_feature_scope_refs(metadata).unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].package, "other-package");
        assert_eq!(refs[0].features, vec!["feature1", "feature2"]);
        assert_eq!(refs[1].package, "another-package");
        assert_eq!(refs[1].features, vec!["feature3"]);
    }

    #[test]
    fn test_parse_manifest_without_metadata() {
        let toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
"#;

        let toml_value: toml::Value = toml::from_str(toml_content).unwrap();
        let metadata = toml_value.get("package").and_then(|p| p.get("metadata"));

        let decl = parse_feature_scope_decl(metadata).unwrap();
        assert!(decl.is_none());

        let refs = parse_feature_scope_refs(metadata).unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_manifest_with_default_feature() {
        let toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"

[package.metadata.feature-scope-decl]
optional = ["feature1"]
"#;

        let toml_value: toml::Value = toml::from_str(toml_content).unwrap();
        let metadata = toml_value.get("package").and_then(|p| p.get("metadata"));

        let decl = parse_feature_scope_decl(metadata).unwrap();
        assert!(decl.is_some());

        let decl = decl.unwrap();
        // Should automatically add default = []
        assert_eq!(decl.features.get("default"), Some(&vec![]));
        assert_eq!(
            decl.features.get("optional"),
            Some(&vec!["feature1".to_string()])
        );
    }
}
