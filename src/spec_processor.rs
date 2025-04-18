use color_eyre::eyre::{OptionExt, Result};
use itertools::Itertools;
use serde_yaml::{Mapping, Value};

#[derive(Default, Clone)]
pub struct Endpoint {
    pub methods: Vec<Method>,
    pub path: String,
    pub description: String,
    pub refs: Vec<String>,
    pub status: Status,
    pub parameters: Vec<String>,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    #[default]
    Unselected,
    Selected,
}

#[derive(Default, Clone)]
pub struct Method {
    pub method: String,
    pub description: String,
}

pub fn fetch_endpoints_from_spec(spec: &Mapping) -> Vec<Endpoint> {
    let mut table_items: Vec<Endpoint> = Vec::new();
    let paths = spec
        .get(Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .ok_or_eyre("No 'paths' field found or it's not a mapping")
        .unwrap();

    for (path, ops) in paths {
        let path_str = path
            .as_str()
            .ok_or_eyre("Path key is not a string")
            .unwrap();
        let mut table_item = Endpoint::default();
        let ops_map = ops
            .as_mapping()
            .ok_or_eyre(format!("Operations for '{}' not a mapping", path_str))
            .unwrap();
        let mut refs: Vec<String> = Vec::new();
        for (ops_method, op) in ops_map {
            let method_str = ops_method
                .as_str()
                .ok_or_eyre("Method key is not a string")
                .unwrap();
                        
            if method_str == "summary" {
                table_item.description = op.as_str().unwrap_or("").to_string();
                continue;
            }
            if method_str == "description" && table_item.description.is_empty() {
                table_item.description = op.as_str().unwrap_or("").to_string();
                continue;
            }
            
            let mut method = Method::default();
            let summary = op
                .as_mapping()
                .and_then(|m| m.get(Value::String("summary".to_string())))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let description = op
                .as_mapping()
                .and_then(|m| m.get(Value::String("description".to_string())))
                .and_then(|v| v.as_str())
                .unwrap_or("No description");
            
            // Check for operation-level parameters
            if let Some(op_map) = op.as_mapping() {
                if let Some(params) = op_map.get(Value::String("parameters".to_string())) {
                    if let Some(params_array) = params.as_sequence() {
                        extract_parameters(params_array, &mut table_item.parameters);
                    }
                }
            }
            
            method.method = method_str.to_string();
            method.description = if !summary.is_empty() {
                summary.to_string()
            } else {
                description.to_string()
            };
            refs.extend(fetch_all_references(op));
            table_item.methods.push(method);
        }
        table_item.path = path_str.to_string();
        table_item.refs = strip_path_from_references(&refs)
            .into_iter()
            .unique()
            .collect();
        table_items.push(table_item);
    }

    // Order table items by path
    table_items.sort_by(|a, b| a.path.cmp(&b.path));
    table_items
}

// Helper function to extract parameter names from a parameters array
fn extract_parameters(params_array: &[Value], parameters: &mut Vec<String>) {
    for param in params_array {
        if let Some(param_map) = param.as_mapping() {
            // Get the parameter name
            if let Some(name_value) = param_map.get(Value::String("name".to_string())) {
                if let Some(name) = name_value.as_str() {
                    // Get the parameter location (in)
                    let prefix = if let Some(in_value) = param_map.get(Value::String("in".to_string())) {
                        if let Some(in_type) = in_value.as_str() {
                            match in_type {
                                "path" => "/",
                                "body" => "body:",
                                "query" => "?",
                                _ => "",
                            }
                        } else {
                            ""
                        }
                    } else {
                        ""
                    };
                    
                    // Add prefixed parameter name
                    parameters.push(format!("{}{}", prefix, name));
                }
            }
        }
    }
}

/// Recursively fetch all $ref values from a Value tree
fn fetch_all_references(value: &Value) -> Vec<String> {
    let mut refs = Vec::new();
    match value {
        Value::Mapping(map) => {
            // Check if this mapping has a $ref key
            if let Some(Value::String(ref_str)) = map.get(Value::String("$ref".to_string())) {
                refs.push(ref_str.clone());
            }
            // Recurse into all values in the mapping
            for (_, v) in map {
                refs.extend(fetch_all_references(v));
            }
        }
        Value::Sequence(seq) => {
            // Recurse into sequence items
            for item in seq {
                refs.extend(fetch_all_references(item));
            }
        }
        _ => {} // Scalars (String, Number, Bool, Null) have no refs
    }
    refs
}

fn strip_path_from_references(references: &[String]) -> Vec<String> {
    references
        .iter()
        .map(|ref_str| ref_str.split('/').next_back().unwrap().to_string())
        .collect::<Vec<String>>()
}

pub fn process_spec_for_output(spec: &Mapping, selected_items: &[&Endpoint]) -> Result<Mapping> {
    let original_path_specifications = spec
        .get(Value::String("paths".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap();

    // Create paths mapping with only selected paths
    let mut paths = Mapping::new();
    for item in selected_items {
        if let Some(path_data) = original_path_specifications.get(Value::String(item.path.clone()))
        {
            paths.insert(Value::String(item.path.clone()), path_data.clone());
        }
    }

    // Collect all references from selected items
    let collected_references: Vec<String> = selected_items
        .iter()
        .flat_map(|item| &item.refs)
        .cloned()
        .unique()
        .collect();

    // Collect all references to preserve
    let mut all_references_to_preserve = Vec::new();
    let components = spec
        .get(Value::String("components".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap();
    for (key, value) in components {
        if key.as_str() == Some("schemas") {
            if let Some(schema) = value.as_mapping() {
                for (schema_key, schema_value) in schema {
                    if collected_references.contains(&schema_key.as_str().unwrap().to_string()) {
                        all_references_to_preserve.extend(fetch_all_references(schema_value));
                    }
                }
            }
        }
    }

    // Process all references
    let mut all_references = strip_path_from_references(&all_references_to_preserve);
    all_references.extend(collected_references);
    all_references.sort();
    all_references.dedup();

    // Store the order of keys from the original spec
    let key_order: Vec<Value> = spec.keys().cloned().collect();

    // Create a new mapping that preserves the original order
    let mut output = Mapping::new();

    // Build the output in the original order
    for key in key_order {
        let value = spec.get(&key).unwrap();
        if key.as_str() == Some("paths") {
            // Replace paths with filtered version
            output.insert(key, Value::Mapping(paths.clone()));
        } else if key.as_str() == Some("components") {
            // Handle components section
            let mut components_output = Mapping::new();
            for (child_key, child_value) in value.as_mapping().unwrap() {
                if child_key.as_str() != Some("schemas") {
                    components_output.insert(child_key.clone(), child_value.clone());
                } else {
                    let mut schema_output = Mapping::new();
                    for (schema_key, schema_value) in child_value.as_mapping().unwrap() {
                        if all_references.contains(&schema_key.as_str().unwrap().to_string()) {
                            schema_output.insert(schema_key.clone(), schema_value.clone());
                        }
                    }
                    components_output.insert(child_key.clone(), Value::Mapping(schema_output));
                }
            }
            output.insert(key, Value::Mapping(components_output));
        } else {
            // Copy other sections as-is
            output.insert(key, value.clone());
        }
    }

    Ok(output)
}
