// CloudFormation resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct CloudFormationCollector;

#[async_trait]
impl ResourceCollector for CloudFormationCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect CloudFormation stacks
        if let Ok(stacks) = cli.execute(&["cloudformation", "describe-stacks", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "cloudformation".to_string(),
                region: region.to_string(),
                resource_type: "stacks".to_string(),
                resources: stacks,
                collected_at: timestamp.clone(),
            });
        }

        // Collect CloudFormation stack sets
        if let Ok(stacksets) = cli.execute(&["cloudformation", "list-stack-sets", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "cloudformation".to_string(),
                region: region.to_string(),
                resource_type: "stack-sets".to_string(),
                resources: stacksets,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect exports
        if let Ok(exports) = cli.execute(&["cloudformation", "list-exports", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "cloudformation".to_string(),
                region: region.to_string(),
                resource_type: "exports".to_string(),
                resources: exports,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect change sets
        if let Ok(stacks_response) = cli.execute(&["cloudformation", "describe-stacks", "--region", region]).await {
            if let Some(stacks) = stacks_response.get("Stacks").and_then(|s| s.as_array()) {
                let mut all_change_sets = Vec::new();
                
                for stack in stacks {
                    if let Some(stack_name) = stack.get("StackName").and_then(|n| n.as_str()) {
                        if let Ok(change_sets) = cli.execute(&[
                            "cloudformation",
                            "list-change-sets",
                            "--stack-name",
                            stack_name,
                            "--region",
                            region
                        ]).await {
                            if let Some(cs_list) = change_sets.get("Summaries").and_then(|cs| cs.as_array()) {
                                all_change_sets.extend(cs_list.clone());
                            }
                        }
                    }
                }
                
                if !all_change_sets.is_empty() {
                    collections.push(ResourceCollection {
                        service: "cloudformation".to_string(),
                        region: region.to_string(),
                        resource_type: "change-sets".to_string(),
                        resources: serde_json::json!({ "ChangeSets": all_change_sets }),
                        collected_at: timestamp,
                    });
                }
            }
        }

        Ok(collections)
    }
}
