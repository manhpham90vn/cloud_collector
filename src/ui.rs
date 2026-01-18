use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Create a MultiProgress instance for tracking collection progress
pub fn create_multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Create a progress bar for a service-region combination
pub fn create_service_progress_bar(
    multi: &MultiProgress,
    service: &str,
    region: &str,
) -> ProgressBar {
    let pb = multi.add(ProgressBar::new_spinner());

    // Style for the progress bar
    let style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {prefix:<20} │ {wide_msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]);

    pb.set_style(style);
    pb.set_prefix(format!("{} ({})", service, region));
    pb.set_message("⏳ pending...");
    pb.enable_steady_tick(Duration::from_millis(100));

    pb
}

/// Update progress bar to running state
pub fn set_progress_running(pb: &ProgressBar) {
    pb.set_message("🔄 collecting...");
}

/// Update progress bar to completed state
pub fn set_progress_completed(pb: &ProgressBar, duration: f64) {
    pb.disable_steady_tick();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {prefix:<20} │ {wide_msg}")
            .unwrap(),
    );
    pb.finish_with_message(format!("✅ {:.1}s", duration));
}

/// Update progress bar to error state
pub fn set_progress_error(pb: &ProgressBar) {
    pb.disable_steady_tick();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {prefix:<20} │ {wide_msg}")
            .unwrap(),
    );
    pb.finish_with_message("❌ error");
}

/// Create the summary progress bar
pub fn create_summary_progress_bar(multi: &MultiProgress) -> ProgressBar {
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("📊 {msg}")
            .unwrap(),
    );
    pb.set_message("Collection Summary: 0/0 tasks completed [0.0s elapsed]");
    pb
}

/// Spawn a task to update the summary header
pub fn spawn_summary_task(
    summary_pb: ProgressBar,
    collection_start: std::time::Instant,
    total_tasks: Arc<Mutex<usize>>,
    completed_tasks: Arc<Mutex<usize>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(200)).await;

            let elapsed = collection_start.elapsed().as_secs_f64();
            let total = *total_tasks.lock().await;
            let completed = *completed_tasks.lock().await;

            if completed >= total && total > 0 {
                summary_pb.finish_with_message(format!(
                    "Collection Summary: {}/{} tasks completed in {:.1}s",
                    completed, total, elapsed
                ));
                break;
            }

            summary_pb.set_message(format!(
                "Collection Summary: {}/{} tasks completed [{:.1}s elapsed]",
                completed, total, elapsed
            ));
        }
    })
}
