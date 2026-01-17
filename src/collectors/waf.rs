// WAF (Web Application Firewall) resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct WafCollector;

#[async_trait]
impl ResourceCollector for WafCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect WAFv2 Web ACLs (Regional)
        if let Ok(web_acls_response) = cli.execute(&[
            "wafv2",
            "list-web-acls",
            "--scope",
            "REGIONAL",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "waf".to_string(),
                region: region.to_string(),
                resource_type: "web-acls-regional".to_string(),
                resources: web_acls_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect WAFv2 Web ACLs (CloudFront - only in us-east-1)
        if region == "us-east-1" {
            if let Ok(cloudfront_acls) = cli.execute(&[
                "wafv2",
                "list-web-acls",
                "--scope",
                "CLOUDFRONT",
                "--region",
                "us-east-1"
            ]).await {
                collections.push(ResourceCollection {
                    service: "waf".to_string(),
                    region: "global".to_string(),
                    resource_type: "web-acls-cloudfront".to_string(),
                    resources: cloudfront_acls,
                    collected_at: timestamp.clone(),
                });
            }
        }
        
        // Collect IP sets (Regional)
        if let Ok(ip_sets) = cli.execute(&[
            "wafv2",
            "list-ip-sets",
            "--scope",
            "REGIONAL",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "waf".to_string(),
                region: region.to_string(),
                resource_type: "ip-sets".to_string(),
                resources: ip_sets,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect regex pattern sets (Regional)
        if let Ok(regex_sets) = cli.execute(&[
            "wafv2",
            "list-regex-pattern-sets",
            "--scope",
            "REGIONAL",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "waf".to_string(),
                region: region.to_string(),
                resource_type: "regex-pattern-sets".to_string(),
                resources: regex_sets,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect rule groups (Regional)
        if let Ok(rule_groups) = cli.execute(&[
            "wafv2",
            "list-rule-groups",
            "--scope",
            "REGIONAL",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "waf".to_string(),
                region: region.to_string(),
                resource_type: "rule-groups".to_string(),
                resources: rule_groups,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
