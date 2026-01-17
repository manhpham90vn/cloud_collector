// Lambda resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct LambdaCollector;

#[async_trait]
impl ResourceCollector for LambdaCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect Lambda functions with detailed information
        if let Ok(functions_response) = cli.execute(&["lambda", "list-functions", "--region", region]).await {
            let functions = crate::parallel::extract_array(&functions_response, "Functions")
                .unwrap_or_default();
            
            // Process functions in parallel with concurrency limit of 10
            let detailed_functions = crate::parallel::fetch_details_parallel(
                functions,
                10,
                |function| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let function_name = match crate::parallel::extract_string(&function, "FunctionName") {
                            Some(name) => name,
                            None => return function,
                        };
                        
                        let function_arn = crate::parallel::extract_string(&function, "FunctionArn");
                        
                        // Define all detail fetching operations
                        let mut detail_configs = vec![
                            crate::parallel::build_detail_config("Configuration", "lambda", "get-function-configuration", "--function-name", &function_name),
                            crate::parallel::build_detail_config("Aliases", "lambda", "list-aliases", "--function-name", &function_name),
                            crate::parallel::build_detail_config("Versions", "lambda", "list-versions-by-function", "--function-name", &function_name),
                            crate::parallel::build_detail_config("EventSourceMappings", "lambda", "list-event-source-mappings", "--function-name", &function_name),
                            crate::parallel::build_detail_config("FunctionUrlConfig", "lambda", "get-function-url-config", "--function-name", &function_name),
                            crate::parallel::build_detail_config("Concurrency", "lambda", "get-function-concurrency", "--function-name", &function_name),
                        ];
                        
                        // Add tags config if ARN is available
                        if let Some(arn) = function_arn {
                            detail_configs.push(
                                crate::parallel::DetailConfig::new(
                                    "Tags",
                                    vec![
                                        "lambda".to_string(),
                                        "list-tags".to_string(),
                                        "--resource".to_string(),
                                        arn,
                                    ],
                                )
                            );
                        }
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, function, detail_configs).await
                    }
                },
            ).await;
            
            let detailed_response = json!({
                "Functions": detailed_functions
            });
            
            collections.push(ResourceCollection {
                service: "lambda".to_string(),
                region: region.to_string(),
                resource_type: "functions".to_string(),
                resources: detailed_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Lambda layers
        if let Ok(layers) = cli.execute(&["lambda", "list-layers", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "lambda".to_string(),
                region: region.to_string(),
                resource_type: "layers".to_string(),
                resources: layers,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect code signing configurations
        if let Ok(code_signing) = cli.execute(&[
            "lambda",
            "list-code-signing-configs",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "lambda".to_string(),
                region: region.to_string(),
                resource_type: "code-signing-configs".to_string(),
                resources: code_signing,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
