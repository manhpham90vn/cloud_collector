// Resource collectors for different AWS services
pub mod acm;
pub mod cloudformation;
pub mod cloudfront;
pub mod cloudwatch;
pub mod ec2;
pub mod ecr;
pub mod ecs;
pub mod elasticache;
pub mod elb;
pub mod eventbridge;
pub mod iam;
pub mod lambda;
pub mod rds;
pub mod route53;
pub mod s3;
pub mod secretsmanager;
pub mod ses;
pub mod sns;
pub mod sqs;
pub mod vpc;
pub mod waf;

use crate::aws::cli::AwsCli;
use crate::models::ResourceCollection;
use anyhow::Result;
use async_trait::async_trait;

// Service categories for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceCategory {
    Compute,
    Storage,
    Networking,
    Security,
    Management,
    Integration,
    DevTools,
}

impl ServiceCategory {
    // Get display name for category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Compute => "🔧 Compute & Containers",
            Self::Storage => "💾 Storage & Database",
            Self::Networking => "🌐 Networking",
            Self::Security => "🔐 Security & Identity",
            Self::Management => "📊 Management & Monitoring",
            Self::Integration => "📬 Application Integration",
            Self::DevTools => "🐳 Developer Tools",
        }
    }
}

// Enum for all supported AWS services
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceType {
    Ec2,
    S3,
    Rds,
    Lambda,
    Iam,
    CloudFormation,
    Ecs,
    Vpc,
    Elb,
    Route53,
    CloudFront,
    ElastiCache,
    SecretsManager,
    Ecr,
    Acm,
    Waf,
    CloudWatch,
    EventBridge,
    Sns,
    Sqs,
    Ses,
}

impl ServiceType {
    // Convert string to ServiceType
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ec2" => Some(Self::Ec2),
            "s3" => Some(Self::S3),
            "rds" => Some(Self::Rds),
            "lambda" => Some(Self::Lambda),
            "iam" => Some(Self::Iam),
            "cloudformation" => Some(Self::CloudFormation),
            "ecs" => Some(Self::Ecs),
            "vpc" => Some(Self::Vpc),
            "elb" | "elbv2" => Some(Self::Elb),
            "route53" => Some(Self::Route53),
            "cloudfront" => Some(Self::CloudFront),
            "elasticache" => Some(Self::ElastiCache),
            "secretsmanager" => Some(Self::SecretsManager),
            "ecr" => Some(Self::Ecr),
            "acm" => Some(Self::Acm),
            "waf" => Some(Self::Waf),
            "cloudwatch" => Some(Self::CloudWatch),
            "eventbridge" => Some(Self::EventBridge),
            "sns" => Some(Self::Sns),
            "sqs" => Some(Self::Sqs),
            "ses" => Some(Self::Ses),
            _ => None,
        }
    }

    // Convert ServiceType to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ec2 => "ec2",
            Self::S3 => "s3",
            Self::Rds => "rds",
            Self::Lambda => "lambda",
            Self::Iam => "iam",
            Self::CloudFormation => "cloudformation",
            Self::Ecs => "ecs",
            Self::Vpc => "vpc",
            Self::Elb => "elb",
            Self::Route53 => "route53",
            Self::CloudFront => "cloudfront",
            Self::ElastiCache => "elasticache",
            Self::SecretsManager => "secretsmanager",
            Self::Ecr => "ecr",
            Self::Acm => "acm",
            Self::Waf => "waf",
            Self::CloudWatch => "cloudwatch",
            Self::EventBridge => "eventbridge",
            Self::Sns => "sns",
            Self::Sqs => "sqs",
            Self::Ses => "ses",
        }
    }

    // Get all supported services
    pub fn all() -> Vec<Self> {
        vec![
            Self::Ec2,
            Self::S3,
            Self::Rds,
            Self::Lambda,
            Self::Iam,
            Self::CloudFormation,
            Self::Ecs,
            Self::Vpc,
            Self::Elb,
            Self::Route53,
            Self::CloudFront,
            Self::ElastiCache,
            Self::SecretsManager,
            Self::Ecr,
            Self::Acm,
            Self::Waf,
            Self::CloudWatch,
            Self::EventBridge,
            Self::Sns,
            Self::Sqs,
            Self::Ses,
        ]
    }

    // Check if this is a global service (not region-specific)
    pub fn is_global(&self) -> bool {
        matches!(self, Self::S3 | Self::Iam)
    }

    // Get the category for this service
    pub fn category(&self) -> ServiceCategory {
        match self {
            Self::Ec2 | Self::Ecs | Self::Lambda => ServiceCategory::Compute,
            Self::S3 | Self::Rds | Self::ElastiCache => ServiceCategory::Storage,
            Self::Vpc | Self::Elb | Self::Route53 | Self::CloudFront => ServiceCategory::Networking,
            Self::Iam | Self::Acm | Self::Waf | Self::SecretsManager => ServiceCategory::Security,
            Self::CloudFormation | Self::CloudWatch | Self::EventBridge => {
                ServiceCategory::Management
            }
            Self::Sns | Self::Sqs | Self::Ses => ServiceCategory::Integration,
            Self::Ecr => ServiceCategory::DevTools,
        }
    }
}

// Trait that all collectors must implement
#[async_trait]
pub trait ResourceCollector: Send + Sync {
    async fn collect(&self, cli: &AwsCli, region: &str) -> Result<Vec<ResourceCollection>>;
}

// Factory function to get collector for a service type
pub fn get_collector(service_type: ServiceType) -> Box<dyn ResourceCollector> {
    match service_type {
        ServiceType::Ec2 => Box::new(ec2::Ec2Collector),
        ServiceType::S3 => Box::new(s3::S3Collector),
        ServiceType::Rds => Box::new(rds::RdsCollector),
        ServiceType::Lambda => Box::new(lambda::LambdaCollector),
        ServiceType::Iam => Box::new(iam::IamCollector),
        ServiceType::CloudFormation => Box::new(cloudformation::CloudFormationCollector),
        ServiceType::Vpc => Box::new(vpc::VpcCollector),
        ServiceType::Elb => Box::new(elb::ElbCollector),
        ServiceType::Ecs => Box::new(ecs::EcsCollector),
        ServiceType::Route53 => Box::new(route53::Route53Collector),
        ServiceType::CloudFront => Box::new(cloudfront::CloudFrontCollector),
        ServiceType::ElastiCache => Box::new(elasticache::ElastiCacheCollector),
        ServiceType::SecretsManager => Box::new(secretsmanager::SecretsManagerCollector),
        ServiceType::Ecr => Box::new(ecr::EcrCollector),
        ServiceType::Acm => Box::new(acm::AcmCollector),
        ServiceType::Waf => Box::new(waf::WafCollector),
        ServiceType::CloudWatch => Box::new(cloudwatch::CloudWatchCollector),
        ServiceType::EventBridge => Box::new(eventbridge::EventBridgeCollector),
        ServiceType::Sns => Box::new(sns::SnsCollector),
        ServiceType::Sqs => Box::new(sqs::SqsCollector),
        ServiceType::Ses => Box::new(ses::SesCollector),
    }
}

// Get all supported service names as strings
pub fn get_all_services() -> Vec<String> {
    ServiceType::all()
        .iter()
        .map(|s| s.as_str().to_string())
        .collect()
}
