use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Spawn a background task to continuously update the collection status display
pub fn spawn_status_display_task(
    service_status: Arc<Mutex<Vec<(String, u8, f64)>>>,
    collection_start: std::time::Instant,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let status = service_status.lock().await;
            let elapsed = collection_start.elapsed().as_secs_f64();

            // Move cursor up to redraw the table
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
            if status
                .iter()
                .all(|(_, state, _)| *state == 2 || *state == 3)
            {
                break;
            }
        }
    })
}
