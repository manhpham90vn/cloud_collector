// RDS resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct RdsCollector;

#[async_trait]
impl ResourceCollector for RdsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect RDS instances
        if let Ok(instances) = cli.execute(&["rds", "describe-db-instances", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-instances".to_string(),
                resources: instances,
                collected_at: timestamp.clone(),
            });
        }

        // Collect RDS clusters
        if let Ok(clusters) = cli.execute(&["rds", "describe-db-clusters", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-clusters".to_string(),
                resources: clusters,
                collected_at: timestamp.clone(),
            });
        }

        // Collect RDS snapshots
        if let Ok(snapshots) = cli.execute(&["rds", "describe-db-snapshots", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-snapshots".to_string(),
                resources: snapshots,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect RDS cluster snapshots
        if let Ok(cluster_snapshots) = cli.execute(&["rds", "describe-db-cluster-snapshots", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-cluster-snapshots".to_string(),
                resources: cluster_snapshots,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect DB subnet groups
        if let Ok(subnet_groups) = cli.execute(&["rds", "describe-db-subnet-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-subnet-groups".to_string(),
                resources: subnet_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect DB parameter groups
        if let Ok(parameter_groups) = cli.execute(&["rds", "describe-db-parameter-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-parameter-groups".to_string(),
                resources: parameter_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect DB cluster parameter groups
        if let Ok(cluster_parameter_groups) = cli.execute(&["rds", "describe-db-cluster-parameter-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-cluster-parameter-groups".to_string(),
                resources: cluster_parameter_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect option groups
        if let Ok(option_groups) = cli.execute(&["rds", "describe-option-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "option-groups".to_string(),
                resources: option_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect DB security groups
        if let Ok(security_groups) = cli.execute(&["rds", "describe-db-security-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-security-groups".to_string(),
                resources: security_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect DB proxies
        if let Ok(proxies) = cli.execute(&["rds", "describe-db-proxies", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "db-proxies".to_string(),
                resources: proxies,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect event subscriptions
        if let Ok(event_subscriptions) = cli.execute(&["rds", "describe-event-subscriptions", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "event-subscriptions".to_string(),
                resources: event_subscriptions,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect reserved DB instances
        if let Ok(reserved_instances) = cli.execute(&["rds", "describe-reserved-db-instances", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "rds".to_string(),
                region: region.to_string(),
                resource_type: "reserved-db-instances".to_string(),
                resources: reserved_instances,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
