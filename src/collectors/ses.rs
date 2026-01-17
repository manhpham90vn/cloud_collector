// SES (Simple Email Service) resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct SesCollector;

#[async_trait]
impl ResourceCollector for SesCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect verified email identities
        if let Ok(identities) = cli.execute(&[
            "ses",
            "list-identities",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ses".to_string(),
                region: region.to_string(),
                resource_type: "identities".to_string(),
                resources: identities,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect configuration sets
        if let Ok(config_sets) = cli.execute(&[
            "ses",
            "list-configuration-sets",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ses".to_string(),
                region: region.to_string(),
                resource_type: "configuration-sets".to_string(),
                resources: config_sets,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect receipt rule sets
        if let Ok(receipt_rule_sets) = cli.execute(&[
            "ses",
            "list-receipt-rule-sets",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ses".to_string(),
                region: region.to_string(),
                resource_type: "receipt-rule-sets".to_string(),
                resources: receipt_rule_sets,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect email templates
        if let Ok(templates) = cli.execute(&[
            "ses",
            "list-templates",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ses".to_string(),
                region: region.to_string(),
                resource_type: "templates".to_string(),
                resources: templates,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect custom verification email templates
        if let Ok(custom_templates) = cli.execute(&[
            "ses",
            "list-custom-verification-email-templates",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ses".to_string(),
                region: region.to_string(),
                resource_type: "custom-verification-email-templates".to_string(),
                resources: custom_templates,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
