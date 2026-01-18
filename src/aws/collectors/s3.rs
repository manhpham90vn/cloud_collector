// S3 resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, DetailTemplate, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct S3Collector;

#[async_trait]
impl ResourceCollector for S3Collector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        // S3 is global, but we use us-east-1 for API calls

        CollectorBuilder::new("s3", RegionMode::Custom("us-east-1".to_string()))
            .add_detailed_resource(
                "buckets",
                vec!["s3api", "list-buckets"],
                "Buckets",
                "Name",
                5, // concurrency
                vec![
                    DetailTemplate::new("Location", "s3api", "get-bucket-location", "--bucket"),
                    DetailTemplate::new("Versioning", "s3api", "get-bucket-versioning", "--bucket"),
                    DetailTemplate::new("Encryption", "s3api", "get-bucket-encryption", "--bucket"),
                    DetailTemplate::new(
                        "Lifecycle",
                        "s3api",
                        "get-bucket-lifecycle-configuration",
                        "--bucket",
                    ),
                    DetailTemplate::new("Logging", "s3api", "get-bucket-logging", "--bucket"),
                    DetailTemplate::new("Tags", "s3api", "get-bucket-tagging", "--bucket"),
                    DetailTemplate::new("ACL", "s3api", "get-bucket-acl", "--bucket"),
                    DetailTemplate::new("Policy", "s3api", "get-bucket-policy", "--bucket"),
                    DetailTemplate::new("CORS", "s3api", "get-bucket-cors", "--bucket"),
                    DetailTemplate::new("Website", "s3api", "get-bucket-website", "--bucket"),
                    DetailTemplate::new(
                        "PublicAccessBlock",
                        "s3api",
                        "get-public-access-block",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "Replication",
                        "s3api",
                        "get-bucket-replication",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "NotificationConfiguration",
                        "s3api",
                        "get-bucket-notification-configuration",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "InventoryConfigurations",
                        "s3api",
                        "list-bucket-inventory-configurations",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "AnalyticsConfigurations",
                        "s3api",
                        "list-bucket-analytics-configurations",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "MetricsConfigurations",
                        "s3api",
                        "list-bucket-metrics-configurations",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "IntelligentTieringConfigurations",
                        "s3api",
                        "list-bucket-intelligent-tiering-configurations",
                        "--bucket",
                    ),
                    DetailTemplate::new(
                        "ObjectLockConfiguration",
                        "s3api",
                        "get-object-lock-configuration",
                        "--bucket",
                    ),
                ],
            )
            .collect(cli)
            .await
    }
}
