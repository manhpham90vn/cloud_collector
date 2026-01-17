// EventBridge resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws_cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct EventBridgeCollector;

#[async_trait]
impl ResourceCollector for EventBridgeCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect event buses with detailed information
        if let Ok(buses_response) = cli.execute(&[
            "events",
            "list-event-buses",
            "--region",
            region
        ]).await {
            let mut detailed_buses = Vec::new();
            
            if let Some(buses) = buses_response.get("EventBuses").and_then(|b| b.as_array()) {
                for bus in buses {
                    if let Some(bus_name) = bus.get("Name").and_then(|n| n.as_str()) {
                        let mut bus_details = bus.clone();
                        
                        // Get rules for this event bus
                        if let Ok(rules_response) = cli.execute(&[
                            "events",
                            "list-rules",
                            "--event-bus-name",
                            bus_name,
                            "--region",
                            region
                        ]).await {
                            // Get targets for each rule
                            let mut detailed_rules = Vec::new();
                            
                            if let Some(rules) = rules_response.get("Rules").and_then(|r| r.as_array()) {
                                for rule in rules {
                                    if let Some(rule_name) = rule.get("Name").and_then(|n| n.as_str()) {
                                        let mut rule_details = rule.clone();
                                        
                                        // Get targets for this rule
                                        if let Ok(targets) = cli.execute(&[
                                            "events",
                                            "list-targets-by-rule",
                                            "--rule",
                                            rule_name,
                                            "--event-bus-name",
                                            bus_name,
                                            "--region",
                                            region
                                        ]).await {
                                            rule_details.as_object_mut().unwrap().insert("Targets".to_string(), targets);
                                        }
                                        
                                        detailed_rules.push(rule_details);
                                    }
                                }
                            }
                            
                            bus_details.as_object_mut().unwrap().insert("Rules".to_string(), json!({ "Rules": detailed_rules }));
                        }
                        
                        detailed_buses.push(bus_details);
                    }
                }
            }
            
            let detailed_response = json!({
                "EventBuses": detailed_buses
            });
            
            collections.push(ResourceCollection {
                service: "eventbridge".to_string(),
                region: region.to_string(),
                resource_type: "event-buses".to_string(),
                resources: detailed_response,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect event archives
        if let Ok(archives) = cli.execute(&[
            "events",
            "list-archives",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "eventbridge".to_string(),
                region: region.to_string(),
                resource_type: "archives".to_string(),
                resources: archives,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect API destinations
        if let Ok(api_destinations) = cli.execute(&[
            "events",
            "list-api-destinations",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "eventbridge".to_string(),
                region: region.to_string(),
                resource_type: "api-destinations".to_string(),
                resources: api_destinations,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect connections
        if let Ok(connections) = cli.execute(&[
            "events",
            "list-connections",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "eventbridge".to_string(),
                region: region.to_string(),
                resource_type: "connections".to_string(),
                resources: connections,
                collected_at: timestamp.clone(),
            });
        }
        
        // Collect replays
        if let Ok(replays) = cli.execute(&[
            "events",
            "list-replays",
            "--region",
            region
        ]).await {
            collections.push(ResourceCollection {
                service: "eventbridge".to_string(),
                region: region.to_string(),
                resource_type: "replays".to_string(),
                resources: replays,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
