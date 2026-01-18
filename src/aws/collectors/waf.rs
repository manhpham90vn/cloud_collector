// WAF (Web Application Firewall) resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct WafCollector;

#[async_trait]
impl ResourceCollector for WafCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut builder = CollectorBuilder::new("waf", RegionMode::Regional)
            .add_simple_list(
                "web-acls-regional",
                vec!["wafv2", "list-web-acls", "--scope", "REGIONAL"],
            )
            .add_simple_list(
                "ip-sets",
                vec!["wafv2", "list-ip-sets", "--scope", "REGIONAL"],
            )
            .add_simple_list(
                "regex-pattern-sets",
                vec!["wafv2", "list-regex-pattern-sets", "--scope", "REGIONAL"],
            )
            .add_simple_list(
                "rule-groups",
                vec!["wafv2", "list-rule-groups", "--scope", "REGIONAL"],
            );

        // Add CloudFront ACLs only for us-east-1
        if region == "us-east-1" {
            builder = builder.add_simple_list(
                "web-acls-cloudfront",
                vec!["wafv2", "list-web-acls", "--scope", "CLOUDFRONT"],
            );
        }

        builder.collect_with_region(cli, region).await
    }
}
