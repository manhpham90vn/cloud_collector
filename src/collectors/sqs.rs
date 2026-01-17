// SQS (Simple Queue Service) resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct SqsCollector;

#[async_trait]
impl ResourceCollector for SqsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect SQS queues with detailed information
        if let Ok(queues_response) = cli.execute(&[
            "sqs",
            "list-queues",
            "--region",
            region
        ]).await {
            let mut detailed_queues = Vec::new();
            
            if let Some(queue_urls) = queues_response.get("QueueUrls").and_then(|q| q.as_array()) {
                for queue_url_value in queue_urls {
                    if let Some(queue_url) = queue_url_value.as_str() {
                        let mut queue_details = json!({
                            "QueueUrl": queue_url
                        });
                        
                        // Get queue attributes
                        if let Ok(attributes) = cli.execute(&[
                            "sqs",
                            "get-queue-attributes",
                            "--queue-url",
                            queue_url,
                            "--attribute-names",
                            "All",
                            "--region",
                            region
                        ]).await {
                            queue_details.as_object_mut().unwrap().insert("Attributes".to_string(), attributes);
                        }
                        
                        // Get tags
                        if let Ok(tags) = cli.execute(&[
                            "sqs",
                            "list-queue-tags",
                            "--queue-url",
                            queue_url,
                            "--region",
                            region
                        ]).await {
                            queue_details.as_object_mut().unwrap().insert("Tags".to_string(), tags);
                        }
                        
                        detailed_queues.push(queue_details);
                    }
                }
            }
            
            collections.push(ResourceCollection {
                service: "sqs".to_string(),
                region: region.to_string(),
                resource_type: "queues".to_string(),
                resources: json!({ "Queues": detailed_queues }),
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
