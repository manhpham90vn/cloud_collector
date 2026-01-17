# AWS Collector - AWS Resource Lister

Tool để list toàn bộ AWS resources và xuất ra file JSON với thông tin chi tiết.

## Tính năng

- ✅ List resources từ **22 AWS services**
- ✅ Hỗ trợ **multi-region** (chỉ định nhiều regions)
- ✅ **Concurrent collection** (chạy đồng thời nhiều collectors) 🆕
- ✅ Thu thập **thông tin chi tiết** cho mỗi resource (metadata, configs, policies, tags)
- ✅ Tự động collect TẤT CẢ resource types của mỗi service
- ✅ Output dạng JSON organized theo service
- ✅ Sử dụng AWS CLI trực tiếp
- ✅ CLI đơn giản với AWS profile selection

## AWS Services được hỗ trợ (22)

### **Compute & Containers**
- **EC2**: 22 resource types (instances, VPCs, subnets, security groups, Auto Scaling Groups, Launch Templates, VPC Peering, Transit Gateway, VPN, etc.)
- **Lambda**: Functions (với configs, event sources, aliases, versions, tags), layers, code signing configs
- **ECS**: Clusters (với capacity providers), services, task definitions, tasks, container instances

### **Storage**
- **S3**: Buckets (với 18 loại metadata: encryption, lifecycle, replication, notifications, inventory, analytics, etc.)
- **EBS**: Volumes, snapshots (part of EC2)

### **Database**
- **RDS**: DB instances, clusters, snapshots, cluster snapshots, subnet groups, parameter groups, option groups, proxies, event subscriptions, reserved instances

### **Networking & Content Delivery**
- **VPC**: VPCs, subnets, route tables, internet gateways, NAT gateways, network ACLs, VPC endpoints, peering connections, VPN gateways, customer gateways
- **ELB**: Classic LBs, ALB/NLB (với attributes, tags), target groups (với health status), listeners (với rules)
- **Route53**: Hosted zones (với record sets, tags), health checks, traffic policies, resolver rules, resolver endpoints
- **CloudFront**: Distributions (với configs, tags), origin access identities, cache policies, origin request policies, response headers policies, functions

### **Security & Identity**
- **IAM**: Users (với attached/inline policies, access keys, MFA, groups), roles (với policies, instance profiles), groups (với policies), SAML/OIDC providers, account password policy
- **ACM**: Certificates (với tags)
- **WAF**: Web ACLs (Regional & CloudFront), IP sets, regex pattern sets, rule groups
- **Secrets Manager**: Secrets (metadata only)

### **Management & Governance**
- **CloudFormation**: Stacks, stack sets, exports, change sets
- **CloudWatch**: Alarms, log groups (với metric filters, subscription filters), dashboards, metric streams, insights rules

### **Application Integration**
- **EventBridge**: Event buses (với rules, targets), archives, API destinations, connections, replays
- **SNS**: Topics (với attributes, subscriptions, tags), platform applications
- **SQS**: Queues (với all attributes, tags)
- **SES**: Identities, configuration sets, receipt rule sets, templates, custom verification templates

### **Containers & Registry**
- **ECR**: Repositories (với images, lifecycle policies, repository policies, tags)
- **ElastiCache**: Cache clusters, replication groups, subnet groups, parameter groups, security groups, snapshots, user groups

## Yêu cầu

- Rust 1.70+ (hoặc mới hơn)
- AWS CLI v2
- AWS credentials đã được cấu hình (`aws configure`)

## Cài đặt

```bash
# Clone repo
git clone <repo-url>
cd aws_collector

# Build
cargo build --release
```

## Sử dụng

### **Cơ bản**

```bash
# Sử dụng profile mặc định (default) và default region từ profile
cargo run --release -- collect

# Sử dụng profile khác
cargo run --release -- collect --profile production
cargo run --release -- collect -p staging
```

### **Multi-Region** 🆕

```bash
# Thu thập từ default region (từ profile) + thêm regions khác
cargo run --release -- collect --regions "eu-west-1,ap-southeast-1"
# Ví dụ: Nếu profile default region là us-east-1, sẽ thu thập: us-east-1, eu-west-1, ap-southeast-1

# Kết hợp với profile
cargo run --release -- collect --profile production --regions "us-west-2,eu-west-1"
# Ví dụ: Nếu production profile default region là us-east-1, sẽ thu thập: us-east-1, us-west-2, eu-west-1
```

### **Timestamp Files** 🆕

```bash
# Tạo file mới với timestamp thay vì overwrite file cũ
cargo run --release -- collect --create-new-file

# Kết hợp với profile và regions
cargo run --release -- collect --profile production --regions "us-east-1" --create-new-file
```

### **Concurrent Collection** 🆕

```bash
# Chạy đồng thời tối đa 5 collectors (default)
cargo run --release -- collect

# Tùy chỉnh số collectors chạy đồng thời (1-10)
cargo run --release -- collect --concurrency 10
cargo run --release -- collect -j 3

# Kết hợp với các options khác
cargo run --release -- collect \
  --profile production \
  --regions "us-east-1,us-west-2" \
  --concurrency 8 \
  --create-new-file
```

> **Lưu ý**: 
> - **Default region** từ profile **luôn được thu thập**
> - `--regions` **thêm** các regions bổ sung, không thay thế default region
> - Ví dụ: Profile default là `us-east-1`, chạy `--regions "eu-west-1"` sẽ thu thập cả `us-east-1` và `eu-west-1`
> - Global services (S3, IAM, CloudFront, Route53) chỉ thu thập **một lần** dù có bao nhiêu regions
> - Nếu **không** chỉ định `--create-new-file`: File cũ sẽ bị overwrite
> - Nếu **có** chỉ định `--create-new-file`: Tạo file mới với timestamp (format: `YYYYMMDD_HHMMSS`)
> - **Concurrency**: Giới hạn từ 1-10, default là 5. Tăng concurrency sẽ nhanh hơn nhưng tốn nhiều tài nguyên

## Performance

### **Comparison Table**

| Concurrency | Thời gian (ước tính) | CPU Usage | Khuyến nghị |
|-------------|---------------------|-----------|-------------|
| 1 (tuần tự) | ~10-15 phút        | Thấp      | Máy yếu, ít services |
| 3           | ~5-7 phút          | Trung bình | Cân bằng |
| 5 (default) | ~3-5 phút          | Trung bình | **Khuyến nghị** |
| 8           | ~2-3 phút          | Cao       | Máy mạnh, nhiều services |
| 10 (max)    | ~2-3 phút          | Rất cao   | Máy rất mạnh |

### **Performance Tips**

1. **Tăng concurrency** nếu:
   - Máy có CPU/RAM đủ mạnh
   - Cần thu thập nhanh
   - Thu thập nhiều services

2. **Giảm concurrency** nếu:
   - Máy yếu hoặc đang chạy nhiều process khác
   - Gặp rate limiting từ AWS
   - Chỉ thu thập vài services

3. **Optimize regions**:
   - Chỉ chỉ định regions thực sự cần thiết
   - Global services (S3, IAM) chỉ chạy 1 lần

4. **Best practice**:
   ```bash
   # Fast collection cho production
   cargo run --release -- collect -p production -r us-east-1 -j 8
   
   # Safe collection cho nhiều regions
   cargo run --release -- collect -p production -r "us-east-1,eu-west-1" -j 5
   ```

### **Sau khi build**

```bash
./target/release/aws_collector collect
./target/release/aws_collector collect --profile production
./target/release/aws_collector collect --regions "us-east-1,eu-west-1"
```

## Output

### **Cấu Trúc Thư Mục** 🆕

Files được tổ chức theo profile để dễ quản lý:

```
output/
├── default/                    # Profile: default
│   ├── ec2_ap-southeast-1_all.json
│   ├── s3_global_all.json
│   ├── lambda_ap-southeast-1_all.json
│   └── ...
├── production/                 # Profile: production
│   ├── ec2_us-east-1_all.json
│   ├── rds_us-east-1_all.json
│   └── ...
└── staging/                    # Profile: staging
    ├── ec2_eu-west-1_all.json
    └── ...
```

### **Với Timestamp** (khi dùng `--create-new-file`)

```
output/
└── default/
    ├── ec2_ap-southeast-1_all_20260118_010000.json
    ├── ec2_ap-southeast-1_all_20260118_020000.json
    ├── s3_global_all_20260118_010000.json
    └── ...
```

Format của mỗi file:

```json
{
  "service": "lambda",
  "region": "ap-southeast-1",
  "resource_type": "functions",
  "resources": {
    "Functions": [
      {
        "FunctionName": "my-function",
        "Configuration": { ... },
        "EventSourceMappings": { ... },
        "Aliases": { ... },
        "Tags": { ... }
      }
    ]
  },
  "collected_at": "2026-01-18T00:30:00Z"
}
```

## Thêm service mới

Để thêm collector cho service mới:

1. Tạo file `src/collectors/<service>.rs` và implement `ResourceCollector` trait
2. Thêm module declaration trong `src/collectors/mod.rs`
3. Thêm variant mới vào `ServiceType` enum
4. Implement các method: `from_str()`, `as_str()`, `all()`, `is_global()`
5. Update `get_collector()` function với service mới

Xem các collector hiện có (ví dụ: `sns.rs`, `sqs.rs`, `ses.rs`) để tham khảo.

## Troubleshooting

### AWS CLI not found
```
Error: AWS CLI not found. Please install AWS CLI first.
```
→ Cài đặt AWS CLI: https://aws.amazon.com/cli/

### Credentials error
```
Error: AWS CLI command failed: Unable to locate credentials
```
→ Chạy `aws configure` để setup credentials

### Permission denied
```
Error: AWS CLI command failed: An error occurred (AccessDenied)
```
→ Kiểm tra IAM permissions của user/role

### Region not found
```
Error: Failed to get default region
```
→ Set region cho profile: `aws configure set region ap-southeast-1 --profile your-profile`

## License

MIT
