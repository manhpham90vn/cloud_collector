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

    // Validate all regions before starting collection
    aws::regions::validate_regions(&cli, &regions)
        .await
        .context("Region validation failed")?;

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

    // Create MultiProgress for tracking all tasks
    let multi = Arc::new(ui::create_multi_progress());

    // Track task counts for summary
    let total_tasks = Arc::new(Mutex::new(0usize));
    let completed_tasks = Arc::new(Mutex::new(0usize));

    // Track start time for elapsed display
    let collection_start = std::time::Instant::now();

    // Create summary progress bar FIRST so it appears at the top
    let summary_pb = ui::create_summary_progress_bar(&multi);

    // Spawn summary task
    let summary_task = ui::spawn_summary_task(
        summary_pb,
        collection_start,
        Arc::clone(&total_tasks),
        Arc::clone(&completed_tasks),
    );

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

            // Increment total tasks
            {
                let mut total = total_tasks.lock().await;
                *total += 1;
            }

            // Create progress bar for this task
            let pb = ui::create_service_progress_bar(&multi, &service_name, &region);

            let cli = cli.clone();
            let region = region.clone();
            let all_collections = Arc::clone(&all_collections);
            let semaphore = Arc::clone(&semaphore);
            let service_name = service_name.clone();
            let completed_tasks = Arc::clone(&completed_tasks);

            let task = tokio::spawn(async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.unwrap();

                // Update to running state
                ui::set_progress_running(&pb);

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

                // Update progress bar based on result
                if success {
                    ui::set_progress_completed(&pb, elapsed);
                } else {
                    ui::set_progress_error(&pb);
                }

                // Increment completed tasks
                {
                    let mut completed = completed_tasks.lock().await;
                    *completed += 1;
                }
            });

            tasks.push(task);
        }
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }

    // Wait for summary task to finish
    let _ = summary_task.await;

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
