# Cloud Collector

[![CI](https://github.com/manhpham90vn/aws_collector/workflows/CI/badge.svg)](https://github.com/manhpham90vn/aws_collector/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)

A high-performance, parallel AWS resource collector built in Rust. Efficiently collect and export AWS resources across multiple regions and services with intelligent concurrency control.

## ✨ Features

- 🚀 **Parallel Collection**: Concurrent resource gathering with configurable concurrency limits
- 🌍 **Multi-Region Support**: Collect resources from multiple AWS regions simultaneously
- 📦 **21 AWS Services**: Comprehensive coverage of major AWS services
- ⚡ **High Performance**: Built with Rust for speed and reliability
- 🎯 **Selective Collection**: Choose specific services for additional regions
- 📊 **Structured Output**: Clean JSON output organized by service and region

## 🔧 Supported AWS Services

### 🔧 Compute & Containers
- **EC2** - Instances, Security Groups, Key Pairs, Volumes, Snapshots, AMIs, Network Interfaces
- **ECS** - Clusters, Services, Tasks, Task Definitions, Container Instances
- **Lambda** - Functions, Layers, Event Source Mappings

### 💾 Storage & Database
- **S3** - Buckets with versioning, encryption, lifecycle, and replication details
- **RDS** - DB Instances, Clusters, Snapshots, Parameter Groups, Subnet Groups
- **ElastiCache** - Redis and Memcached clusters, parameter groups, subnet groups

### 🌐 Networking
- **VPC** - VPCs, Subnets, Route Tables, Internet Gateways, NAT Gateways, VPC Endpoints
- **ELB** - Application, Network, and Classic Load Balancers with target groups
- **Route53** - Hosted Zones and Record Sets
- **CloudFront** - Distributions

### 🔐 Security & Identity
- **IAM** - Users, Groups, Roles, Policies (Global service)
- **ACM** - SSL/TLS Certificates
- **WAF** - Web ACLs, Rules, IP Sets
- **Secrets Manager** - Secrets with rotation configuration

### 📊 Management & Monitoring
- **CloudFormation** - Stacks and Stack Resources
- **CloudWatch** - Alarms, Log Groups, Metrics
- **EventBridge** - Event Buses and Rules

### 📬 Application Integration
- **SNS** - Topics and Subscriptions
- **SQS** - Queues
- **SES** - Identities and Configuration Sets

### 🐳 Developer Tools
- **ECR** - Repositories and Images

## 📋 Requirements

- **Rust** 1.70 or higher
- **AWS CLI** installed and configured
- **AWS Credentials** with appropriate read permissions

## 🚀 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/manhpham90vn/aws_collector.git
cd aws_collector

# Build release binary
cargo build --release

# Binary will be at ./target/release/cloud_collector
```

### From GitHub Releases

Download pre-built binaries from the [Releases](https://github.com/manhpham90vn/aws_collector/releases) page.

## 📖 Usage

### List Available Services

```bash
cloud_collector aws list-services
```

This displays all 21 supported AWS services grouped by category.

### Basic Collection

```bash
# Collect from default profile and region
cloud_collector aws collect

# Specify AWS profile
cloud_collector aws collect --profile production

# Create timestamped output file
cloud_collector aws collect --create-new-file
```

### Multi-Region Collection

```bash
# Collect from additional regions (all services)
cloud_collector aws collect --regions us-west-2,eu-west-1

# Collect specific services from additional regions
cloud_collector aws collect \
  --regions us-west-2,eu-west-1 \
  --region-services acm,cloudfront,lambda
```

### Concurrency Control

```bash
# Adjust concurrent collectors (default: 10)
cloud_collector aws collect --concurrency 20
```

### Complete Example

```bash
# Collect from production profile
# Additional regions: us-west-2, eu-west-1
# Only collect ACM and CloudFront from additional regions
# Use 15 concurrent collectors
# Create new timestamped file
cloud_collector aws collect \
  --profile production \
  --regions us-west-2,eu-west-1 \
  --region-services acm,cloudfront \
  --concurrency 15 \
  --create-new-file
```

## 📁 Output Structure

Resources are saved to `./output/{profile}/` directory:

```
output/
└── default/
    ├── ec2_us-east-1_all.json
    ├── s3_global_all.json
    ├── rds_us-east-1_all.json
    └── lambda_us-east-1_all.json
```

Each file contains:
```json
{
  "service": "ec2",
  "region": "us-east-1",
  "resources": {
    "Instances": [...],
    "SecurityGroups": [...],
    "Volumes": [...]
  },
  "collected_at": "2026-01-18T04:12:01+00:00"
}
```

## 🏗️ Architecture

### Parallel Execution Framework

The project uses a sophisticated parallel execution framework:

- **Generic Utilities** (`src/utils/parallel.rs`): Reusable async parallel patterns
- **AWS Adapters** (`src/aws/parallel_aws.rs`): AWS-specific parallel implementations
- **Collector Builder** (`src/aws/collector_builder.rs`): Declarative collector construction

### Collector Pattern

Collectors use three collection modes:

1. **SimpleList**: Direct API call → JSON output
2. **BatchCommands**: Multiple parallel API calls
3. **ListWithDetails**: List resources → Fetch details in parallel

Example:
```rust
CollectorBuilder::new("lambda", RegionMode::Regional)
    .add_detailed_resource(
        "Functions",
        vec!["lambda", "list-functions"],
        "Functions",
        "FunctionName",
        10, // concurrency
        vec![
            DetailTemplate::new("Configuration", "lambda", "get-function", "--function-name"),
            DetailTemplate::new("Policy", "lambda", "get-policy", "--function-name"),
        ],
    )
    .collect_with_region(cli, region)
    .await
```

## 🛠️ Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy --all-targets --all-features
```

### Format

```bash
cargo fmt --all
```

## 🗺️ Roadmap

- [x] AWS support with 21 services
- [x] Parallel collection with concurrency control
- [x] Multi-region support
- [x] Selective service collection
- [x] CI/CD with GitHub Actions
- [ ] Google Cloud Platform support
- [ ] Azure support
- [ ] Unified multi-cloud output format
- [ ] Resource change detection
- [ ] Export to multiple formats (CSV, Parquet)

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📧 Contact

For questions or feedback, please open an issue on GitHub.

