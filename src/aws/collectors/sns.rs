// SNS (Simple Notification Service) resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, DetailTemplate, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct SnsCollector;

#[async_trait]
impl ResourceCollector for SnsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("sns", RegionMode::Regional)
            // Detailed collection for topics
            .add_detailed_resource(
                "topics",
                vec!["sns", "list-topics"],
                "Topics",
                "TopicArn",
                10, // concurrency
                vec![
                    DetailTemplate::new("Attributes", "sns", "get-topic-attributes", "--topic-arn"),
                    DetailTemplate::new(
                        "Subscriptions",
                        "sns",
                        "list-subscriptions-by-topic",
                        "--topic-arn",
                    ),
                    DetailTemplate::new("Tags", "sns", "list-tags-for-resource", "--resource-arn"),
                ],
            )
            // Simple list for platform applications
            .add_simple_list(
                "platform-applications",
                vec!["sns", "list-platform-applications"],
            )
            .collect_with_region(cli, region)
            .await
    }
}
