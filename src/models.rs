// Data models for configuration and resources
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ResourceCollection {
    pub service: String,
    pub region: String,
    pub resource_type: String,
    pub resources: serde_json::Value,
    pub collected_at: String,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub generated_at: String,
    pub aws_profile: String,
    pub regions: Vec<String>,
    pub services: Vec<String>,
}
