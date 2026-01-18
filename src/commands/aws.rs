use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

use crate::aws;
use crate::models::Metadata;
use crate::output;
use crate::ui;

/// List all available AWS services
pub fn list_services() {
    println!("📋 Available AWS Services:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let all_services = aws::collectors::ServiceType::all();

    // Group services by category
    let mut services_by_category: HashMap<aws::collectors::ServiceCategory, Vec<&str>> =
        HashMap::new();

    for service_type in &all_services {
        let category = service_type.category();
        services_by_category
            .entry(category)
            .or_default()
            .push(service_type.as_str());
    }

    // Define the order of categories to display
    let category_order = [
        aws::collectors::ServiceCategory::Compute,
        aws::collectors::ServiceCategory::Storage,
        aws::collectors::ServiceCategory::Networking,
        aws::collectors::ServiceCategory::Security,
        aws::collectors::ServiceCategory::Management,
        aws::collectors::ServiceCategory::Integration,
        aws::collectors::ServiceCategory::DevTools,
    ];

    // Display services grouped by category
    for category in category_order.iter() {
        if let Some(services) = services_by_category.get(category) {
            println!("\n{}:", category.display_name());
            for service in services {
                println!("  • {}", service);
            }
        }
    }

    println!("\n💡 Total: {} services available", all_services.len());
    println!("\nUsage examples:");
    println!("  # Collect specific services from additional regions");
    println!("  cloud_collector aws collect --regions us-east-1 --region-services acm,cloudfront");
}

pub async fn collect_resources(
    aws_profile: &str,
    additional_regions: Option<&str>,
    region_services: Option<&str>,
    create_new_file: bool,
    concurrency: usize,
) -> Result<()> {
    println!("🚀 AWS Resource Lister");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check if AWS CLI is available
    println!("🔍 Checking AWS CLI...");
    aws::cli::AwsCli::check_available()
        .await
        .context("AWS CLI check failed")?;
    println!("✓ AWS CLI is available\n");

    // Initialize AWS CLI
    let cli = aws::cli::AwsCli::new(aws_profile.to_string());

    // Validate credentials before proceeding
    println!("🔐 Validating AWS credentials...");
    cli.validate_credentials()
        .await
        .context("Credential validation failed")?;
    println!("✓ AWS credentials are valid\n");

    // Get regions to collect from
    println!("🌍 Determining regions...");

    // Always get default region from profile
    let default_region = cli
        .get_default_region()
        .await
        .context("Failed to get default region")?;

    let mut regions = vec![default_region.clone()];

    // Add additional regions if specified
    if let Some(region_str) = additional_regions {
        let additional: Vec<String> = region_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|r| r != &default_region) // Avoid duplicates
            .collect();
        regions.extend(additional);
    }

    // Configuration
    let output_directory = "./output".to_string();

    // All supported services are enabled by default
    let enabled_services = aws::collectors::get_all_services();

    println!("✓ Configuration loaded");
    println!("  Profile: {}", aws_profile);
    println!("  Regions: {}", regions.join(", "));
    println!("  Concurrency: {} collectors", concurrency);
    println!("  Services ({}): ", enabled_services.len());
    for (i, service) in enabled_services.iter().enumerate() {
        if i % 6 == 0 {
            print!("\n    ");
        }
        print!("{:<16}", service);
    }
    println!("\n");

    // Collect resources with concurrency control
    println!(
        "📦 Collecting resources (max {} concurrent)...\n",
        concurrency
    );

    let all_collections = Arc::new(Mutex::new(Vec::new()));
    let processed_global_services = Arc::new(Mutex::new(HashSet::new()));
    let semaphore = Arc::new(Semaphore::new(concurrency));

    // Track service status: (service_name, status, duration)
    // Status: 0 = pending, 1 = running, 2 = completed, 3 = error
    let service_status = Arc::new(Mutex::new(
        enabled_services
            .iter()
            .map(|s| (s.clone(), 0, 0.0))
            .collect::<Vec<_>>(),
    ));

    // Print initial table header
    println!("📊 Collection Summary:");

    // Track start time for elapsed display
    let collection_start = std::time::Instant::now();

    // Spawn a task to update the display
    let display_task = ui::spawn_status_display_task(Arc::clone(&service_status), collection_start);

    let mut tasks = Vec::new();

    // Parse region-specific services if provided
    let region_services_set: Option<HashSet<String>> =
        region_services.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());

    // Display info about region-specific collection
    if let Some(ref services) = region_services_set {
        println!(
            "ℹ️  Additional regions will only collect: {}",
            services.iter().cloned().collect::<Vec<_>>().join(", ")
        );
    }

    for service_name in enabled_services.clone() {
        let service_type = match aws::collectors::ServiceType::from_str(&service_name) {
            Some(s) => s,
            None => {
                eprintln!("⚠ Warning: Unknown service '{}'", service_name);
                continue;
            }
        };

        for region in regions.clone() {
            // Skip global services in non-default regions
            if service_type.is_global() && region != default_region {
                continue;
            }

            // For additional regions (not default), check if service is in region_services list
            if region != default_region {
                if let Some(ref allowed_services) = region_services_set {
                    if !allowed_services.contains(&service_name) {
                        continue; // Skip this service for this additional region
                    }
                }
            }

            // Check if this is a global service and we've already processed it
            if service_type.is_global() {
                let mut processed = processed_global_services.lock().await;
                if processed.contains(&service_name) {
                    continue;
                }
                processed.insert(service_name.clone());
            }

            let cli = cli.clone();
            let region = region.clone();
            let all_collections = Arc::clone(&all_collections);
            let semaphore = Arc::clone(&semaphore);
            let service_status = Arc::clone(&service_status);
            let service_name = service_name.clone();

            let task = tokio::spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.unwrap();

                // Update status to running
                {
                    let mut status = service_status.lock().await;
                    if let Some(entry) = status.iter_mut().find(|(s, _, _)| s == &service_name) {
                        entry.1 = 1; // running
                    }
                }

                let start_time = std::time::Instant::now();
                let collector = aws::collectors::get_collector(service_type);

                let success = match collector.collect(&cli, &region).await {
                    Ok(collections) => {
                        let mut all = all_collections.lock().await;
                        all.extend(collections);
                        true
                    }
                    Err(e) => {
                        eprintln!(
                            "\n⚠️  Error collecting {} in {}: {}",
                            service_name, region, e
                        );
                        false
                    }
                };

                let elapsed = start_time.elapsed().as_secs_f64();

                // Update status to completed or error
                {
                    let mut status = service_status.lock().await;
                    if let Some(entry) = status.iter_mut().find(|(s, _, _)| s == &service_name) {
                        entry.1 = if success { 2 } else { 3 }; // completed or error
                        entry.2 = elapsed;
                    }
                }
            });

            tasks.push(task);
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    // Wait for display task to finish
    let _ = display_task.await;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All collectors completed!");

    // Unwrap Arc to get collections
    let all_collections = Arc::try_unwrap(all_collections).unwrap().into_inner();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Total collections: {}", all_collections.len());

    // Write output
    println!("\n💾 Writing output...");
    let metadata = Metadata {
        generated_at: chrono::Utc::now().to_rfc3339(),
        aws_profile: aws_profile.to_string(),
        regions: regions.clone(),
        services: enabled_services,
    };

    output::write_output(
        all_collections,
        &output_directory,
        create_new_file,
        metadata,
        aws_profile,
    )?;

    println!("\n✅ Done!");

    Ok(())
}
