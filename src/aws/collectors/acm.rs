// ACM (AWS Certificate Manager) resource collector
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use super::ResourceCollector;

pub struct AcmCollector;

#[async_trait]
impl ResourceCollector for AcmCollector {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>> {
        let mut collections = Vec::new();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Collect certificates with tags
        if let Ok(certs_response) = cli.execute(&[
            "acm",
            "list-certificates",
            "--region",
            region
        ]).await {
            let certs = crate::parallel::extract_array(&certs_response, "CertificateSummaryList")
                .unwrap_or_default();
            
            // Process certificates in parallel with concurrency limit of 10
            let detailed_certs_results = crate::parallel::fetch_details_parallel(
                certs,
                10,
                |cert| {
                    let cli = cli.clone();
                    let region = region.to_string();
                    async move {
                        let cert_arn = match crate::parallel::extract_string(&cert, "CertificateArn") {
                            Some(arn) => arn,
                            None => return serde_json::Value::Null,
                        };
                        
                        // Get certificate details first
                        if let Ok(cert_details) = cli.execute(&[
                            "acm",
                            "describe-certificate",
                            "--certificate-arn",
                            &cert_arn,
                            "--region",
                            &region
                        ]).await {
                            if let Some(certificate) = cert_details.get("Certificate").cloned() {
                                // Get tags for the certificate
                                let detail_configs = vec![
                                    crate::parallel::DetailConfig::new(
                                        "Tags",
                                        vec![
                                            "acm".to_string(),
                                            "list-tags-for-certificate".to_string(),
                                            "--certificate-arn".to_string(),
                                            cert_arn,
                                        ],
                                    ),
                                ];
                                
                                return crate::parallel::fetch_resource_details(&cli, &region, certificate, detail_configs).await;
                            }
                        }
                        
                        serde_json::Value::Null
                    }
                },
            ).await;
            
            // Filter out null values
            let detailed_certs: Vec<_> = detailed_certs_results
                .into_iter()
                .filter(|v| !v.is_null())
                .collect();
            
            let detailed_response = json!({
                "Certificates": detailed_certs
            });
            
            collections.push(ResourceCollection {
                service: "acm".to_string(),
                region: region.to_string(),
                resource_type: "certificates".to_string(),
                resources: detailed_response,
                collected_at: timestamp,
            });
        }

        Ok(collections)
    }
}
