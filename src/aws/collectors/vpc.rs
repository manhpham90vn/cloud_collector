// VPC resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct VpcCollector;

#[async_trait]
impl ResourceCollector for VpcCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("vpc", RegionMode::Regional)
            .add_batch_commands(vec![
                ("vpcs", vec!["ec2", "describe-vpcs"]),
                ("subnets", vec!["ec2", "describe-subnets"]),
                ("route-tables", vec!["ec2", "describe-route-tables"]),
                (
                    "internet-gateways",
                    vec!["ec2", "describe-internet-gateways"],
                ),
                ("nat-gateways", vec!["ec2", "describe-nat-gateways"]),
                ("network-acls", vec!["ec2", "describe-network-acls"]),
                ("vpc-endpoints", vec!["ec2", "describe-vpc-endpoints"]),
                (
                    "vpc-peering-connections",
                    vec!["ec2", "describe-vpc-peering-connections"],
                ),
                ("vpn-gateways", vec!["ec2", "describe-vpn-gateways"]),
                (
                    "customer-gateways",
                    vec!["ec2", "describe-customer-gateways"],
                ),
            ])
            .collect_with_region(cli, region)
            .await
    }
}
