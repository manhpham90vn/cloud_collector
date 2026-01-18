use anyhow::{anyhow, Context, Result};

use super::cli::AwsCli;

/// Validate AWS regions by querying AWS CLI
pub async fn validate_regions(cli: &AwsCli, regions: &[String]) -> Result<()> {
    // Get list of valid regions from AWS
    let valid_regions = get_valid_regions(cli).await?;

    // Check each region
    for region in regions {
        if !valid_regions.contains(region) {
            // Find similar regions for helpful error message
            let similar: Vec<String> = valid_regions
                .iter()
                .filter(|r| {
                    r.starts_with(&region[..region.len().min(3)])
                        || region.starts_with(&r[..r.len().min(3)])
                })
                .take(5)
                .cloned()
                .collect();

            let suggestion = if !similar.is_empty() {
                format!(
                    "\n\n💡 Did you mean one of these?\n   {}",
                    similar.join(", ")
                )
            } else {
                let sample: Vec<String> = valid_regions.iter().take(10).cloned().collect();
                format!("\n\n💡 Available regions:\n   {}", sample.join(", "))
            };

            return Err(anyhow!("❌ Invalid AWS region: '{}'{}", region, suggestion));
        }
    }

    Ok(())
}

/// Get list of valid AWS regions from AWS CLI
async fn get_valid_regions(cli: &AwsCli) -> Result<Vec<String>> {
    let json = cli
        .execute(&["ec2", "describe-regions", "--query", "Regions[].RegionName"])
        .await
        .context("Failed to query AWS regions")?;

    let regions: Vec<String> =
        serde_json::from_value(json).context("Failed to parse regions from AWS response")?;

    Ok(regions)
}
