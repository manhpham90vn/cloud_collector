// ElastiCache resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct ElastiCacheCollector;

#[async_trait]
impl ResourceCollector for ElastiCacheCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect cache clusters (Redis and Memcached)
        if let Ok(clusters_response) = cli.execute(&[
            "elasticache",
            "describe-cache-clusters",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "cache-clusters".to_string(),
                resources: clusters_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect replication groups (Redis only)
        if let Ok(replication_groups) = cli.execute(&[
            "elasticache",
            "describe-replication-groups",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "replication-groups".to_string(),
                resources: replication_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect cache subnet groups
        if let Ok(subnet_groups) = cli.execute(&[
            "elasticache",
            "describe-cache-subnet-groups",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "cache-subnet-groups".to_string(),
                resources: subnet_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect cache parameter groups
        if let Ok(parameter_groups) = cli.execute(&[
            "elasticache",
            "describe-cache-parameter-groups",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "cache-parameter-groups".to_string(),
                resources: parameter_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect cache security groups
        if let Ok(security_groups) = cli.execute(&[
            "elasticache",
            "describe-cache-security-groups",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "cache-security-groups".to_string(),
                resources: security_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect snapshots
        if let Ok(snapshots) = cli.execute(&[
            "elasticache",
            "describe-snapshots",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "snapshots".to_string(),
                resources: snapshots,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect user groups (Redis 6+ only)
        if let Ok(user_groups) = cli.execute(&[
            "elasticache",
            "describe-user-groups",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "elasticache".to_string(),
                region: region.to_string(),
                resource_type: "user-groups".to_string(),
                resources: user_groups,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
