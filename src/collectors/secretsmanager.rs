// Secrets Manager resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct SecretsManagerCollector;

#[async_trait]
impl ResourceCollector for SecretsManagerCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect secrets list (metadata only, not values)
        if let Ok(secrets_response) = cli.execute(&[
            "secretsmanager",
            "list-secrets",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "secretsmanager".to_string(),
                region: region.to_string(),
                resource_type: "secrets".to_string(),
                resources: secrets_response,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
