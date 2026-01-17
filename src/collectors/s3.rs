// S3 resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct S3Collector;

#[async_trait]
impl ResourceCollector for S3Collector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // S3 is global, but we only collect once (not per region)
        
        // Collect S3 buckets with detailed information
        if let Ok(buckets_response) = cli.execute(&["s3api", "list-buckets"]).await {
            let buckets = crate::parallel::extract_array(&buckets_response, "Buckets")
                .unwrap_or_default();
            
            // Process buckets in parallel with concurrency limit of 5
            let detailed_buckets = crate::parallel::fetch_details_parallel(
                buckets,
                5,
                |bucket| {
                    let cli = cli.clone();
                    async move {
                        let bucket_name = match crate::parallel::extract_string(&bucket, "Name") {
                            Some(name) => name,
                            None => return bucket,
                        };
                        
                        // Define all detail fetching operations
                        let detail_configs = vec![
                            crate::parallel::build_detail_config("Location", "s3api", "get-bucket-location", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Versioning", "s3api", "get-bucket-versioning", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Encryption", "s3api", "get-bucket-encryption", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Lifecycle", "s3api", "get-bucket-lifecycle-configuration", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Logging", "s3api", "get-bucket-logging", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Tags", "s3api", "get-bucket-tagging", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("ACL", "s3api", "get-bucket-acl", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Policy", "s3api", "get-bucket-policy", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("CORS", "s3api", "get-bucket-cors", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Website", "s3api", "get-bucket-website", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("PublicAccessBlock", "s3api", "get-public-access-block", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("Replication", "s3api", "get-bucket-replication", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("NotificationConfiguration", "s3api", "get-bucket-notification-configuration", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("InventoryConfigurations", "s3api", "list-bucket-inventory-configurations", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("AnalyticsConfigurations", "s3api", "list-bucket-analytics-configurations", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("MetricsConfigurations", "s3api", "list-bucket-metrics-configurations", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("IntelligentTieringConfigurations", "s3api", "list-bucket-intelligent-tiering-configurations", "--bucket", &bucket_name),
                            crate::parallel::build_detail_config("ObjectLockConfiguration", "s3api", "get-object-lock-configuration", "--bucket", &bucket_name),
                        ];
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, "us-east-1", bucket, detail_configs).await
                    }
                },
            ).await;
            
            let detailed_response = json!({
                "Buckets": detailed_buckets,
                "Owner": buckets_response.get("Owner")
            });
            
            collections.push(ResourceCollection {
                service: "s3".to_string(),
                region: "global".to_string(),
                resource_type: "buckets".to_string(),
                resources: detailed_response,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
