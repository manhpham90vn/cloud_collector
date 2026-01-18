// AWS-specific parallel execution adapters
//
// This module provides AWS CLI-specific implementations using the generic
// parallel utilities from utils::parallel

use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use crate::utils::parallel;
use futures::stream::{self, StreamExt};
use serde_json::Value;

/// Default concurrency for AWS resource collection
const AWS_COLLECTION_CONCURRENCY: usize = 10;

/// Default concurrency for AWS detail fetching
const AWS_DETAIL_CONCURRENCY: usize = 10;

// ============================================================================
// Configuration Types
// ============================================================================

/// Configuration for fetching a single detail field from AWS
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
    /// Create a new DetailConfig with default error handling (ignore errors)
    pub fn new(field_name: impl Into<String>, command: Vec<String>) -> Self {
        Self {
            field_name: field_name.into(),
            command,
            ignore_errors: true,
        }
    }
}

// ============================================================================
// Resource Detail Fetching
// ============================================================================

/// Fetch details for multiple AWS resources in parallel
///
/// # Arguments
/// * `resources` - Vector of resources to process
/// * `concurrency` - Maximum number of concurrent operations
/// * `fetch_fn` - Async function to fetch details for each resource
///
/// # Returns
/// Vector of detailed resources (order not preserved)
pub async fn fetch_details_parallel<T, F, Fut>(
    resources: Vec<T>,
    concurrency: usize,
    fetch_fn: F,
) -> Vec<Value>
where
    T: Send,
    F: Fn(T) -> Fut,
    Fut: std::future::Future<Output = Value> + Send,
{
    parallel::execute_parallel(resources, concurrency, fetch_fn).await
}

/// Fetch multiple detail fields for a single AWS resource in parallel
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
    let obj = resource
        .as_object_mut()
        .expect("Resource must be a JSON object");

    // Execute all detail fetches in parallel using generic utilities
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

                let args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
                let result = cli.execute(&args).await;

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
        .buffer_unordered(AWS_DETAIL_CONCURRENCY)
        .collect()
        .await;

    // Insert successful results into the resource object
    for result in results.into_iter().flatten() {
        obj.insert(result.0, result.1);
    }

    resource
}

// ============================================================================
// Resource Collection
// ============================================================================

/// Execute multiple independent AWS CLI commands in parallel
///
/// # Arguments
/// * `cli` - AWS CLI instance
/// * `region` - AWS region
/// * `service` - Service name for all collections
/// * `timestamp` - Timestamp for all collections
/// * `commands` - Vector of (resource_type, aws_cli_args) tuples
///
/// # Returns
/// Vector of ResourceCollections (only successful ones)
pub async fn collect_resources_parallel(
    cli: &AwsCli,
    region: &str,
    service: &str,
    timestamp: &str,
    commands: Vec<(&str, Vec<&str>)>,
) -> Vec<ResourceCollection> {
    let futures: Vec<_> = commands
        .into_iter()
        .map(|(resource_type, args)| {
            let cli = cli.clone();
            let region = region.to_string();
            let service = service.to_string();
            let timestamp = timestamp.to_string();
            let resource_type = resource_type.to_string();

            async move {
                cli.execute(&args)
                    .await
                    .ok()
                    .map(|resources| ResourceCollection {
                        service,
                        region,
                        resource_type,
                        resources,
                        collected_at: timestamp,
                    })
            }
        })
        .collect();

    stream::iter(futures)
        .buffer_unordered(AWS_COLLECTION_CONCURRENCY)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

// ============================================================================
// JSON Helper Functions
// ============================================================================

/// Extract array from JSON response
pub fn extract_array(response: &Value, key: &str) -> Option<Vec<Value>> {
    response.get(key)?.as_array().cloned()
}

/// Extract string field from JSON value
pub fn extract_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(String::from)
}
