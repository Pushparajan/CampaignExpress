# CampaignExpress — Operating Guide for Rust Engineers (College Freshers)

## Table of Contents

1. [Welcome](#welcome)
2. [What You'll Be Working On](#what-youll-be-working-on)
3. [Prerequisites & Setup](#prerequisites--setup)
4. [Key Technologies & Concepts](#key-technologies--concepts)
5. [Your Development Workflow](#your-development-workflow)
6. [Common Tasks & Examples](#common-tasks--examples)
7. [Best Practices & Tips](#best-practices--tips)
8. [Learning Resources](#learning-resources)
9. [Getting Help](#getting-help)

---

## Welcome

Welcome to the CampaignExpress Rust engineering team! As a college fresher, you're joining a high-performance real-time platform that processes **50 million ad offers per hour** using cutting-edge Rust technology. Don't worry if this sounds overwhelming—this guide will help you get started step by step.

### What Makes CampaignExpress Special?

- **Rust-First Architecture**: We chose Rust for its memory safety, fearless concurrency, and blazing-fast performance
- **Real-Time Systems**: Sub-10ms response times for ad bidding and offer personalization
- **Modern Stack**: Async/await with Tokio, microservices, distributed systems
- **Production-Scale**: Running on 20-node Kubernetes clusters with millions of requests per hour

### Your Role as a Rust Engineer

You'll be writing, testing, and maintaining Rust code across various modules of the platform. You'll learn:
- How to write safe, concurrent Rust code
- Building APIs with Axum and gRPC with Tonic
- Working with distributed systems (NATS, Redis, ClickHouse)
- Real-time data processing and async programming
- Production-grade error handling and observability

---

## What You'll Be Working On

CampaignExpress is organized as a **Rust workspace** with multiple crates. Here's what each part does:

### Core Platform Crates

```
crates/
├── core/                 # Common types, configs, errors, traits
├── npu-engine/           # ML inference engine (CoLaNet neural network)
├── agents/               # 20 Tokio agents that process bids
├── cache/                # Two-tier caching (DashMap + Redis)
├── analytics/            # Async logging to ClickHouse database
└── api-server/           # REST (Axum) and gRPC (Tonic) servers
```

### Feature Crates (Marketing & Campaign Tools)

```
crates/
├── loyalty/              # Customer loyalty program logic
├── dsp/                  # DSP integrations (Google, Amazon, etc.)
├── channels/             # Email, SMS, push notifications
├── management/           # Campaign CRUD operations
├── journey/              # Customer journey orchestration
├── dco/                  # Dynamic Creative Optimization
├── cdp/                  # Customer Data Platform adapters
└── ... (many more!)
```

### Your First Tasks

As a fresher, you'll typically start with:
1. **Bug fixes** in existing crates to learn the codebase
2. **Adding tests** to increase code coverage
3. **Small feature additions** to existing modules
4. **API endpoint implementations** in the api-server crate

---

## Prerequisites & Setup

### 1. Install Rust

```bash
# Install Rust using rustup (the official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install the stable toolchain
rustup install stable

# Verify installation
rustc --version  # Should show 1.77 or higher
cargo --version
```

### 2. Install Development Tools

```bash
# Install Docker (for running dependencies)
# Follow instructions at: https://docs.docker.com/get-docker/

# Install Docker Compose
# Follow instructions at: https://docs.docker.com/compose/install/

# Install useful Rust tools
cargo install cargo-watch  # Auto-rebuild on file changes
cargo install cargo-expand # View macro expansions
cargo install cargo-audit   # Check for security vulnerabilities
```

### 3. Clone the Repository

```bash
git clone https://github.com/Pushparajan/CampaignExpress.git
cd CampaignExpress
```

### 4. Set Up Your Environment

```bash
# Copy the example environment file
cp .env.example .env

# Open .env in your editor and review the settings
# Most defaults are fine for local development
```

### 5. Start Infrastructure Services

```bash
# Start NATS, Redis, ClickHouse, Prometheus, and Grafana
docker compose -f deploy/docker/docker-compose.yml up -d

# Check that services are running
docker ps
```

### 6. Build the Project

```bash
# Build all crates in the workspace
cargo build

# Or build in release mode (optimized, takes longer)
cargo build --release
```

**🎉 Congratulations!** If this completes successfully, you're ready to start coding!

---

## Key Technologies & Concepts

### 1. Rust Language Fundamentals

#### Ownership & Borrowing (Most Important!)

Rust's ownership system prevents memory bugs without garbage collection:

```rust
// Ownership: Each value has exactly one owner
let s1 = String::from("hello");
let s2 = s1;  // s1 is moved to s2, s1 is no longer valid
// println!("{}", s1);  // ❌ ERROR: s1 was moved

// Borrowing: You can borrow references without taking ownership
let s3 = String::from("hello");
let len = calculate_length(&s3);  // &s3 borrows s3
println!("{} has length {}", s3, len);  // ✅ OK: s3 is still valid

fn calculate_length(s: &String) -> usize {
    s.len()  // We can read but not modify
}
```

#### Lifetimes

Lifetimes ensure references are always valid:

```rust
// The compiler ensures 'result' doesn't outlive 's1' or 's2'
fn longest<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() > s2.len() {
        s1
    } else {
        s2
    }
}
```

#### Result & Option for Error Handling

```rust
use anyhow::Result;  // We use the anyhow crate for error handling

// Functions that can fail return Result<T, E>
fn fetch_campaign(id: &str) -> Result<Campaign> {
    let campaign = database.get(id)?;  // ? propagates errors
    Ok(campaign)
}

// Option for values that might not exist
fn find_user(id: &str) -> Option<User> {
    users.get(id).cloned()
}
```

### 2. Async Programming with Tokio

CampaignExpress is built on **Tokio**, an async runtime for Rust:

```rust
use tokio;

#[tokio::main]  // This macro sets up the async runtime
async fn main() {
    // Async functions return Futures
    let result = fetch_data().await;  // .await waits for the future to complete
    println!("Got: {:?}", result);
}

async fn fetch_data() -> String {
    // Simulating an async operation
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    "Data".to_string()
}
```

#### Concurrent Tasks

```rust
use tokio::task;

// Spawn multiple tasks that run concurrently
let task1 = task::spawn(async { process_request_1().await });
let task2 = task::spawn(async { process_request_2().await });

// Wait for both to complete
let (result1, result2) = tokio::join!(task1, task2);
```

### 3. Web APIs with Axum

We use **Axum** for HTTP APIs:

```rust
use axum::{
    Router,
    routing::{get, post},
    Json,
    extract::Path,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Campaign {
    id: String,
    name: String,
    budget: f64,
}

// Define routes
let app = Router::new()
    .route("/campaigns", get(list_campaigns))
    .route("/campaigns/:id", get(get_campaign))
    .route("/campaigns", post(create_campaign));

// Handler functions
async fn list_campaigns() -> Json<Vec<Campaign>> {
    let campaigns = vec![/* ... */];
    Json(campaigns)
}

async fn get_campaign(Path(id): Path<String>) -> Json<Campaign> {
    let campaign = Campaign {
        id,
        name: "Summer Sale".to_string(),
        budget: 10000.0,
    };
    Json(campaign)
}

async fn create_campaign(Json(campaign): Json<Campaign>) -> Json<Campaign> {
    // Save to database...
    Json(campaign)
}
```

### 4. gRPC with Tonic

For high-performance RPC, we use **Tonic**:

```rust
use tonic::{transport::Server, Request, Response, Status};

// Proto definition in proto/service.proto generates this code
pub mod campaign_service {
    tonic::include_proto!("campaign");
}

use campaign_service::{
    campaign_service_server::{CampaignService, CampaignServiceServer},
    GetCampaignRequest, GetCampaignResponse,
};

#[derive(Default)]
pub struct MyCampaignService {}

#[tonic::async_trait]
impl CampaignService for MyCampaignService {
    async fn get_campaign(
        &self,
        request: Request<GetCampaignRequest>,
    ) -> Result<Response<GetCampaignResponse>, Status> {
        let req = request.into_inner();
        // Fetch campaign logic...
        let response = GetCampaignResponse {
            id: req.id,
            name: "Campaign".to_string(),
        };
        Ok(Response::new(response))
    }
}
```

### 5. Message Queues with NATS

**NATS JetStream** is our message queue for distributing work:

```rust
use async_nats::jetstream;

// Connect to NATS
let client = async_nats::connect("nats://localhost:4222").await?;
let jetstream = jetstream::new(client);

// Subscribe to a stream
let stream = jetstream.get_stream("BID_REQUESTS").await?;
let consumer = stream.get_consumer("bid-processor").await?;

// Process messages
while let Some(message) = consumer.next().await? {
    let data = message.payload;
    // Process the bid request...
    process_bid(data).await?;
    message.ack().await?;  // Acknowledge processing
}
```

### 6. Caching with Redis and DashMap

Two-tier caching for performance:

```rust
use dashmap::DashMap;
use redis::AsyncCommands;

// L1 Cache: DashMap (in-memory, lock-free)
let l1_cache: DashMap<String, Campaign> = DashMap::new();
l1_cache.insert("campaign-1".to_string(), campaign);

// L2 Cache: Redis (distributed)
let mut redis_conn = redis_client.get_async_connection().await?;
let campaign_json = serde_json::to_string(&campaign)?;
redis_conn.set_ex("campaign:1", campaign_json, 3600).await?;  // Expire in 1 hour
```

### 7. Database Access with ClickHouse

Analytics are stored in **ClickHouse**:

```rust
use clickhouse::Client;

let client = Client::default()
    .with_url("http://localhost:8123");

// Insert analytics event
client.query("
    INSERT INTO campaign_events (timestamp, campaign_id, event_type, user_id)
    VALUES (?, ?, ?, ?)
")
.bind(chrono::Utc::now())
.bind("campaign-1")
.bind("impression")
.bind("user-123")
.execute()
.await?;
```

### 8. Observability: Metrics & Tracing

We use **Prometheus** metrics and **tracing**:

```rust
use prometheus::{Counter, Histogram};
use tracing::{info, warn, error, instrument};

// Define metrics
lazy_static! {
    static ref REQUESTS_TOTAL: Counter = 
        Counter::new("requests_total", "Total requests").unwrap();
    
    static ref RESPONSE_TIME: Histogram = 
        Histogram::new("response_time_seconds", "Response time").unwrap();
}

// Use tracing for structured logging
#[instrument(skip(db))]  // Automatically traces function entry/exit
async fn process_request(request_id: String, db: &Database) -> Result<Response> {
    info!("Processing request {}", request_id);
    
    REQUESTS_TOTAL.inc();
    let timer = RESPONSE_TIME.start_timer();
    
    let result = db.query(&request_id).await?;
    
    timer.observe_duration();
    info!("Request processed successfully");
    
    Ok(result)
}
```

---

## Your Development Workflow

### Daily Development Loop

1. **Pull Latest Changes**
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feature/my-new-feature
   ```

2. **Make Your Changes**
   - Edit code in your favorite editor (VS Code, RustRover, Vim, etc.)
   - Focus on one small change at a time

3. **Check Your Code Compiles**
   ```bash
   # Type-check without building
   cargo check
   
   # Or use cargo-watch for automatic checking
   cargo watch -x check
   ```

4. **Run Tests**
   ```bash
   # Run all tests
   cargo test
   
   # Run tests for a specific crate
   cargo test -p campaign-agents
   
   # Run a specific test
   cargo test test_bid_processing
   ```

5. **Format Your Code**
   ```bash
   cargo fmt --all
   ```

6. **Lint Your Code**
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

7. **Build & Run**
   ```bash
   # Build
   cargo build
   
   # Run the main application
   cargo run --release -- --node-id dev-01 --api-only
   
   # Or run a specific binary
   cargo run --bin campaign-express --release
   ```

8. **Test Manually**
   ```bash
   # Make API requests to test your changes
   curl http://localhost:8080/health
   
   curl -X POST http://localhost:8080/v1/bid \
     -H "Content-Type: application/json" \
     -d '{"id": "test-req", "imp": [{"id": "1"}]}'
   ```

9. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add campaign filtering by status"
   git push origin feature/my-new-feature
   ```

10. **Create a Pull Request**
    - Go to GitHub and create a PR from your branch
    - Request review from your team lead or mentor

---

## Common Tasks & Examples

### Task 1: Add a New Field to a Struct

**Scenario**: Add an `end_date` field to the `Campaign` struct.

```rust
// In crates/core/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub budget: f64,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,  // ← New field
    pub status: CampaignStatus,
}
```

**Don't forget to**:
- Update any constructors or builder patterns
- Update tests that create Campaign objects
- Update database schemas if applicable

### Task 2: Add a New API Endpoint

**Scenario**: Add an endpoint to get campaigns by status.

```rust
// In crates/api-server/src/routes/campaigns.rs

use axum::{
    routing::get,
    Router,
    Json,
    extract::Query,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct CampaignFilter {
    status: String,
}

pub fn campaign_routes() -> Router {
    Router::new()
        .route("/campaigns", get(list_campaigns))
        .route("/campaigns/by-status", get(campaigns_by_status))  // ← New route
}

async fn campaigns_by_status(
    Query(filter): Query<CampaignFilter>
) -> Json<Vec<Campaign>> {
    // In a real implementation, you'd query the database
    let campaigns = get_campaigns_from_db(&filter.status).await;
    Json(campaigns)
}

async fn get_campaigns_from_db(status: &str) -> Vec<Campaign> {
    // TODO: Implement database query
    vec![]
}
```

**Test it**:
```bash
curl "http://localhost:8080/api/v1/campaigns/by-status?status=active"
```

### Task 3: Add Error Handling

**Scenario**: Handle the case where a campaign is not found.

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

// Define custom error type
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Campaign not found: {0}")]
    NotFound(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

// Implement IntoResponse for your error type
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(id) => (
                StatusCode::NOT_FOUND,
                format!("Campaign {} not found", id)
            ),
            ApiError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e)
            ),
        };
        
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// Use in handler
async fn get_campaign(
    Path(id): Path<String>
) -> Result<Json<Campaign>, ApiError> {
    let campaign = database
        .get_campaign(&id)
        .await?
        .ok_or_else(|| ApiError::NotFound(id))?;
    
    Ok(Json(campaign))
}
```

### Task 4: Write a Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_campaign_creation() {
        let campaign = Campaign {
            id: "test-1".to_string(),
            name: "Test Campaign".to_string(),
            budget: 1000.0,
            start_date: chrono::Utc::now(),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            status: CampaignStatus::Active,
        };
        
        assert_eq!(campaign.id, "test-1");
        assert_eq!(campaign.budget, 1000.0);
    }
    
    #[tokio::test]  // For async tests
    async fn test_fetch_campaign() {
        let result = fetch_campaign("test-1").await;
        assert!(result.is_ok());
    }
}
```

### Task 5: Add Logging and Metrics

```rust
use tracing::{info, warn, debug};
use prometheus::Counter;

lazy_static! {
    static ref CAMPAIGN_CREATED: Counter = 
        Counter::new("campaigns_created_total", "Total campaigns created").unwrap();
}

async fn create_campaign(campaign: Campaign) -> Result<Campaign> {
    debug!("Creating campaign with ID: {}", campaign.id);
    
    // Validate
    if campaign.budget <= 0.0 {
        warn!("Invalid budget for campaign {}: {}", campaign.id, campaign.budget);
        return Err(anyhow::anyhow!("Budget must be positive"));
    }
    
    // Save to database
    save_to_database(&campaign).await?;
    
    // Update metrics
    CAMPAIGN_CREATED.inc();
    
    info!("Campaign {} created successfully", campaign.id);
    Ok(campaign)
}
```

---

## Best Practices & Tips

### 1. Write Idiomatic Rust

```rust
// ✅ Good: Use pattern matching
match result {
    Ok(value) => println!("Success: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}

// ❌ Avoid: Unwrapping without handling errors
let value = result.unwrap();  // Will panic if result is Err
```

```rust
// ✅ Good: Use iterators
let sum: i32 = numbers.iter().sum();
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// ❌ Avoid: Manual loops when iterators work better
let mut sum = 0;
for i in 0..numbers.len() {
    sum += numbers[i];
}
```

### 2. Handle Errors Properly

```rust
// ✅ Good: Propagate errors with ?
fn process_file(path: &str) -> Result<String> {
    let contents = std::fs::read_to_string(path)?;
    let processed = process_contents(&contents)?;
    Ok(processed)
}

// ❌ Avoid: Ignoring errors
fn process_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()  // Panics on error!
}
```

### 3. Use the Type System

```rust
// ✅ Good: Use newtypes for domain concepts
struct CampaignId(String);
struct UserId(String);

fn get_campaign(id: CampaignId) -> Campaign {
    // ...
}

// ❌ Avoid: Using raw strings everywhere
fn get_campaign(id: String) -> Campaign {
    // Could accidentally pass a UserId here!
}
```

### 4. Write Tests

```rust
// ✅ Good: Test edge cases
#[test]
fn test_campaign_budget_validation() {
    assert!(validate_budget(1000.0).is_ok());
    assert!(validate_budget(0.0).is_err());      // Zero budget
    assert!(validate_budget(-100.0).is_err());   // Negative budget
}
```

### 5. Document Your Code

```rust
/// Creates a new campaign with the specified parameters.
///
/// # Arguments
/// * `name` - The display name for the campaign
/// * `budget` - Total budget in dollars (must be positive)
///
/// # Returns
/// * `Ok(Campaign)` - Successfully created campaign
/// * `Err` - If validation fails
///
/// # Examples
/// ```
/// let campaign = create_campaign("Summer Sale", 5000.0)?;
/// ```
pub fn create_campaign(name: &str, budget: f64) -> Result<Campaign> {
    // Implementation...
}
```

### 6. Profile Before Optimizing

```bash
# Use cargo-flamegraph to see where time is spent
cargo install flamegraph
sudo cargo flamegraph --bin campaign-express

# Use cargo-bench for benchmarking
cargo bench
```

### 7. Keep Dependencies Updated

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Check for security vulnerabilities
cargo audit
```

---

## Learning Resources

### Official Rust Resources

1. **[The Rust Book](https://doc.rust-lang.org/book/)** - Start here! The official Rust programming language book
2. **[Rust by Example](https://doc.rust-lang.org/rust-by-example/)** - Learn by looking at code examples
3. **[Rustlings](https://github.com/rust-lang/rustlings)** - Small exercises to practice Rust

### Async Rust & Tokio

1. **[Tokio Tutorial](https://tokio.rs/tokio/tutorial)** - Official Tokio async runtime tutorial
2. **[Async Book](https://rust-lang.github.io/async-book/)** - Understanding async/await in Rust

### Web Development

1. **[Axum Documentation](https://docs.rs/axum/)** - Web framework we use for REST APIs
2. **[Tonic Tutorial](https://github.com/hyperium/tonic)** - gRPC framework for Rust

### Best Practices

1. **[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)** - How to design Rust APIs
2. **[Effective Rust](https://www.lurklurk.org/effective-rust/)** - Best practices and patterns

### Community

1. **[Rust Users Forum](https://users.rust-lang.org/)** - Ask questions and get help
2. **[Rust Subreddit](https://www.reddit.com/r/rust/)** - News and discussions
3. **[Rust Discord](https://discord.gg/rust-lang)** - Real-time chat with other Rustaceans

### Video Courses

1. **[Rust for Beginners (Microsoft)](https://www.youtube.com/playlist?list=PLlrxD0HtieHjbTjrchBwOVks_sr8EVW1x)** - Free video series
2. **[Crust of Rust (Jon Gjengset)](https://www.youtube.com/c/JonGjengset)** - Deep dives into Rust topics

### Books

1. **"Programming Rust" by Jim Blandy & Jason Orendorff** - Comprehensive Rust guide
2. **"Rust in Action" by Tim McNamara** - Practical Rust programming

---

## Getting Help

### When You're Stuck

1. **Read the Compiler Errors** - Rust's compiler errors are very helpful and often tell you exactly what to fix
2. **Check Documentation** - Run `cargo doc --open` to see documentation for all crates
3. **Search the Codebase** - Use `grep` or your editor's search to find similar examples
4. **Ask Your Mentor** - Don't hesitate to ask senior engineers for help
5. **Rubber Duck Debugging** - Explain your problem out loud (to a rubber duck or colleague)

### Common Compiler Errors

**"cannot borrow as mutable"**
```rust
// Problem: Trying to mutate an immutable reference
let s = String::from("hello");
s.push_str(" world");  // ❌ ERROR

// Solution: Make it mutable
let mut s = String::from("hello");
s.push_str(" world");  // ✅ OK
```

**"moved value"**
```rust
// Problem: Value was moved and can't be used again
let s1 = String::from("hello");
let s2 = s1;
println!("{}", s1);  // ❌ ERROR: s1 was moved

// Solution: Clone if you need both, or use references
let s1 = String::from("hello");
let s2 = s1.clone();  // or: let s2 = &s1;
println!("{}", s1);   // ✅ OK
```

**"lifetime may not live long enough"**
```rust
// Problem: Reference outlives its data
fn get_first<'a>(s: &str) -> &'a str {
    let words: Vec<&str> = s.split_whitespace().collect();
    words[0]  // ❌ ERROR: words is dropped at end of function
}

// Solution: Return owned data or fix lifetimes
fn get_first(s: &str) -> String {
    s.split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()  // ✅ OK: returns owned String
}
```

### Useful Commands

```bash
# Explain compiler error in detail
cargo explain E0308

# Run with backtraces for debugging panics
RUST_BACKTRACE=1 cargo run

# Check what a macro expands to
cargo expand

# See what dependencies are pulling in a crate
cargo tree

# Clean build artifacts (use if you have weird errors)
cargo clean
```

---

## Final Thoughts

Remember:
- **It's okay to not know everything** - Rust has a steep learning curve, and even experienced developers are always learning
- **Compiler is your friend** - Rust's compiler catches bugs early; embrace the error messages
- **Ask questions** - The Rust community is welcoming and helpful
- **Practice regularly** - Rust concepts become clearer with practice
- **Read others' code** - Learn by exploring the CampaignExpress codebase
- **Take breaks** - Step away when frustrated; solutions often come after a break

Welcome to the team, and happy coding! 🦀

---

*For questions specific to CampaignExpress, reach out to your team lead or post in the #rust-help Slack channel.*
