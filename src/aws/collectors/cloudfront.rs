// CloudFront resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct CloudFrontCollector;

#[async_trait]
impl ResourceCollector for CloudFrontCollector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // CloudFront is a global service
        
        // Collect distributions
        if let Ok(distributions_response) = cli.execute(&["cloudfront", "list-distributions"]).await {
            let mut detailed_distributions = Vec::new();
            
            if let Some(dist_list) = distributions_response.get("DistributionList") {
                if let Some(distributions) = dist_list.get("Items").and_then(|d| d.as_array()) {
                    for distribution in distributions {
                        if let Some(dist_id) = distribution.get("Id").and_then(|id| id.as_str()) {
                            let mut dist_details = distribution.clone();
                            
                            // Get distribution configuration
                            if let Ok(config) = cli.execute(&[
                                "cloudfront",
                                "get-distribution-config",
                                "--id",
                                dist_id
                            ]).await {
                                dist_details.as_object_mut().unwrap().insert("Config".to_string(), config);
                            }
                            
                            // Get distribution tags
                            if let Ok(tags) = cli.execute(&[
                                "cloudfront",
                                "list-tags-for-resource",
                                "--resource",
                                &format!("arn:aws:cloudfront::*:distribution/{}", dist_id)
                            ]).await {
                                dist_details.as_object_mut().unwrap().insert("Tags".to_string(), tags);
                            }
                            
                            detailed_distributions.push(dist_details);
                        }
                    }
                }
            }
            
            let detailed_response = json!({
                "Distributions": detailed_distributions
            });
            
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "distributions".to_string(),
                resources: detailed_response,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect origin access identities
        if let Ok(oai_response) = cli.execute(&["cloudfront", "list-cloud-front-origin-access-identities"]).await {
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "origin-access-identities".to_string(),
                resources: oai_response,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect cache policies
        if let Ok(cache_policies) = cli.execute(&["cloudfront", "list-cache-policies"]).await {
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "cache-policies".to_string(),
                resources: cache_policies,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect origin request policies
        if let Ok(origin_request_policies) = cli.execute(&["cloudfront", "list-origin-request-policies"]).await {
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "origin-request-policies".to_string(),
                resources: origin_request_policies,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect response headers policies
        if let Ok(response_headers_policies) = cli.execute(&["cloudfront", "list-response-headers-policies"]).await {
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "response-headers-policies".to_string(),
                resources: response_headers_policies,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect CloudFront Functions
        if let Ok(functions) = cli.execute(&["cloudfront", "list-functions"]).await {
            collections.push(ResourceCollection {
                service: "cloudfront".to_string(),
                region: "global".to_string(),
                resource_type: "functions".to_string(),
                resources: functions,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
