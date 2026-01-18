// Load Balancer resource collector (ELB, ALB, NLB)
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct ElbCollector;

#[async_trait]
impl ResourceCollector for ElbCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect Classic Load Balancers (ELB v1)
        if let Ok(elbs) = cli.execute(&["elb", "describe-load-balancers", "--region", region]).await {
            collections.push(ResourceCollection {
                service: "elb".to_string(),
                region: region.to_string(),
                resource_type: "classic-load-balancers".to_string(),
                resources: elbs,
                collected_at: timestamp.clone(),
            });
        }

        // Collect Application/Network Load Balancers (ELB v2) with detailed information
        if let Ok(elbv2s_response) = cli.execute(&["elbv2", "describe-load-balancers", "--region", region]).await {
            let lbs = crate::parallel::extract_array(&elbv2s_response, "LoadBalancers")
                .unwrap_or_default();
            
            // Process load balancers in parallel with concurrency limit of 10
            let detailed_lbs = crate::parallel::fetch_details_parallel(
                lbs,
                10,
                |lb| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let lb_arn = match crate::parallel::extract_string(&lb, "LoadBalancerArn") {
                            Some(arn) => arn,
                            None => return lb,
                        };
                        
                        // Define all detail fetching operations
                        let detail_configs = vec![
                            crate::parallel::DetailConfig::new(
                                "Attributes",
                                vec![
                                    "elbv2".to_string(),
                                    "describe-load-balancer-attributes".to_string(),
                                    "--load-balancer-arn".to_string(),
                                    lb_arn.clone(),
                                ],
                            ),
                            crate::parallel::DetailConfig::new(
                                "Tags",
                                vec![
                                    "elbv2".to_string(),
                                    "describe-tags".to_string(),
                                    "--resource-arns".to_string(),
                                    lb_arn,
                                ],
                            ),
                        ];
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, lb, detail_configs).await
                    }
                },
            ).await;
            
            collections.push(ResourceCollection {
                service: "elb".to_string(),
                region: region.to_string(),
                resource_type: "load-balancers".to_string(),
                resources: json!({ "LoadBalancers": detailed_lbs }),
                collected_at: timestamp.clone(),
            });
        }

        // Collect Target Groups with health information
        if let Ok(target_groups_response) = cli.execute(&["elbv2", "describe-target-groups", "--region", region]).await {
            let target_groups = crate::parallel::extract_array(&target_groups_response, "TargetGroups")
                .unwrap_or_default();
            
            // Process target groups in parallel with concurrency limit of 10
            let detailed_target_groups = crate::parallel::fetch_details_parallel(
                target_groups,
                10,
                |target_group| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let tg_arn = match crate::parallel::extract_string(&target_group, "TargetGroupArn") {
                            Some(arn) => arn,
                            None => return target_group,
                        };
                        
                        // Define all detail fetching operations
                        let detail_configs = vec![
                            crate::parallel::DetailConfig::new(
                                "TargetHealth",
                                vec![
                                    "elbv2".to_string(),
                                    "describe-target-health".to_string(),
                                    "--target-group-arn".to_string(),
                                    tg_arn.clone(),
                                ],
                            ),
                            crate::parallel::DetailConfig::new(
                                "Attributes",
                                vec![
                                    "elbv2".to_string(),
                                    "describe-target-group-attributes".to_string(),
                                    "--target-group-arn".to_string(),
                                    tg_arn,
                                ],
                            ),
                        ];
                        
                        // Fetch all details in parallel
                        crate::parallel::fetch_resource_details(&cli, &region, target_group, detail_configs).await
                    }
                },
            ).await;
            
            collections.push(ResourceCollection {
                service: "elb".to_string(),
                region: region.to_string(),
                resource_type: "target-groups".to_string(),
                resources: json!({ "TargetGroups": detailed_target_groups }),
                collected_at: timestamp.clone(),
            });
        }

        // Collect Listeners with rules
        if let Ok(listeners_response) = cli.execute(&["elbv2", "describe-load-balancers", "--region", region]).await {
            // Get listeners for each load balancer
            if let Some(lbs) = listeners_response.get("LoadBalancers").and_then(|l| l.as_array()) {
                let mut all_listeners = Vec::new();
                
                for lb in lbs {
                    if let Some(lb_arn) = lb.get("LoadBalancerArn").and_then(|a| a.as_str()) {
                        if let Ok(listeners) = cli.execute(&["elbv2", "describe-listeners", "--load-balancer-arn", lb_arn, "--region", region]).await {
                            if let Some(listener_list) = listeners.get("Listeners").and_then(|l| l.as_array()) {
                                for listener in listener_list {
                                    if let Some(listener_arn) = listener.get("ListenerArn").and_then(|a| a.as_str()) {
                                        let mut listener_details = listener.clone();
                                        
                                        // Get listener rules
                                        if let Ok(rules) = cli.execute(&[
                                            "elbv2",
                                            "describe-rules",
                                            "--listener-arn",
                                            listener_arn,
                                            "--region",
                                            region
                                        ]).await {
                                            listener_details.as_object_mut().unwrap().insert("Rules".to_string(), rules);
                                        }
                                        
                                        all_listeners.push(listener_details);
                                    } else {
                                        all_listeners.push(listener.clone());
                                    }
                                }
                            }
                        }
                    }
                }
                
                if !all_listeners.is_empty() {
                    collections.push(ResourceCollection {
                        service: "elb".to_string(),
                        region: region.to_string(),
                        resource_type: "listeners".to_string(),
                        resources: json!({ "Listeners": all_listeners }),
                        collected_at: timestamp,
                    });
                }
            }
        }

        Ok(collections)
    }
}
