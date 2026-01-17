// Parallel execution helpers for AWS resource collection
use futures::stream::{self, StreamExt};
use serde_json::Value;
use std::future::Future;
use crate::aws_cli::AwsCli;

/// Configuration for fetching a single detail field
#[derive(Clone)]
pub struct DetailConfig {
    /// Name of the field to insert into the result
    pub field_name: String,
    /// AWS CLI command arguments
    pub command: Vec<String>,
    /// Whether to ignore errors (continue if this command fails)
    pub ignore_errors: bool,
}

impl DetailConfig {
    /// Create a new DetailConfig
    pub fn new(field_name: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            field_name: field_name.into(),
            command,
            ignore_errors: true, // Default to ignoring errors for optional fields
        }
    }
}

/// Fetch details for multiple resources in parallel
/// 
/// # Arguments
/// * `resources` - Vector of resources to process
/// * `concurrency` - Maximum number of concurrent operations
/// * `fetch_fn` - Async function to fetch details for each resource
/// 
/// # Returns
/// Vector of detailed resources in the same order as input
pub async fn fetch_details_parallel<T, F, Fut>(
    resources: Vec<T>,
    concurrency: usize,
    fetch_fn: F,
) -> Vec<Value>
where
    T: Send,
    F: Fn(T) -> Fut,
    Fut: Future<Output = Value> + Send,
{
    let futures = resources.into_iter().map(fetch_fn);
    
    stream::iter(futures)
        .buffer_unordered(concurrency)
        .collect()
        .await
}

/// Fetch multiple detail fields for a single resource in parallel
/// 
/// # Arguments
/// * `cli` - AWS CLI instance
/// * `region` - AWS region
/// * `base_resource` - Base resource object to augment with details
/// * `detail_configs` - Vector of detail configurations to fetch
/// 
/// # Returns
/// Resource object with all detail fields added
pub async fn fetch_resource_details(
    cli: &AwsCli,
    region: &str,
    base_resource: Value,
    detail_configs: Vec<DetailConfig>,
) -> Value {
    let mut resource = base_resource;
    let obj = resource.as_object_mut().expect("Resource must be a JSON object");
    
    // Execute all detail fetches in parallel
    let cli_clone = cli.clone();
    let region_clone = region.to_string();
    
    let detail_futures: Vec<_> = detail_configs
        .into_iter()
        .map(|config| {
            let cli = cli_clone.clone();
            let region = region_clone.clone();
            
            async move {
                let mut command = config.command.clone();
                // Add region if not already present
                if !command.contains(&"--region".to_string()) {
                    command.push("--region".to_string());
                    command.push(region);
                }
                
                let result = cli.execute(&command.iter().map(|s| s.as_str()).collect::<Vec<_>>()).await;
                
                match result {
                    Ok(value) => Some((config.field_name, value)),
                    Err(_) if config.ignore_errors => None,
                    Err(e) => {
                        eprintln!("Error fetching {}: {}", config.field_name, e);
                        None
                    }
                }
            }
        })
        .collect();
    
    // Collect all results
    let results: Vec<Option<(String, Value)>> = stream::iter(detail_futures)
        .buffer_unordered(10) // Parallel detail fetching within a resource
        .collect()
        .await;
    
    // Insert successful results into the resource object
    for result in results.into_iter().flatten() {
        obj.insert(result.0, result.1);
    }
    
    resource
}

/// Helper to extract array from JSON response
pub fn extract_array(response: &Value, key: &str) -> Option<Vec<Value>> {
    response.get(key)?.as_array().map(|arr| arr.clone())
}

/// Helper to extract string field from JSON value
pub fn extract_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(|s| s.to_string())
}

/// Build detail config for a resource-specific command
/// 
/// # Arguments
/// * `field_name` - Name of the field to insert
/// * `service` - AWS service name (e.g., "s3api", "lambda")
/// * `operation` - Operation name (e.g., "get-bucket-versioning")
/// * `resource_param` - Parameter name for the resource (e.g., "--bucket", "--function-name")
/// * `resource_value` - Value of the resource identifier
/// 
/// # Returns
/// DetailConfig ready to use
pub fn build_detail_config(
    field_name: impl Into<String>,
    service: &str,
    operation: &str,
    resource_param: &str,
    resource_value: &str,
) -> DetailConfig {
    DetailConfig::new(
        field_name,
        vec![
            service.to_string(),
            operation.to_string(),
            resource_param.to_string(),
            resource_value.to_string(),
        ],
    )
}
