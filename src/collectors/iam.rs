// IAM (Identity and Access Management) resource collector - OPTIMIZED VERSION
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct IamCollector;

#[async_trait]
impl ResourceCollector for IamCollector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Note: IAM is a global service, region parameter is ignored
        
        // Collect IAM users (basic list only - NO per-user details to save time)
        if let Ok(users) = cli.execute(&["iam", "list-users"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "users".to_string(),
                resources: users,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect IAM roles (basic list only - NO per-role details to save time)
        if let Ok(roles) = cli.execute(&["iam", "list-roles"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "roles".to_string(),
                resources: roles,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect IAM groups (basic list only)
        if let Ok(groups) = cli.execute(&["iam", "list-groups"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "groups".to_string(),
                resources: groups,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect IAM policies (local only)
        if let Ok(policies) = cli.execute(&["iam", "list-policies", "--scope", "Local"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "policies".to_string(),
                resources: policies,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect SAML providers
        if let Ok(saml_providers) = cli.execute(&["iam", "list-saml-providers"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "saml-providers".to_string(),
                resources: saml_providers,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect OIDC providers
        if let Ok(oidc_providers) = cli.execute(&["iam", "list-open-id-connect-providers"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "oidc-providers".to_string(),
                resources: oidc_providers,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect instance profiles
        if let Ok(instance_profiles) = cli.execute(&["iam", "list-instance-profiles"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "instance-profiles".to_string(),
                resources: instance_profiles,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect account password policy
        if let Ok(password_policy) = cli.execute(&["iam", "get-account-password-policy"]).await {
            collections.push(ResourceCollection {
                service: "iam".to_string(),
                region: "global".to_string(),
                resource_type: "password-policy".to_string(),
                resources: password_policy,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
