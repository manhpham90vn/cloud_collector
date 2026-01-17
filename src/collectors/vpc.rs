// VPC resource collector (separate from EC2)
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct VpcCollector;

#[async_trait]
impl ResourceCollector for VpcCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect VPCs
        if let Ok(vpcs) = cli.execute(&["ec2", "describe-vpcs", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "vpcs".to_string(),
                resources: vpcs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Subnets
        if let Ok(subnets) = cli.execute(&["ec2", "describe-subnets", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "subnets".to_string(),
                resources: subnets,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Route Tables
        if let Ok(route_tables) = cli.execute(&["ec2", "describe-route-tables", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "route-tables".to_string(),
                resources: route_tables,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Internet Gateways
        if let Ok(igws) = cli.execute(&["ec2", "describe-internet-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "internet-gateways".to_string(),
                resources: igws,
                collected_at: timestamp.clone(),
            });
        }

        // Collect NAT Gateways
        if let Ok(nat_gws) = cli.execute(&["ec2", "describe-nat-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "nat-gateways".to_string(),
                resources: nat_gws,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Network ACLs
        if let Ok(nacls) = cli.execute(&["ec2", "describe-network-acls", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "network-acls".to_string(),
                resources: nacls,
                collected_at: timestamp.clone(),
            });
        }

        // Collect VPC Endpoints
        if let Ok(vpc_endpoints) = cli.execute(&["ec2", "describe-vpc-endpoints", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "vpc-endpoints".to_string(),
                resources: vpc_endpoints,
                collected_at: timestamp.clone(),
            });
        }

        // Collect VPC Peering Connections
        if let Ok(peering) = cli.execute(&["ec2", "describe-vpc-peering-connections", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "vpc-peering-connections".to_string(),
                resources: peering,
                collected_at: timestamp.clone(),
            });
        }

        // Collect VPN Gateways
        if let Ok(vpn_gws) = cli.execute(&["ec2", "describe-vpn-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "vpn-gateways".to_string(),
                resources: vpn_gws,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Customer Gateways
        if let Ok(cgws) = cli.execute(&["ec2", "describe-customer-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "vpc".to_string(),
                region: region.to_string(),
                resource_type: "customer-gateways".to_string(),
                resources: cgws,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
