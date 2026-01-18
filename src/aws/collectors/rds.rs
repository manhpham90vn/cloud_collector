// RDS resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct RdsCollector;

#[async_trait]
impl ResourceCollector for RdsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("rds", RegionMode::Regional)
            .add_batch_commands(vec![
                ("db-instances", vec!["rds", "describe-db-instances"]),
                ("db-clusters", vec!["rds", "describe-db-clusters"]),
                ("db-snapshots", vec!["rds", "describe-db-snapshots"]),
                (
                    "db-cluster-snapshots",
                    vec!["rds", "describe-db-cluster-snapshots"],
                ),
                ("db-subnet-groups", vec!["rds", "describe-db-subnet-groups"]),
                (
                    "db-parameter-groups",
                    vec!["rds", "describe-db-parameter-groups"],
                ),
                (
                    "db-cluster-parameter-groups",
                    vec!["rds", "describe-db-cluster-parameter-groups"],
                ),
                ("option-groups", vec!["rds", "describe-option-groups"]),
                (
                    "db-security-groups",
                    vec!["rds", "describe-db-security-groups"],
                ),
                ("db-proxies", vec!["rds", "describe-db-proxies"]),
                (
                    "event-subscriptions",
                    vec!["rds", "describe-event-subscriptions"],
                ),
                (
                    "reserved-db-instances",
                    vec!["rds", "describe-reserved-db-instances"],
                ),
            ])
            .collect_with_region(cli, region)
            .await
    }
}
