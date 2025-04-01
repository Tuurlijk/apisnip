use color_eyre::eyre::{self, Result};
use indexmap::IndexMap;
use serde_json;
use serde_yaml::{Mapping, Value};
use std::fs;
use std::path::Path;

pub fn read_spec(path: &str) -> Result<Mapping> {
    let input_content = fs::read_to_string(path)?;

    // Detect file extension and parse accordingly
    match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("json") => {
            let json_value: serde_json::Value = serde_json::from_str(&input_content)?;
            // Convert JSON to YAML while preserving order
            let yaml_str = serde_yaml::to_string(&json_value)?;
            let value: Value = serde_yaml::from_str(&yaml_str)?;
            if let Value::Mapping(mapping) = value {
                // Convert to IndexMap to preserve order
                let ordered_mapping =
                    Mapping::from_iter(mapping.into_iter().collect::<IndexMap<_, _>>());
                Ok(ordered_mapping)
            } else {
                Err(eyre::eyre!("JSON did not convert to a YAML mapping"))
            }
        }
        Some("yaml") | Some("yml") => {
            let value: Value = serde_yaml::from_str(&input_content)?;
            if let Value::Mapping(mapping) = value {
                // Convert to IndexMap to preserve order
                let ordered_mapping =
                    Mapping::from_iter(mapping.into_iter().collect::<IndexMap<_, _>>());
                Ok(ordered_mapping)
            } else {
                Err(eyre::eyre!("YAML did not parse to a mapping"))
            }
        }
        _ => Err(eyre::eyre!(
            "Unsupported file format. Please use .json, .yaml, or .yml files"
        )),
    }
}

pub fn write_spec(path: &str, spec: &Mapping) -> Result<()> {
    let output_content = match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("json") => {
            let json_value = serde_json::to_value(spec)?;
            serde_json::to_string_pretty(&json_value)?
        }
        Some("yaml") | Some("yml") => {
            // Convert to IndexMap to preserve order when writing
            let ordered_mapping = Mapping::from_iter(
                spec.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<IndexMap<_, _>>(),
            );
            serde_yaml::to_string(&ordered_mapping)?
        }
        _ => {
            return Err(eyre::eyre!(
                "Unsupported output format. Please use .json, .yaml, or .yml files"
            ));
        }
    };

    fs::write(path, output_content)?;
    Ok(())
}
