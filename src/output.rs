// Output formatter and file writer
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::models::{Metadata, ResourceCollection};

pub fn write_output(
    collections: Vec<ResourceCollection>,
    output_dir: &str,
    create_new_file: bool,
    _metadata: Metadata,
    profile: &str,
) -> Result<()> {
    // Create output directory structure: output/{profile}/
    let profile_dir = format!("{}/{}", output_dir, profile);
    fs::create_dir_all(&profile_dir).context(format!(
        "Failed to create profile directory: {}",
        profile_dir
    ))?;

    write_files(&collections, &profile_dir, create_new_file)?;

    Ok(())
}

fn write_files(
    collections: &[ResourceCollection],
    output_dir: &str,
    create_new_file: bool,
) -> Result<()> {
    use std::collections::HashMap;

    // Group collections by service and region
    let mut grouped: HashMap<(String, String), Vec<&ResourceCollection>> = HashMap::new();

    for collection in collections {
        let key = (collection.service.clone(), collection.region.clone());
        grouped.entry(key).or_default().push(collection);
    }

    // Write grouped files - each service gets one combined file per region
    for ((service, region), group) in grouped {
        let filename = if create_new_file {
            format!(
                "{}_{}_all_{}.json",
                service,
                region,
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            )
        } else {
            format!("{}_{}_all.json", service, region)
        };

        let filepath = Path::new(output_dir).join(filename);

        // Create a combined structure with all resource types
        let mut combined_resources = serde_json::Map::new();
        for collection in &group {
            combined_resources.insert(
                collection.resource_type.clone(),
                collection.resources.clone(),
            );
        }

        let combined = serde_json::json!({
            "service": service,
            "region": region,
            "resources": combined_resources,
            "collected_at": group[0].collected_at
        });

        let json = serde_json::to_string_pretty(&combined)
            .context("Failed to serialize combined collection")?;

        fs::write(&filepath, json).context(format!("Failed to write file: {:?}", filepath))?;

        println!(
            "✓ Wrote: {} ({} resource types)",
            filepath.display(),
            group.len()
        );
    }

    Ok(())
}
