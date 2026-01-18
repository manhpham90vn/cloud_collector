// SES (Simple Email Service) resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct SesCollector;

#[async_trait]
impl ResourceCollector for SesCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("ses", RegionMode::Regional)
            .add_simple_list("identities", vec!["ses", "list-identities"])
            .add_simple_list("configuration-sets", vec!["ses", "list-configuration-sets"])
            .add_simple_list("receipt-rule-sets", vec!["ses", "list-receipt-rule-sets"])
            .add_simple_list("templates", vec!["ses", "list-templates"])
            .add_simple_list(
                "custom-verification-email-templates",
                vec!["ses", "list-custom-verification-email-templates"],
            )
            .collect_with_region(cli, region)
            .await
    }
}
