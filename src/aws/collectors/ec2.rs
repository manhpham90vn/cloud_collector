// EC2 resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct Ec2Collector;

#[async_trait]
impl ResourceCollector for Ec2Collector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("ec2", RegionMode::Regional)
            .add_batch_commands(vec![
                ("instances", vec!["ec2", "describe-instances"]),
                ("vpcs", vec!["ec2", "describe-vpcs"]),
                ("subnets", vec!["ec2", "describe-subnets"]),
                ("route-tables", vec!["ec2", "describe-route-tables"]),
                (
                    "internet-gateways",
                    vec!["ec2", "describe-internet-gateways"],
                ),
                ("nat-gateways", vec!["ec2", "describe-nat-gateways"]),
                ("network-acls", vec!["ec2", "describe-network-acls"]),
                ("security-groups", vec!["ec2", "describe-security-groups"]),
                ("vpc-endpoints", vec!["ec2", "describe-vpc-endpoints"]),
                ("elastic-ips", vec!["ec2", "describe-addresses"]),
                ("volumes", vec!["ec2", "describe-volumes"]),
                (
                    "snapshots",
                    vec!["ec2", "describe-snapshots", "--owner-ids", "self"],
                ),
                ("images", vec!["ec2", "describe-images", "--owners", "self"]),
                ("key-pairs", vec!["ec2", "describe-key-pairs"]),
                (
                    "network-interfaces",
                    vec!["ec2", "describe-network-interfaces"],
                ),
                ("launch-templates", vec!["ec2", "describe-launch-templates"]),
                (
                    "auto-scaling-groups",
                    vec!["autoscaling", "describe-auto-scaling-groups"],
                ),
                ("placement-groups", vec!["ec2", "describe-placement-groups"]),
                (
                    "vpc-peering-connections",
                    vec!["ec2", "describe-vpc-peering-connections"],
                ),
                (
                    "transit-gateway-attachments",
                    vec!["ec2", "describe-transit-gateway-attachments"],
                ),
                ("vpn-connections", vec!["ec2", "describe-vpn-connections"]),
                (
                    "customer-gateways",
                    vec!["ec2", "describe-customer-gateways"],
                ),
            ])
            .collect_with_region(cli, region)
            .await
    }
}
