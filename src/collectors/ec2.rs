// EC2 resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct Ec2Collector;

#[async_trait]
impl ResourceCollector for Ec2Collector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect EC2 instances
        if let Ok(instances) = cli.execute(&["ec2", "describe-instances", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "instances".to_string(),
                resources: instances,
                collected_at: timestamp.clone(),
            });
        }

        // Collect VPCs
        if let Ok(vpcs) = cli.execute(&["ec2", "describe-vpcs", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "vpcs".to_string(),
                resources: vpcs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Subnets
        if let Ok(subnets) = cli.execute(&["ec2", "describe-subnets", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "subnets".to_string(),
                resources: subnets,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Route Tables
        if let Ok(route_tables) = cli.execute(&["ec2", "describe-route-tables", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "route-tables".to_string(),
                resources: route_tables,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Internet Gateways
        if let Ok(igws) = cli.execute(&["ec2", "describe-internet-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "internet-gateways".to_string(),
                resources: igws,
                collected_at: timestamp.clone(),
            });
        }

        // Collect NAT Gateways
        if let Ok(nat_gws) = cli.execute(&["ec2", "describe-nat-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "nat-gateways".to_string(),
                resources: nat_gws,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Network ACLs
        if let Ok(nacls) = cli.execute(&["ec2", "describe-network-acls", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "network-acls".to_string(),
                resources: nacls,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Security Groups
        if let Ok(sgs) = cli.execute(&["ec2", "describe-security-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "security-groups".to_string(),
                resources: sgs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect VPC Endpoints
        if let Ok(vpc_endpoints) = cli.execute(&["ec2", "describe-vpc-endpoints", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "vpc-endpoints".to_string(),
                resources: vpc_endpoints,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Elastic IPs
        if let Ok(eips) = cli.execute(&["ec2", "describe-addresses", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "elastic-ips".to_string(),
                resources: eips,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Volumes
        if let Ok(volumes) = cli.execute(&["ec2", "describe-volumes", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "volumes".to_string(),
                resources: volumes,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Snapshots (owned by self)
        if let Ok(snapshots) = cli.execute(&["ec2", "describe-snapshots", "--owner-ids", "self", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "snapshots".to_string(),
                resources: snapshots,
                collected_at: timestamp.clone(),
            });
        }

        // Collect AMIs (owned by self)
        if let Ok(images) = cli.execute(&["ec2", "describe-images", "--owners", "self", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "images".to_string(),
                resources: images,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Key Pairs
        if let Ok(keypairs) = cli.execute(&["ec2", "describe-key-pairs", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "key-pairs".to_string(),
                resources: keypairs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Network Interfaces
        if let Ok(enis) = cli.execute(&["ec2", "describe-network-interfaces", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "network-interfaces".to_string(),
                resources: enis,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Launch Templates
        if let Ok(launch_templates) = cli.execute(&["ec2", "describe-launch-templates", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "launch-templates".to_string(),
                resources: launch_templates,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Auto Scaling Groups
        if let Ok(asgs) = cli.execute(&["autoscaling", "describe-auto-scaling-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "auto-scaling-groups".to_string(),
                resources: asgs,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Placement Groups
        if let Ok(placement_groups) = cli.execute(&["ec2", "describe-placement-groups", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "placement-groups".to_string(),
                resources: placement_groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect VPC Peering Connections
        if let Ok(vpc_peering) = cli.execute(&["ec2", "describe-vpc-peering-connections", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "vpc-peering-connections".to_string(),
                resources: vpc_peering,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Transit Gateway Attachments
        if let Ok(tgw_attachments) = cli.execute(&["ec2", "describe-transit-gateway-attachments", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "transit-gateway-attachments".to_string(),
                resources: tgw_attachments,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect VPN Connections
        if let Ok(vpn_connections) = cli.execute(&["ec2", "describe-vpn-connections", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "vpn-connections".to_string(),
                resources: vpn_connections,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect Customer Gateways
        if let Ok(customer_gateways) = cli.execute(&["ec2", "describe-customer-gateways", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ec2".to_string(),
                region: region.to_string(),
                resource_type: "customer-gateways".to_string(),
                resources: customer_gateways,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
