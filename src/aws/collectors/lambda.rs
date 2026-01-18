// Lambda resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, DetailTemplate, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct LambdaCollector;

#[async_trait]
impl ResourceCollector for LambdaCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("lambda", RegionMode::Regional)
            // Detailed collection for functions
            .add_detailed_resource(
                "functions",
                vec!["lambda", "list-functions"],
                "Functions",
                "FunctionName",
                10, // concurrency
                vec![
                    DetailTemplate::new(
                        "Configuration",
                        "lambda",
                        "get-function-configuration",
                        "--function-name",
                    ),
                    DetailTemplate::new("Aliases", "lambda", "list-aliases", "--function-name"),
                    DetailTemplate::new(
                        "Versions",
                        "lambda",
                        "list-versions-by-function",
                        "--function-name",
                    ),
                    DetailTemplate::new(
                        "EventSourceMappings",
                        "lambda",
                        "list-event-source-mappings",
                        "--function-name",
                    ),
                    DetailTemplate::new(
                        "FunctionUrlConfig",
                        "lambda",
                        "get-function-url-config",
                        "--function-name",
                    ),
                    DetailTemplate::new(
                        "Concurrency",
                        "lambda",
                        "get-function-concurrency",
                        "--function-name",
                    ),
                ],
            )
            // Simple lists for layers and code signing
            .add_simple_list("layers", vec!["lambda", "list-layers"])
            .add_simple_list(
                "code-signing-configs",
                vec!["lambda", "list-code-signing-configs"],
            )
            .collect_with_region(cli, region)
            .await
    }
}
