// ECS resource collector
use anyhow::Result;
use async_trait::async_trait;
use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct EcsCollector;

#[async_trait]
impl ResourceCollector for EcsCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect ECS Clusters with capacity providers
        if let Ok(cluster_arns) = cli.execute(&["ecs", "list-clusters", "--region", region]).await {
            if let Some(arns) = cluster_arns.get("clusterArns").and_then(|a| a.as_array()) {
                if !arns.is_empty() {
                    let arn_strings: Vec<String> = arns
                        .iter()
                        .filter_map(|a| a.as_str().map(String::from))
                        .collect();
                    
                    if !arn_strings.is_empty() {
                        let arn_args: Vec<&str> = arn_strings.iter().map(|s| s.as_str()).collect();
                        let mut args = vec!["ecs", "describe-clusters", "--clusters"];
                        args.extend(arn_args);
                        args.push("--include");
                        args.push("ATTACHMENTS");
                        args.push("SETTINGS");
                        args.push("STATISTICS");
                        args.push("TAGS");
                        args.push("--region");
                        args.push(region);
                        
                        if let Ok(clusters) = cli.execute(&args).await {
                            collections.push(ResourceCollection {
                                service: "ecs".to_string(),
                                region: region.to_string(),
                                resource_type: "clusters".to_string(),
                                resources: clusters,
                                collected_at: timestamp.clone(),
                            });
                        }
                    }
                }
            }
        }
        
        // Collect Capacity Providers
        if let Ok(capacity_providers) = cli.execute(&[
            "ecs",
            "describe-capacity-providers",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "ecs".to_string(),
                region: region.to_string(),
                resource_type: "capacity-providers".to_string(),
                resources: capacity_providers,
                collected_at: timestamp.clone(),
            });
        }

        // Collect ECS Services
        if let Ok(cluster_arns) = cli.execute(&["ecs", "list-clusters", "--region", region]).await {
            if let Some(arns) = cluster_arns.get("clusterArns").and_then(|a| a.as_array()) {
                let mut all_services = Vec::new();
                
                for arn in arns {
                    if let Some(cluster_arn) = arn.as_str() {
                        if let Ok(service_arns) = cli.execute(&["ecs", "list-services", "--cluster", cluster_arn, "--region", region]).await {
                            if let Some(svc_arns) = service_arns.get("serviceArns").and_then(|a| a.as_array()) {
                                if !svc_arns.is_empty() {
                                    let svc_arn_strings: Vec<String> = svc_arns
                                        .iter()
                                        .filter_map(|a| a.as_str().map(String::from))
                                        .collect();
                                    
                                    if !svc_arn_strings.is_empty() {
                                        let svc_arn_args: Vec<&str> = svc_arn_strings.iter().map(|s| s.as_str()).collect();
                                        let mut args = vec!["ecs", "describe-services", "--cluster", cluster_arn, "--services"];
                                        args.extend(svc_arn_args);
                                        args.push("--region");
                                        args.push(region);
                                        
                                        if let Ok(services) = cli.execute(&args).await {
                                            if let Some(svc_list) = services.get("services").and_then(|s| s.as_array()) {
                                                all_services.extend(svc_list.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                if !all_services.is_empty() {
                    collections.push(ResourceCollection {
                        service: "ecs".to_string(),
                        region: region.to_string(),
                        resource_type: "services".to_string(),
                        resources: serde_json::json!({ "services": all_services }),
                        collected_at: timestamp.clone(),
                    });
                }
            }
        }

        // Collect ECS Task Definitions
        if let Ok(task_defs) = cli.execute(&["ecs", "list-task-definitions", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "ecs".to_string(),
                region: region.to_string(),
                resource_type: "task-definitions".to_string(),
                resources: task_defs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect ECS Tasks (including Fargate)
        if let Ok(cluster_arns) = cli.execute(&["ecs", "list-clusters", "--region", region]).await {
            if let Some(arns) = cluster_arns.get("clusterArns").and_then(|a| a.as_array()) {
                let mut all_tasks = Vec::new();
                
                for arn in arns {
                    if let Some(cluster_arn) = arn.as_str() {
                        if let Ok(task_arns) = cli.execute(&["ecs", "list-tasks", "--cluster", cluster_arn, "--region", region]).await {
                            if let Some(t_arns) = task_arns.get("taskArns").and_then(|a| a.as_array()) {
                                if !t_arns.is_empty() {
                                    let task_arn_strings: Vec<String> = t_arns
                                        .iter()
                                        .filter_map(|a| a.as_str().map(String::from))
                                        .collect();
                                    
                                    if !task_arn_strings.is_empty() {
                                        let task_arn_args: Vec<&str> = task_arn_strings.iter().map(|s| s.as_str()).collect();
                                        let mut args = vec!["ecs", "describe-tasks", "--cluster", cluster_arn, "--tasks"];
                                        args.extend(task_arn_args);
                                        args.push("--region");
                                        args.push(region);
                                        
                                        if let Ok(tasks) = cli.execute(&args).await {
                                            if let Some(task_list) = tasks.get("tasks").and_then(|t| t.as_array()) {
                                                all_tasks.extend(task_list.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                if !all_tasks.is_empty() {
                    collections.push(ResourceCollection {
                        service: "ecs".to_string(),
                        region: region.to_string(),
                        resource_type: "tasks".to_string(),
                        resources: serde_json::json!({ "tasks": all_tasks }),
                        collected_at: timestamp.clone(),
                    });
                }
            }
        }

        // Collect ECS Container Instances
        if let Ok(cluster_arns) = cli.execute(&["ecs", "list-clusters", "--region", region]).await {
            if let Some(arns) = cluster_arns.get("clusterArns").and_then(|a| a.as_array()) {
                let mut all_container_instances = Vec::new();
                
                for arn in arns {
                    if let Some(cluster_arn) = arn.as_str() {
                        if let Ok(container_instances) = cli.execute(&["ecs", "list-container-instances", "--cluster", cluster_arn, "--region", region]).await {
                            if let Some(ci_arns) = container_instances.get("containerInstanceArns").and_then(|a| a.as_array()) {
                                if !ci_arns.is_empty() {
                                    let ci_arn_strings: Vec<String> = ci_arns
                                        .iter()
                                        .filter_map(|a| a.as_str().map(String::from))
                                        .collect();
                                    
                                    if !ci_arn_strings.is_empty() {
                                        let ci_arn_args: Vec<&str> = ci_arn_strings.iter().map(|s| s.as_str()).collect();
                                        let mut args = vec!["ecs", "describe-container-instances", "--cluster", cluster_arn, "--container-instances"];
                                        args.extend(ci_arn_args);
                                        args.push("--region");
                                        args.push(region);
                                        
                                        if let Ok(instances) = cli.execute(&args).await {
                                            if let Some(inst_list) = instances.get("containerInstances").and_then(|i| i.as_array()) {
                                                all_container_instances.extend(inst_list.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                if !all_container_instances.is_empty() {
                    collections.push(ResourceCollection {
                        service: "ecs".to_string(),
                        region: region.to_string(),
                        resource_type: "container-instances".to_string(),
                        resources: serde_json::json!({ "containerInstances": all_container_instances }),
                        collected_at: timestamp,
                    });
                }
            }
        }

        Ok(collections)
    }
}
