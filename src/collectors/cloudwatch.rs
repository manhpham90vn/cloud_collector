// CloudWatch resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct CloudWatchCollector;

#[async_trait]
impl ResourceCollector for CloudWatchCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect CloudWatch alarms
        if let Ok(alarms_response) = cli.execute(&[
            "cloudwatch",
            "describe-alarms",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "cloudwatch".to_string(),
                region: region.to_string(),
                resource_type: "alarms".to_string(),
                resources: alarms_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect CloudWatch Logs log groups with detailed information
        if let Ok(log_groups_response) = cli.execute(&[
            "logs",
            "describe-log-groups",
            "--region",
            region
        ]).await {
            let log_groups = crate::parallel::extract_array(&log_groups_response, "logGroups")
                .unwrap_or_default();
            
            // Process log groups in parallel with concurrency limit of 10
            let detailed_log_groups = crate::parallel::fetch_details_parallel(
                log_groups,
                10,
                |log_group| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let log_group_name = match crate::parallel::extract_string(&log_group, "logGroupName") {
                            Some(name) => name,
                            None => return log_group,
                        };
                        
                        // Define all detail fetching operations
                        let detail_configs = vec![
                            crate::parallel::build_detail_config("MetricFilters", "logs", "describe-metric-filters", "--log-group-name", &log_group_name),
                            crate::parallel::build_detail_config("SubscriptionFilters", "logs", "describe-subscription-filters", "--log-group-name", &log_group_name),
                        ];
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, log_group, detail_configs).await
                    }
                },
            ).await;
            
            collections.push(ResourceCollection {
                service: "cloudwatch".to_string(),
                region: region.to_string(),
                resource_type: "log-groups".to_string(),
                resources: json!({ "logGroups": detailed_log_groups }),
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect CloudWatch dashboards
        if let Ok(dashboards) = cli.execute(&[
            "cloudwatch",
            "list-dashboards",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "cloudwatch".to_string(),
                region: region.to_string(),
                resource_type: "dashboards".to_string(),
                resources: dashboards,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect metric streams
        if let Ok(metric_streams) = cli.execute(&[
            "cloudwatch",
            "list-metric-streams",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "cloudwatch".to_string(),
                region: region.to_string(),
                resource_type: "metric-streams".to_string(),
                resources: metric_streams,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Insights rules
        if let Ok(insights_rules) = cli.execute(&[
            "cloudwatch",
            "describe-insight-rules",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "cloudwatch".to_string(),
                region: region.to_string(),
                resource_type: "insights-rules".to_string(),
                resources: insights_rules,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
