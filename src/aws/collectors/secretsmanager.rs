// Secrets Manager resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct SecretsManagerCollector;

#[async_trait]
impl ResourceCollector for SecretsManagerCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        // Collect secrets list (metadata only, not values)
        CollectorBuilder::new("secretsmanager", RegionMode::Regional)
            .add_simple_list("secrets", vec!["secretsmanager", "list-secrets"])
            .collect_with_region(cli, region)
            .await
    }
}
