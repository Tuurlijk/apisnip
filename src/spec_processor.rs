use color_eyre::eyre::{OptionExt, Result};
use itertools::Itertools;
use serde_yaml::{Mapping, Value};
use std::collections::HashSet;

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

/// Extract component name and type from a $ref string
/// Returns (component_type, component_name) or None if not a component reference
fn parse_component_ref(ref_str: &str) -> Option<(String, String)> {
    if ref_str.starts_with("#/components/") {
        let parts: Vec<&str> = ref_str.split('/').collect();
        if parts.len() >= 4 {
            let component_type = parts[2].to_string();
            let component_name = parts[3..].join("/");
            return Some((component_type, component_name));
        }
    }
    None
}

/// Recursively collect all transitive component references
/// Returns a set of (component_type, component_name) tuples
fn collect_transitive_references(
    components: &Mapping,
    initial_refs: &[String],
) -> HashSet<(String, String)> {
    let mut all_refs = HashSet::new();
    let mut to_process: Vec<(String, String)> = Vec::new();

    // Parse initial references
    for ref_str in initial_refs {
        if let Some((comp_type, comp_name)) = parse_component_ref(ref_str) {
            let key = (comp_type, comp_name);
            if all_refs.insert(key.clone()) {
                to_process.push(key);
            }
        }
    }

    // Process references recursively
    while let Some((comp_type, comp_name)) = to_process.pop() {
        if let Some(comp_section) = components.get(Value::String(comp_type.clone())) {
            if let Some(comp_mapping) = comp_section.as_mapping() {
                if let Some(comp_value) = comp_mapping.get(Value::String(comp_name.clone())) {
                    // Extract all references from this component
                    for nested_ref in fetch_all_references(comp_value) {
                        if let Some((nested_type, nested_name)) = parse_component_ref(&nested_ref) {
                            let key = (nested_type.clone(), nested_name.clone());
                            if all_refs.insert(key.clone()) {
                                to_process.push(key);
                            }
                        }
                    }
                }
            }
        }
    }

    all_refs
}

/// Extract security scheme names from security requirements
fn extract_security_schemes(value: &Value) -> Vec<String> {
    let mut schemes = Vec::new();
    match value {
        Value::Sequence(seq) => {
            for item in seq {
                if let Some(map) = item.as_mapping() {
                    for (key, _) in map {
                        if let Some(scheme_name) = key.as_str() {
                            schemes.push(scheme_name.to_string());
                        }
                    }
                }
            }
        }
        Value::Mapping(map) => {
            for (key, _) in map {
                if let Some(scheme_name) = key.as_str() {
                    schemes.push(scheme_name.to_string());
                }
            }
        }
        _ => {}
    }
    schemes
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

    // Collect all $ref references from selected paths
    let initial_refs: Vec<String> = selected_items
        .iter()
        .filter_map(|item| original_path_specifications.get(Value::String(item.path.clone())))
        .flat_map(fetch_all_references)
        .collect();

    // Extract security scheme references from selected paths and top-level
    let mut security_schemes = HashSet::new();
    for item in selected_items {
        if let Some(path_data) = original_path_specifications.get(Value::String(item.path.clone())) {
            if let Some(path_map) = path_data.as_mapping() {
                for op_value in path_map.values() {
                    if let Some(op_map) = op_value.as_mapping() {
                        if let Some(security) = op_map.get(Value::String("security".to_string())) {
                            security_schemes.extend(extract_security_schemes(security));
                        }
                    }
                }
            }
        }
    }
    if let Some(security) = spec.get(Value::String("security".to_string())) {
        security_schemes.extend(extract_security_schemes(security));
    }

    // Get components section
    let empty_components = Mapping::new();
    let components = spec
        .get(Value::String("components".to_string()))
        .and_then(|v| v.as_mapping())
        .unwrap_or(&empty_components);

    // Collect all transitive component references
    let all_component_refs = collect_transitive_references(components, &initial_refs);

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
            if let Some(components_map) = value.as_mapping() {
                for (child_key, child_value) in components_map {
                    let child_key_str = child_key.as_str().unwrap_or("");
                    let mut filtered_section = Mapping::new();

                    if let Some(section_map) = child_value.as_mapping() {
                        for (item_key, item_value) in section_map {
                            let item_key_str = item_key.as_str().unwrap_or("");
                            let lookup_key = (child_key_str.to_string(), item_key_str.to_string());
                            let should_include = all_component_refs.contains(&lookup_key)
                                || (child_key_str == "securitySchemes" && security_schemes.contains(item_key_str));

                            if should_include {
                                filtered_section.insert(item_key.clone(), item_value.clone());
                            }
                        }
                    }

                    if !filtered_section.is_empty() {
                        components_output.insert(child_key.clone(), Value::Mapping(filtered_section));
                    }
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
