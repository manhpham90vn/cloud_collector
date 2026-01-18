use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cloud_collector")]
#[command(about = "Cloud Resource Collector - Collect resources from multiple cloud providers (AWS, GCP, Azure)", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub provider: Provider,
}

#[derive(Parser, Debug)]
pub enum Provider {
    /// AWS (Amazon Web Services)
    Aws {
        #[command(subcommand)]
        command: AwsCommands,
    },
    /// GCP (Google Cloud Platform) - Coming soon
    #[command(hide = true)]
    Gcp,
    /// Azure (Microsoft Azure) - Coming soon
    #[command(hide = true)]
    Azure,
}

#[derive(Parser, Debug)]
pub enum AwsCommands {
    /// Collect AWS resources
    Collect {
        /// AWS profile to use
        #[arg(short, long, default_value = "default")]
        profile: String,

        /// Additional regions to collect from (comma-separated)
        #[arg(short, long)]
        regions: Option<String>,

        /// Services to collect from additional regions (comma-separated)
        /// If not specified, all services will be collected from additional regions
        #[arg(short = 's', long, requires = "regions")]
        region_services: Option<String>,

        /// Create new timestamped files instead of overwriting existing ones
        #[arg(short = 'n', long, default_value = "false")]
        create_new_file: bool,

        /// Maximum number of concurrent collectors (1-10)
        #[arg(short = 'j', long, default_value = "5")]
        concurrency: usize,
    },

    /// List all available AWS services
    #[command(alias = "ls")]
    ListServices,
}
