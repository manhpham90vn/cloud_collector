// AWS CLI command executor
use anyhow::{Context, Result};
use serde_json::Value;
use tokio::process::Command;

#[derive(Clone)]
pub struct AwsCli {
    profile: String,
}

impl AwsCli {
    pub fn new(profile: String) -> Self {
        Self { profile }
    }

    // Execute AWS CLI command and return JSON output
    pub async fn execute(&self, args: &[&str]) -> Result<Value> {
        let mut cmd = Command::new("aws");

        // Disable pager to prevent hanging on large outputs
        cmd.env("AWS_PAGER", "");

        // Add profile
        cmd.arg("--profile").arg(&self.profile);

        // Add output format
        cmd.arg("--output").arg("json");

        // Add no-cli-pager flag
        cmd.arg("--no-cli-pager");

        // Add user arguments
        cmd.args(args);

        let output = cmd
            .output()
            .await
            .context("Failed to execute AWS CLI command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("AWS CLI command failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value =
            serde_json::from_str(&stdout).context("Failed to parse AWS CLI JSON output")?;

        Ok(json)
    }

    // Check if AWS CLI is available
    pub async fn check_available() -> Result<()> {
        let output = Command::new("aws")
            .arg("--version")
            .output()
            .await
            .context("AWS CLI not found. Please install AWS CLI first.")?;

        if !output.status.success() {
            anyhow::bail!("AWS CLI is not working properly");
        }

        Ok(())
    }

    // Get default region for the profile
    pub async fn get_default_region(&self) -> Result<String> {
        let output = Command::new("aws")
            .arg("configure")
            .arg("get")
            .arg("region")
            .arg("--profile")
            .arg(&self.profile)
            .output()
            .await
            .context("Failed to get default region from AWS profile")?;

        if !output.status.success() {
            // If no region configured, default to ap-southeast-1
            return Ok("ap-southeast-1".to_string());
        }

        let region = String::from_utf8(output.stdout)
            .context("Failed to parse region output")?
            .trim()
            .to_string();

        if region.is_empty() {
            // If empty, default to ap-southeast-1
            Ok("ap-southeast-1".to_string())
        } else {
            Ok(region)
        }
    }

    // Validate AWS credentials by making a simple API call
    pub async fn validate_credentials(&self) -> Result<()> {
        let output = Command::new("aws")
            .env("AWS_PAGER", "")
            .arg("--profile")
            .arg(&self.profile)
            .arg("--output")
            .arg("json")
            .arg("sts")
            .arg("get-caller-identity")
            .output()
            .await
            .context("Failed to validate AWS credentials")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check for common credential errors
            if stderr.contains("InvalidToken") || stderr.contains("malformed") {
                anyhow::bail!(
                    "AWS credentials are invalid or expired for profile '{}'. Please refresh your credentials.\nError: {}",
                    self.profile,
                    stderr.trim()
                );
            } else if stderr.contains("could not be found") || stderr.contains("NoCredentialsError")
            {
                anyhow::bail!(
                    "No credentials found for profile '{}'. Please configure your AWS credentials.\nError: {}",
                    self.profile,
                    stderr.trim()
                );
            } else if stderr.contains("ExpiredToken") {
                anyhow::bail!(
                    "AWS credentials have expired for profile '{}'. Please refresh your credentials.\nError: {}",
                    self.profile,
                    stderr.trim()
                );
            } else {
                anyhow::bail!(
                    "Failed to validate AWS credentials for profile '{}'.\nError: {}",
                    self.profile,
                    stderr.trim()
                );
            }
        }

        Ok(())
    }
}
