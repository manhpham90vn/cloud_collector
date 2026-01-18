// ElastiCache resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct ElastiCacheCollector;

#[async_trait]
impl ResourceCollector for ElastiCacheCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("elasticache", RegionMode::Regional)
            .add_batch_commands(vec![
                (
                    "cache-clusters",
                    vec!["elasticache", "describe-cache-clusters"],
                ),
                (
                    "replication-groups",
                    vec!["elasticache", "describe-replication-groups"],
                ),
                (
                    "cache-subnet-groups",
                    vec!["elasticache", "describe-cache-subnet-groups"],
                ),
                (
                    "cache-parameter-groups",
                    vec!["elasticache", "describe-cache-parameter-groups"],
                ),
                (
                    "cache-security-groups",
                    vec!["elasticache", "describe-cache-security-groups"],
                ),
                ("snapshots", vec!["elasticache", "describe-snapshots"]),
                ("user-groups", vec!["elasticache", "describe-user-groups"]),
            ])
            .collect_with_region(cli, region)
            .await
    }
}
