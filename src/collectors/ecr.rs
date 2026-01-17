// ECR resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct EcrCollector;

#[async_trait]
impl ResourceCollector for EcrCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect repositories
        if let Ok(repos_response) = cli.execute(&[
            "ecr",
            "describe-repositories",
            "--region",
            region
        ]).await {
            let repos = crate::parallel::extract_array(&repos_response, "repositories")
                .unwrap_or_default();
            
            // Process repos in parallel with concurrency limit of 10
            let detailed_repos = crate::parallel::fetch_details_parallel(
                repos,
                10,
                |repo| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let repo_name = match crate::parallel::extract_string(&repo, "repositoryName") {
                            Some(name) => name,
                            None => return repo,
                        };
                        
                        let repo_arn = crate::parallel::extract_string(&repo, "repositoryArn");
                        
                        // Define all detail fetching operations
                        let mut detail_configs = vec![
                            crate::parallel::build_detail_config("Images", "ecr", "list-images", "--repository-name", &repo_name),
                            crate::parallel::build_detail_config("LifecyclePolicy", "ecr", "get-lifecycle-policy", "--repository-name", &repo_name),
                            crate::parallel::build_detail_config("RepositoryPolicy", "ecr", "get-repository-policy", "--repository-name", &repo_name),
                        ];
                        
                        // Add tags config if ARN is available
                        if let Some(arn) = repo_arn {
                            detail_configs.push(
                                crate::parallel::DetailConfig::new(
                                    "Tags",
                                    vec![
                                        "ecr".to_string(),
                                        "list-tags-for-resource".to_string(),
                                        "--resource-arn".to_string(),
                                        arn,
                                    ],
                                )
                            );
                        }
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, repo, detail_configs).await
                    }
                },
            ).await;
            
            let detailed_response = json!({
                "repositories": detailed_repos
            });
            
            collections.push(ResourceCollection {
                service: "ecr".to_string(),
                region: region.to_string(),
                resource_type: "repositories".to_string(),
                resources: detailed_response,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
