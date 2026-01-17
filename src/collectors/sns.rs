// SNS (Simple Notification Service) resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct SnsCollector;

#[async_trait]
impl ResourceCollector for SnsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect SNS topics with detailed information
        if let Ok(topics_response) = cli.execute(&[
            "sns",
            "list-topics",
            "--region",
            region
        ]).await {
            let topics = crate::parallel::extract_array(&topics_response, "Topics")
                .unwrap_or_default();
            
            // Process topics in parallel with concurrency limit of 10
            let detailed_topics = crate::parallel::fetch_details_parallel(
                topics,
                10,
                |topic| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let topic_arn = match crate::parallel::extract_string(&topic, "TopicArn") {
                            Some(arn) => arn,
                            None => return topic,
                        };
                        
                        // Define all detail fetching operations
                        let detail_configs = vec![
                            crate::parallel::DetailConfig::new(
                                "Attributes",
                                vec![
                                    "sns".to_string(),
                                    "get-topic-attributes".to_string(),
                                    "--topic-arn".to_string(),
                                    topic_arn.clone(),
                                ],
                            ),
                            crate::parallel::DetailConfig::new(
                                "Subscriptions",
                                vec![
                                    "sns".to_string(),
                                    "list-subscriptions-by-topic".to_string(),
                                    "--topic-arn".to_string(),
                                    topic_arn.clone(),
                                ],
                            ),
                            crate::parallel::DetailConfig::new(
                                "Tags",
                                vec![
                                    "sns".to_string(),
                                    "list-tags-for-resource".to_string(),
                                    "--resource-arn".to_string(),
                                    topic_arn,
                                ],
                            ),
                        ];
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, topic, detail_configs).await
                    }
                },
            ).await;
            
            collections.push(ResourceCollection {
                service: "sns".to_string(),
                region: region.to_string(),
                resource_type: "topics".to_string(),
                resources: json!({ "Topics": detailed_topics }),
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect platform applications
        if let Ok(platform_apps) = cli.execute(&[
            "sns",
            "list-platform-applications",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "sns".to_string(),
                region: region.to_string(),
                resource_type: "platform-applications".to_string(),
                resources: platform_apps,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
