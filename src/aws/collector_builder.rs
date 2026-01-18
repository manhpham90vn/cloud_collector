// Collector Builder Framework
//
// This module provides a declarative way to build AWS resource collectors
// with minimal boilerplate code.

use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use crate::parallel;
use anyhow::Result;
use serde_json::json;

/// Region mode for collector
#[derive(Debug, Clone)]
pub enum RegionMode {
    /// Use the provided region parameter
    Regional,
    /// Always use "global" as region
    Global,
    /// Use a specific custom region (e.g., "us-east-1" for S3)
    Custom(String),
}

/// Collection mode for a resource type
#[derive(Debug, Clone)]
pub enum CollectionMode {
    /// Simple list - just execute command and wrap in ResourceCollection
    SimpleList { command: Vec<String> },

    /// Batch commands - execute multiple commands in parallel
    BatchCommands {
        commands: Vec<(String, Vec<String>)>, // (resource_type, command)
    },

    /// List with details - list resources then fetch details for each
    ListWithDetails {
        list_command: Vec<String>,
        array_key: String,
        identifier_key: String,
        detail_templates: Vec<DetailTemplate>,
        concurrency: usize,
    },
}

/// Template for fetching detail information
#[derive(Debug, Clone)]
pub struct DetailTemplate {
    pub field_name: String,
    pub service: String,
    pub operation: String,
    pub param_name: String,
}

impl DetailTemplate {
    pub fn new(
        field_name: impl Into<String>,
        service: impl Into<String>,
        operation: impl Into<String>,
        param_name: impl Into<String>,
    ) -> Self {
        Self {
            field_name: field_name.into(),
            service: service.into(),
            operation: operation.into(),
            param_name: param_name.into(),
        }
    }
}

/// Parameters for collecting resources with details
struct DetailCollectionParams<'a> {
    service: &'a str,
    cli: &'a AwsCli,
    region: &'a str,
    resource_type: &'a str,
    timestamp: &'a str,
    list_command: Vec<String>,
    array_key: &'a str,
    identifier_key: &'a str,
    detail_templates: Vec<DetailTemplate>,
    concurrency: usize,
}

/// Resource configuration
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub resource_type: String,
    pub mode: CollectionMode,
}

/// Main collector builder
pub struct CollectorBuilder {
    service: String,
    region_mode: RegionMode,
    resources: Vec<ResourceConfig>,
}

impl CollectorBuilder {
    /// Create a new collector builder
    pub fn new(service: impl Into<String>, region_mode: RegionMode) -> Self {
        Self {
            service: service.into(),
            region_mode,
            resources: Vec::new(),
        }
    }

    /// Add a simple list resource
    pub fn add_simple_list(
        mut self,
        resource_type: impl Into<String>,
        command: Vec<impl Into<String>>,
    ) -> Self {
        self.resources.push(ResourceConfig {
            resource_type: resource_type.into(),
            mode: CollectionMode::SimpleList {
                command: command.into_iter().map(|s| s.into()).collect(),
            },
        });
        self
    }

    /// Add batch commands (multiple resource types collected in parallel)
    pub fn add_batch_commands(mut self, commands: Vec<(&str, Vec<&str>)>) -> Self {
        let batch_commands: Vec<(String, Vec<String>)> = commands
            .into_iter()
            .map(|(rt, cmd)| {
                (
                    rt.to_string(),
                    cmd.into_iter().map(|s| s.to_string()).collect(),
                )
            })
            .collect();

        self.resources.push(ResourceConfig {
            resource_type: "batch".to_string(), // Special marker
            mode: CollectionMode::BatchCommands {
                commands: batch_commands,
            },
        });
        self
    }

    /// Add a resource with detailed information fetching
    pub fn add_detailed_resource(
        mut self,
        resource_type: impl Into<String>,
        list_command: Vec<impl Into<String>>,
        array_key: impl Into<String>,
        identifier_key: impl Into<String>,
        concurrency: usize,
        detail_templates: Vec<DetailTemplate>,
    ) -> Self {
        self.resources.push(ResourceConfig {
            resource_type: resource_type.into(),
            mode: CollectionMode::ListWithDetails {
                list_command: list_command.into_iter().map(|s| s.into()).collect(),
                array_key: array_key.into(),
                identifier_key: identifier_key.into(),
                detail_templates,
                concurrency,
            },
        });
        self
    }

    /// Collect all resources
    pub async fn collect(self, cli: &AwsCli) -> Result<Vec<ResourceCollection>> {
        self.collect_with_region(cli, "").await
    }

    /// Collect all resources with a specific region
    pub async fn collect_with_region(
        mut self,
        cli: &AwsCli,
        region: &str,
    ) -> Result<Vec<ResourceCollection>> {
        let mut all_collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Determine the actual region to use
        let actual_region = match &self.region_mode {
            RegionMode::Regional => region.to_string(),
            RegionMode::Global => "global".to_string(),
            RegionMode::Custom(r) => r.clone(),
        };

        // Store service name to avoid borrow issues
        let service = self.service.clone();

        for resource_config in self.resources.drain(..) {
            match resource_config.mode {
                CollectionMode::SimpleList { command } => {
                    let collections = Self::collect_simple_list_impl(
                        &service,
                        cli,
                        &actual_region,
                        &resource_config.resource_type,
                        &timestamp,
                        command,
                    )
                    .await;
                    all_collections.extend(collections);
                }

                CollectionMode::BatchCommands { commands } => {
                    let collections = Self::collect_batch_commands_impl(
                        &service,
                        cli,
                        &actual_region,
                        &timestamp,
                        commands,
                    )
                    .await;
                    all_collections.extend(collections);
                }

                CollectionMode::ListWithDetails {
                    list_command,
                    array_key,
                    identifier_key,
                    detail_templates,
                    concurrency,
                } => {
                    let collections = Self::collect_with_details_impl(DetailCollectionParams {
                        service: &service,
                        cli,
                        region: &actual_region,
                        resource_type: &resource_config.resource_type,
                        timestamp: &timestamp,
                        list_command,
                        array_key: &array_key,
                        identifier_key: &identifier_key,
                        detail_templates,
                        concurrency,
                    })
                    .await;
                    all_collections.extend(collections);
                }
            }
        }

        Ok(all_collections)
    }

    // Private helper methods

    async fn collect_simple_list_impl(
        service: &str,
        cli: &AwsCli,
        region: &str,
        resource_type: &str,
        timestamp: &str,
        command: Vec<String>,
    ) -> Vec<ResourceCollection> {
        let args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();

        if let Ok(resources) = cli.execute(&args).await {
            vec![ResourceCollection {
                service: service.to_string(),
                region: region.to_string(),
                resource_type: resource_type.to_string(),
                resources,
                collected_at: timestamp.to_string(),
            }]
        } else {
            Vec::new()
        }
    }

    async fn collect_batch_commands_impl(
        service: &str,
        cli: &AwsCli,
        region: &str,
        timestamp: &str,
        commands: Vec<(String, Vec<String>)>,
    ) -> Vec<ResourceCollection> {
        let commands_with_refs: Vec<(&str, Vec<&str>)> = commands
            .iter()
            .map(|(rt, cmd)| (rt.as_str(), cmd.iter().map(|s| s.as_str()).collect()))
            .collect();

        parallel::collect_resources_parallel(cli, region, service, timestamp, commands_with_refs)
            .await
    }

    async fn collect_with_details_impl(
        params: DetailCollectionParams<'_>,
    ) -> Vec<ResourceCollection> {
        // Execute list command
        let args: Vec<&str> = params.list_command.iter().map(|s| s.as_str()).collect();
        let list_response = match params.cli.execute(&args).await {
            Ok(resp) => resp,
            Err(_) => return Vec::new(),
        };

        // Extract array of resources
        let resources = match parallel::extract_array(&list_response, params.array_key) {
            Some(arr) => arr,
            None => return Vec::new(),
        };

        // Fetch details for each resource in parallel
        let cli_clone = params.cli.clone();
        let region_clone = params.region.to_string();
        let identifier_key_clone = params.identifier_key.to_string();
        let detail_templates = params.detail_templates;

        let detailed_resources =
            parallel::fetch_details_parallel(resources, params.concurrency, move |resource| {
                let cli = cli_clone.clone();
                let region = region_clone.clone();
                let identifier_key = identifier_key_clone.clone();
                let templates = detail_templates.clone();

                async move {
                    // Extract identifier
                    let identifier = match parallel::extract_string(&resource, &identifier_key) {
                        Some(id) => id,
                        None => return resource,
                    };

                    // Build detail configs from templates
                    let detail_configs: Vec<parallel::DetailConfig> = templates
                        .iter()
                        .map(|template| {
                            parallel::DetailConfig::new(
                                &template.field_name,
                                vec![
                                    template.service.clone(),
                                    template.operation.clone(),
                                    template.param_name.clone(),
                                    identifier.clone(),
                                ],
                            )
                        })
                        .collect();

                    // Fetch all details
                    parallel::fetch_resource_details(&cli, &region, resource, detail_configs).await
                }
            })
            .await;

        // Wrap in response object
        let response = json!({
            params.array_key: detailed_resources
        });

        vec![ResourceCollection {
            service: params.service.to_string(),
            region: params.region.to_string(),
            resource_type: params.resource_type.to_string(),
            resources: response,
            collected_at: params.timestamp.to_string(),
        }]
    }
}
