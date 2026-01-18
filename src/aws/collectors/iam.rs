// IAM (Identity and Access Management) resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct IamCollector;

#[async_trait]
impl ResourceCollector for IamCollector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        // Note: IAM is a global service, region parameter is ignored

        CollectorBuilder::new("iam", RegionMode::Global)
            .add_simple_list("users", vec!["iam", "list-users"])
            .add_simple_list("roles", vec!["iam", "list-roles"])
            .add_simple_list("groups", vec!["iam", "list-groups"])
            .add_simple_list("policies", vec!["iam", "list-policies", "--scope", "Local"])
            .add_simple_list("saml-providers", vec!["iam", "list-saml-providers"])
            .add_simple_list(
                "oidc-providers",
                vec!["iam", "list-open-id-connect-providers"],
            )
            .add_simple_list("instance-profiles", vec!["iam", "list-instance-profiles"])
            .add_simple_list(
                "password-policy",
                vec!["iam", "get-account-password-policy"],
            )
            .collect(cli)
            .await
    }
}
