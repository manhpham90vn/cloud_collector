mod models;
mod aws_cli;
mod collectors;
mod output;
mod parallel;

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::io::{self, Write};

use crate::models::Metadata;

#[derive(Parser, Debug)]
#[command(name = "aws_collector")]
#[command(about = "AWS Resource Collector - Collect AWS resources", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Collect AWS resources
    Collect {
        /// AWS profile to use
        #[arg(short, long, default_value = "default")]
        profile: String,
        
        /// Additional regions to collect from (comma-separated)
        /// If not specified, only the profile's default region will be used
        #[arg(short, long)]
        regions: Option<String>,
        
        /// Create new timestamped files instead of overwriting existing ones
        #[arg(long, default_value = "false")]
        create_new_file: bool,
        
        /// Maximum number of concurrent collectors (1-10)
        #[arg(short = 'j', long, default_value = "5")]
        concurrency: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Collect { profile, regions, create_new_file, concurrency } => {
            // Validate concurrency
            let concurrency = concurrency.clamp(1, 10);
            collect_resources(&profile, regions.as_deref(), create_new_file, concurrency).await?;
        }
    }
    
    Ok(())
}

async fn collect_resources(aws_profile: &str, additional_regions: Option<&str>, create_new_file: bool, concurrency: usize) -> Result<()> {
    println!("🚀 AWS Resource Lister");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Check if AWS CLI is available
    println!("🔍 Checking AWS CLI...");
    aws_cli::AwsCli::check_available()
        .await
        .context("AWS CLI check failed")?;
    println!("✓ AWS CLI is available\n");
    
    // Initialize AWS CLI
    let cli = aws_cli::AwsCli::new(aws_profile.to_string());
    
    // Get regions to collect from
    println!("🌍 Determining regions...");
    
    // Always get default region from profile
    let default_region = cli.get_default_region().await
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
    let enabled_services = collectors::get_all_services();
    
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
    println!("📦 Collecting resources (max {} concurrent)...\n", concurrency);
    
    use tokio::sync::Semaphore;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let all_collections = Arc::new(Mutex::new(Vec::new()));
    let processed_global_services = Arc::new(Mutex::new(HashSet::new()));
    let semaphore = Arc::new(Semaphore::new(concurrency));
    
    // Track service status: (service_name, status, duration)
    // Status: 0 = pending, 1 = running, 2 = completed, 3 = error
    let service_status = Arc::new(Mutex::new(
        enabled_services.iter()
            .map(|s| (s.clone(), 0, 0.0))
            .collect::<Vec<_>>()
    ));
    
    // Print initial table header
    println!("📊 Collection Summary:");
    
    // Track start time for elapsed display
    let collection_start = std::time::Instant::now();
    
    // Spawn a task to update the display
    let display_status = Arc::clone(&service_status);
    let display_task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            let status = display_status.lock().await;
            let elapsed = collection_start.elapsed().as_secs_f64();
            
            // Move cursor up to redraw the table
            // We need to move up by the number of services + 2 (header + services)
            print!("\x1B[{}A", status.len() + 1);
            
            // Print header with elapsed time
            print!("\x1B[2K");
            println!("📊 Collection Summary: [{:.1}s elapsed]", elapsed);
            
            // Sort by duration (descending)
            let mut sorted: Vec<_> = status.iter().collect();
            sorted.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            
            // Print each service line
            for (service, state, duration) in sorted.iter() {
                // Clear the line
                print!("\x1B[2K");
                
                match state {
                    0 => println!("  • {:<20} ⏳ pending", service),
                    1 => println!("  • {:<20} 🔄 running...", service),
                    2 => println!("  • {:<20} {:.1}s", service, duration),
                    3 => println!("  • {:<20} ❌ error", service),
                    _ => println!("  • {:<20} unknown", service),
                }
            }
            
            io::stdout().flush().unwrap();
            
            // Check if all completed
            if status.iter().all(|(_, state, _)| *state == 2 || *state == 3) {
                break;
            }
        }
    });
    
    let mut tasks = Vec::new();
    
    for service_name in enabled_services.clone() {
        let service_type = match collectors::ServiceType::from_str(&service_name) {
            Some(s) => s,
            None => {
                eprintln!("⚠ Warning: Unknown service '{}'", service_name);
                continue;
            }
        };
        
        let cli = cli.clone();
        let regions = regions.clone();
        let all_collections = Arc::clone(&all_collections);
        let processed_global_services = Arc::clone(&processed_global_services);
        let semaphore = Arc::clone(&semaphore);
        let service_status = Arc::clone(&service_status);
        
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
            let collector = collectors::get_collector(service_type);
            
            let mut success = true;
            
            // Check if this is a global service
            if service_type.is_global() {
                // Only process global services once
                let mut processed = processed_global_services.lock().await;
                if processed.contains(&service_name) {
                    return;
                }
                processed.insert(service_name.clone());
                drop(processed);
                
                match collector.collect(&cli, &regions[0]).await {
                    Ok(collections) => {
                        let mut all = all_collections.lock().await;
                        all.extend(collections);
                    }
                    Err(_) => {
                        success = false;
                    }
                }
            } else {
                // Regional services - collect for each region
                for region in &regions {
                    match collector.collect(&cli, region).await {
                        Ok(collections) => {
                            let mut all = all_collections.lock().await;
                            all.extend(collections);
                        }
                        Err(_) => {
                            success = false;
                        }
                    }
                }
            }
            
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
    
    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
    
    // Wait for display task to finish
    let _ = display_task.await;
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All collectors completed!");
    
    // Unwrap Arc to get collections
    let all_collections = Arc::try_unwrap(all_collections)
        .unwrap()
        .into_inner();
    
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
