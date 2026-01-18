// CloudWatch resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, DetailTemplate, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct CloudWatchCollector;

#[async_trait]
impl ResourceCollector for CloudWatchCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("cloudwatch", RegionMode::Regional)
            // Simple lists
            .add_simple_list("alarms", vec!["cloudwatch", "describe-alarms"])
            .add_simple_list("dashboards", vec!["cloudwatch", "list-dashboards"])
            .add_simple_list("metric-streams", vec!["cloudwatch", "list-metric-streams"])
            .add_simple_list(
                "insights-rules",
                vec!["cloudwatch", "describe-insight-rules"],
            )
            // Detailed collection for log groups
            .add_detailed_resource(
                "log-groups",
                vec!["logs", "describe-log-groups"],
                "logGroups",
                "logGroupName",
                10, // concurrency
                vec![
                    DetailTemplate::new(
                        "MetricFilters",
                        "logs",
                        "describe-metric-filters",
                        "--log-group-name",
                    ),
                    DetailTemplate::new(
                        "SubscriptionFilters",
                        "logs",
                        "describe-subscription-filters",
                        "--log-group-name",
                    ),
                ],
            )
            .collect_with_region(cli, region)
            .await
    }
}
