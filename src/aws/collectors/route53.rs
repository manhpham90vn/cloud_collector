// Route 53 resource collector
use super::ResourceCollector;
use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

pub struct Route53Collector;

#[async_trait]
impl ResourceCollector for Route53Collector {
    async fn collect(&self, cli: &AwsCli, _region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Route 53 is a global service

        // Collect hosted zones
        if let Ok(zones_response) = cli.execute(&["route53", "list-hosted-zones"]).await {
            let zones =
                crate::parallel::extract_array(&zones_response, "HostedZones").unwrap_or_default();

            // Process zones in parallel with concurrency limit of 10
            let detailed_zones = crate::parallel::fetch_details_parallel(zones, 10, |zone| {
                let cli = cli.clone();
                async move {
                    let zone_id = match crate::parallel::extract_string(&zone, "Id") {
                        Some(id) => id,
                        None => return zone,
                    };

                    // Prepare zone ID for tags (trim /hostedzone/ prefix)
                    let zone_id_trimmed = zone_id.trim_start_matches("/hostedzone/").to_string();

                    // Define all detail fetching operations
                    let detail_configs = vec![
                        crate::parallel::DetailConfig::new(
                            "RecordSets",
                            vec![
                                "route53".to_string(),
                                "list-resource-record-sets".to_string(),
                                "--hosted-zone-id".to_string(),
                                zone_id,
                            ],
                        ),
                        crate::parallel::DetailConfig::new(
                            "Tags",
                            vec![
                                "route53".to_string(),
                                "list-tags-for-resource".to_string(),
                                "--resource-type".to_string(),
                                "hostedzone".to_string(),
                                "--resource-id".to_string(),
                                zone_id_trimmed,
                            ],
                        ),
                    ];

                    // Fetch all details in parallel
                    crate::parallel::fetch_resource_details(&cli, "us-east-1", zone, detail_configs)
                        .await
                }
            })
            .await;

            let detailed_response = json!({
                "HostedZones": detailed_zones
            });

            collections.push(ResourceCollection {
                service: "route53".to_string(),
                region: "global".to_string(),
                resource_type: "hosted-zones".to_string(),
                resources: detailed_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect health checks
        if let Ok(health_checks_response) = cli.execute(&["route53", "list-health-checks"]).await {
            collections.push(ResourceCollection {
                service: "route53".to_string(),
                region: "global".to_string(),
                resource_type: "health-checks".to_string(),
                resources: health_checks_response,
                collected_at: timestamp.clone(),
            });
        }

        // Collect traffic policies
        if let Ok(traffic_policies) = cli.execute(&["route53", "list-traffic-policies"]).await {
            collections.push(ResourceCollection {
                service: "route53".to_string(),
                region: "global".to_string(),
                resource_type: "traffic-policies".to_string(),
                resources: traffic_policies,
                collected_at: timestamp.clone(),
            });
        }

        // Collect resolver rules (regional)
        if let Ok(resolver_rules) = cli
            .execute(&[
                "route53resolver",
                "list-resolver-rules",
                "--region",
                _region,
            ])
            .await
        {
            collections.push(ResourceCollection {
                service: "route53".to_string(),
                region: _region.to_string(),
                resource_type: "resolver-rules".to_string(),
                resources: resolver_rules,
                collected_at: timestamp.clone(),
            });
        }

        // Collect resolver endpoints (regional)
        if let Ok(resolver_endpoints) = cli
            .execute(&[
                "route53resolver",
                "list-resolver-endpoints",
                "--region",
                _region,
            ])
            .await
        {
            collections.push(ResourceCollection {
                service: "route53".to_string(),
                region: _region.to_string(),
                resource_type: "resolver-endpoints".to_string(),
                resources: resolver_endpoints,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
