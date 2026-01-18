// ECR resource collector - REFACTORED WITH BUILDER
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::aws::collector_builder::{CollectorBuilder, DetailTemplate, RegionMode};
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

pub struct EcrCollector;

#[async_trait]
impl ResourceCollector for EcrCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        CollectorBuilder::new("ecr", RegionMode::Regional)
            .add_detailed_resource(
                "repositories",
                vec!["ecr", "describe-repositories"],
                "repositories",
                "repositoryName",
                10, // concurrency
                vec![
                    DetailTemplate::new("Images", "ecr", "list-images", "--repository-name"),
                    DetailTemplate::new(
                        "LifecyclePolicy",
                        "ecr",
                        "get-lifecycle-policy",
                        "--repository-name",
                    ),
                    DetailTemplate::new(
                        "RepositoryPolicy",
                        "ecr",
                        "get-repository-policy",
                        "--repository-name",
                    ),
                ],
            )
            .collect_with_region(cli, region)
            .await
    }
}
