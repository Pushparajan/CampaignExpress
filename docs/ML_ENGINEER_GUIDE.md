# CampaignExpress — Operating Guide for Real-Time ML Engineers (College Freshers)

## Table of Contents

1. [Welcome](#welcome)
2. [What You'll Be Working On](#what-youll-be-working-on)
3. [Prerequisites & Setup](#prerequisites--setup)
4. [Key Technologies & Concepts](#key-technologies--concepts)
5. [ML Inference Pipeline](#ml-inference-pipeline)
6. [Your Development Workflow](#your-development-workflow)
7. [Common Tasks & Examples](#common-tasks--examples)
8. [Best Practices & Tips](#best-practices--tips)
9. [Learning Resources](#learning-resources)
10. [Getting Help](#getting-help)

---

## Welcome

Welcome to the CampaignExpress ML engineering team! As a college fresher, you're joining a platform that serves **50 million personalized offers per hour** using real-time machine learning. This guide will help you understand how ML inference works in production and how you'll contribute to it.

### What Makes CampaignExpress ML Special?

- **Real-Time Inference**: Sub-10ms predictions for ad bidding and personalization
- **Spiking Neural Networks (SNNs)**: Using CoLaNet, an energy-efficient neural architecture
- **Hardware-Agnostic**: Runs on CPU, Groq LPU, AWS Inferentia, AMD NPU, and more
- **Production Scale**: Handling millions of predictions per hour with batching and caching
- **Edge Deployment**: Models deployed in Kubernetes, with edge support via WASM

### Your Role as an ML Engineer

You'll be working on:
- Model inference optimization and performance tuning
- Feature engineering and preprocessing pipelines
- Model evaluation and A/B testing infrastructure
- Hardware acceleration and batching strategies
- Monitoring model performance and detecting drift
- Integrating new ML models into the platform

You **won't** need to train models from scratch initially (we have data scientists for that), but you'll learn how to deploy, serve, and optimize models in production.

---

## What You'll Be Working On

### ML-Related Crates in the Workspace

```
crates/
├── npu-engine/           # Core ML inference engine
│   ├── backends/         # Hardware backends (CPU, GPU, NPU, etc.)
│   │   ├── cpu.rs        # Pure Rust CPU inference
│   │   ├── groq.rs       # Groq LPU acceleration
│   │   ├── inferentia.rs # AWS Inferentia chips
│   │   ├── ampere.rs     # Oracle Ampere Altra
│   │   └── tenstorrent.rs# RISC-V acceleration
│   ├── colanet.rs        # CoLaNet SNN model implementation
│   └── provider.rs       # Hardware abstraction trait
│
├── agents/               # Bid agents that use ML inference
│   └── batcher.rs        # Nagle-style batching for throughput
│
├── personalization/      # Recommendation engine
│   ├── collaborative_filtering.rs
│   ├── content_based.rs
│   ├── trending.rs
│   └── decisioning.rs    # ★ Real-time multi-objective decisioning
│                          #   - CTR, ConversionRate, Revenue, LTV optimization
│                          #   - Per-offer explainability with factor categories
│                          #   - Simulation mode for what-if testing
│
├── cdp/
│   └── feature_store.rs  # ★ Online feature store for ML features
│                          #   - TTL-based staleness detection
│                          #   - Computed features (DaysSince, Ratio, Threshold)
│                          #   - Feature health monitoring
│
├── rl-engine/            # Reinforcement learning
│   ├── offerfit.rs       # OfferFit integration
│   └── thompson_sampling.rs
│
└── dco/                  # Dynamic Creative Optimization
    └── variant_scoring.rs # A/B testing with Thompson Sampling
```

### New ML Modules (v2)

**Real-Time Decision API** (`personalization/decisioning.rs`):
This is a key module for ML engineers. It implements multi-objective optimization that blends scores across CTR, ConversionRate, Revenue, LTV, Engagement, and Retention with configurable weights. Each decision includes per-offer explainability factors showing WHY an offer was selected (SegmentMembership, BehavioralSignal, ContextualRelevance, ModelPrediction, BusinessRule, ExplorationBonus). The simulation mode lets you test decisioning scenarios without logging.

**Online Feature Store** (`cdp/feature_store.rs`):
Manages typed feature definitions with TTL-based staleness tracking. Features can be Behavioral, Demographic, Transactional, Engagement, or Predictive. Includes computed features (DaysSince last purchase, Ratio of conversions to views, etc.) and health monitoring to detect stale features that need refreshing.

### The ML Stack

```
┌─────────────────────────────────────────────────────────┐
│                   Incoming Requests                      │
│              (50M offers/hour, <10ms SLA)               │
└────────────────────┬────────────────────────────────────┘
                     │
            ┌────────▼────────┐
            │  Bid Agents     │  20 Tokio tasks per node
            │  (agents crate) │  Consume from NATS queue
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  Feature        │  Extract user/context features
            │  Extraction     │  from OpenRTB bid request
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  Batcher        │  Nagle-style: 500µs or 16 items
            │  (agents/       │  Groups requests for efficiency
            │   batcher.rs)   │
            └────────┬────────┘
                     │
            ┌────────▼────────┐
            │  Two-Tier Cache │  L1: DashMap (in-memory)
            │  (cache crate)  │  L2: Redis (distributed)
            └────────┬────────┘
                     │ (on cache miss)
            ┌────────▼────────┐
            │  CoLaNet SNN    │  Spiking Neural Network
            │  (npu-engine/   │  Hardware-agnostic inference
            │   colanet.rs)   │
            └────────┬────────┘
                     │
       ┌─────────────┼─────────────┐
       │             │             │
  ┌────▼───┐   ┌───▼────┐   ┌───▼─────┐
  │  CPU   │   │  Groq  │   │ AWS     │  Hardware backends
  │Backend │   │  LPU   │   │Inferentia│  (abstracted via trait)
  └────────┘   └────────┘   └─────────┘
                     │
            ┌────────▼────────┐
            │  Response       │  Offer scores, rankings
            │  (sub-10ms)     │
            └─────────────────┘
```

---

## Prerequisites & Setup

### 1. Install Python (for Model Training Scripts)

Even though the inference engine is in Rust, you'll work with Python for data analysis:

```bash
# Install Python 3.9 or higher
# On Ubuntu/Debian:
sudo apt update
sudo apt install python3 python3-pip python3-venv

# On macOS:
brew install python@3.11

# Verify
python3 --version
pip3 --version
```

### 2. Set Up Python Environment

```bash
cd CampaignExpress

# Create virtual environment
python3 -m venv venv

# Activate it
source venv/bin/activate  # On Linux/macOS
# or
venv\Scripts\activate  # On Windows

# Install ML tools
pip install numpy pandas scikit-learn jupyter matplotlib seaborn
```

### 3. Install Rust (for Inference Code)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup install stable

# Verify
rustc --version
cargo --version
```

### 4. Clone and Build the Project

```bash
git clone https://github.com/Pushparajan/CampaignExpress.git
cd CampaignExpress

# Start infrastructure (NATS, Redis, ClickHouse, etc.)
docker compose -f deploy/docker/docker-compose.yml up -d

# Build the ML inference crate
cargo build -p campaign-npu

# Run tests
cargo test -p campaign-npu
```

### 5. Install Optional ML Tools

```bash
# For model conversion and analysis
pip install onnx onnxruntime torch torchvision

# For performance profiling
cargo install flamegraph
cargo install cargo-criterion
```

**🎉 Setup Complete!** You're ready to work on ML inference!

---

## Key Technologies & Concepts

### 1. What is Real-Time ML Inference?

Unlike training (which happens offline and can take hours), **inference** is making predictions in real-time when a user request arrives.

**Challenges**:
- **Latency**: Must respond in <10ms (milliseconds!)
- **Throughput**: Handle millions of requests per hour
- **Consistency**: Same input should give same output
- **Resource Efficiency**: Minimize CPU/memory/GPU usage

### 2. Spiking Neural Networks (SNNs)

CampaignExpress uses **CoLaNet**, a Spiking Neural Network architecture:

**Traditional Neural Networks**:
- Neurons output continuous values (e.g., 0.7, 0.3, 0.9)
- Always active, consume constant power
- Like a dimmer light switch

**Spiking Neural Networks**:
- Neurons output binary spikes (0 or 1, fire or don't fire)
- Only active when spiking, energy-efficient
- Like a regular on/off light switch
- Inspired by biological neurons in the brain

**Why SNNs?**
- **Energy Efficient**: Great for edge devices and battery-powered systems
- **Fast**: Binary operations are faster than floating-point
- **Temporal Dynamics**: Can process time-series data naturally

### 3. Hardware Abstraction with Traits

The `CoLaNetProvider` trait allows the same model to run on different hardware:

```rust
// In crates/npu-engine/src/provider.rs

use ndarray::Array2;
use anyhow::Result;

/// Hardware-agnostic inference provider trait
#[async_trait::async_trait]
pub trait CoLaNetProvider: Send + Sync {
    /// Initialize the inference provider
    async fn initialize(&mut self) -> Result<()>;
    
    /// Run inference on a batch of inputs
    /// 
    /// # Arguments
    /// * `features` - 2D array (batch_size × feature_dim)
    /// 
    /// # Returns
    /// * 2D array (batch_size × num_classes) with prediction scores
    async fn infer(&self, features: Array2<f32>) -> Result<Array2<f32>>;
    
    /// Get the device name (e.g., "cpu", "groq-lpu", "inferentia-2")
    fn device_name(&self) -> &str;
    
    /// Check if the provider is available on this system
    fn is_available() -> bool where Self: Sized;
}
```

**Implementations**:
- **CPU Backend**: Pure Rust, always available
- **Groq LPU**: Ultra-fast language processing unit
- **AWS Inferentia**: AWS's ML inference chips
- **AMD XDNA NPU**: Neural Processing Unit for edge devices
- **Tenstorrent**: RISC-V based AI accelerator

### 4. Feature Engineering

Converting raw data into ML-friendly features:

```rust
use serde::{Deserialize, Serialize};
use ndarray::Array1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidRequest {
    pub user_id: String,
    pub device_type: String,  // "mobile", "desktop", "tablet"
    pub time_of_day: u8,      // 0-23
    pub geo_country: String,
    pub site_category: String,
    // ... more fields
}

/// Extract features from a bid request for ML inference
pub fn extract_features(request: &BidRequest) -> Array1<f32> {
    let mut features = Vec::new();
    
    // Categorical encoding: device_type (one-hot)
    features.push(if request.device_type == "mobile" { 1.0 } else { 0.0 });
    features.push(if request.device_type == "desktop" { 1.0 } else { 0.0 });
    features.push(if request.device_type == "tablet" { 1.0 } else { 0.0 });
    
    // Numerical: time of day (normalized to 0-1)
    features.push(request.time_of_day as f32 / 24.0);
    
    // Geographic encoding (simple example)
    features.push(if request.geo_country == "US" { 1.0 } else { 0.0 });
    
    // ... more features
    
    Array1::from_vec(features)
}
```

### 5. Batching for Throughput

**Problem**: Running inference on 1 item at a time is slow.
**Solution**: Batch multiple items together.

**Nagle-Style Batching** (used in CampaignExpress):
- Wait up to **500 microseconds** OR collect **16 items**, whichever comes first
- Send the batch for inference
- Distribute results back to individual requests

```rust
// In crates/agents/src/batcher.rs

pub struct InferenceBatcher {
    batch_timeout: Duration,     // 500µs
    batch_size: usize,            // 16 items
    pending_requests: Vec<PendingRequest>,
    npu_provider: Arc<dyn CoLaNetProvider>,
}

impl InferenceBatcher {
    pub async fn add_request(&mut self, request: BidRequest) -> Result<Prediction> {
        self.pending_requests.push(request);
        
        // Flush if batch is full
        if self.pending_requests.len() >= self.batch_size {
            return self.flush().await;
        }
        
        // Otherwise, wait for timeout
        tokio::time::sleep(self.batch_timeout).await;
        self.flush().await
    }
    
    async fn flush(&mut self) -> Result<Vec<Prediction>> {
        let features = self.extract_batch_features();
        let predictions = self.npu_provider.infer(features).await?;
        self.pending_requests.clear();
        Ok(predictions)
    }
}
```

### 6. Two-Tier Caching

Avoid redundant inference calls:

```
Request → L1 Cache (DashMap) → L2 Cache (Redis) → Model Inference
          ↓ hit (instant)        ↓ hit (~1ms)      ↓ miss (~5-10ms)
          Return                  Return             Return + Cache
```

**When to cache**:
- Same user seen recently
- Popular items (trending products, top campaigns)
- Stable predictions (user preferences don't change every millisecond)

**When NOT to cache**:
- Time-sensitive features (e.g., time-of-day changes constantly)
- Unique, one-time requests

### 7. Model Metrics & Monitoring

Track model performance in production:

```rust
use prometheus::{Histogram, Counter};

lazy_static! {
    // Latency: How long does inference take?
    static ref INFERENCE_LATENCY: Histogram = 
        Histogram::new("ml_inference_duration_seconds", 
                       "Time to run ML inference")
        .unwrap();
    
    // Throughput: How many predictions per second?
    static ref PREDICTIONS_TOTAL: Counter = 
        Counter::new("ml_predictions_total", 
                     "Total ML predictions made")
        .unwrap();
    
    // Cache hit rate
    static ref CACHE_HITS: Counter = 
        Counter::new("ml_cache_hits_total", 
                     "ML cache hits")
        .unwrap();
    
    static ref CACHE_MISSES: Counter = 
        Counter::new("ml_cache_misses_total", 
                     "ML cache misses")
        .unwrap();
}

pub async fn run_inference(features: Array2<f32>) -> Result<Array2<f32>> {
    let timer = INFERENCE_LATENCY.start_timer();
    
    let predictions = npu_provider.infer(features).await?;
    
    timer.observe_duration();
    PREDICTIONS_TOTAL.inc_by(features.nrows() as u64);
    
    Ok(predictions)
}
```

**View metrics**:
```bash
curl http://localhost:9091/metrics | grep ml_
```

### 8. A/B Testing and Experimentation

**Thompson Sampling** for multi-armed bandit problems:

```rust
use rand::distributions::{Beta, Distribution};

pub struct ThompsonSampler {
    // For each variant, track successes (α) and failures (β)
    variants: Vec<(f64, f64)>,  // (alpha, beta) parameters
}

impl ThompsonSampler {
    /// Select which variant to show based on Thompson Sampling
    pub fn select_variant(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut best_score = 0.0;
        let mut best_variant = 0;
        
        for (i, (alpha, beta)) in self.variants.iter().enumerate() {
            // Sample from Beta distribution
            let beta_dist = Beta::new(*alpha, *beta).unwrap();
            let score = beta_dist.sample(&mut rng);
            
            if score > best_score {
                best_score = score;
                best_variant = i;
            }
        }
        
        best_variant
    }
    
    /// Update after observing a result
    pub fn update(&mut self, variant: usize, success: bool) {
        if success {
            self.variants[variant].0 += 1.0;  // Increment α (successes)
        } else {
            self.variants[variant].1 += 1.0;  // Increment β (failures)
        }
    }
}
```

---

## ML Inference Pipeline

### Step-by-Step Flow

#### 1. Request Arrives
```
Bid Request (OpenRTB JSON)
↓
{
  "user": {"id": "user-123"},
  "device": {"type": "mobile"},
  "site": {"domain": "example.com"},
  "imp": [{"id": "imp-1", "banner": {"w": 300, "h": 250}}]
}
```

#### 2. Feature Extraction
```rust
let features = extract_features(&bid_request);
// → Array1<f32>: [1.0, 0.0, 0.0, 0.75, 1.0, ...]
//   (device_mobile, device_desktop, device_tablet, time_normalized, geo_us, ...)
```

#### 3. Cache Lookup
```rust
let cache_key = generate_cache_key(&bid_request);

// L1: In-memory cache (DashMap)
if let Some(cached) = l1_cache.get(&cache_key) {
    return cached.clone();
}

// L2: Redis
if let Some(cached) = redis.get(&cache_key).await? {
    l1_cache.insert(cache_key.clone(), cached.clone());
    return cached;
}

// Cache miss → run inference
```

#### 4. Batching
```rust
// Add to batch
batcher.add_request(features).await;

// When batch is ready (500µs or 16 items):
let batch_features = Array2::from_shape_vec(
    (batch_size, feature_dim),
    all_features
)?;
```

#### 5. Inference
```rust
// Call the hardware-specific backend
let predictions = npu_provider.infer(batch_features).await?;

// predictions: Array2<f32> of shape (batch_size, num_campaigns)
// Each row contains scores for different campaigns/offers
```

#### 6. Post-Processing
```rust
// Select top-K campaigns
let top_k = 3;
let mut campaign_scores: Vec<(CampaignId, f32)> = predictions
    .row(i)
    .indexed_iter()
    .map(|(campaign_idx, &score)| (campaign_idx, score))
    .collect();

campaign_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let top_campaigns: Vec<CampaignId> = campaign_scores
    .iter()
    .take(top_k)
    .map(|(id, _)| *id)
    .collect();
```

#### 7. Cache Results
```rust
// Store in both caches
l1_cache.insert(cache_key.clone(), top_campaigns.clone());
redis.set_ex(&cache_key, serde_json::to_string(&top_campaigns)?, 3600).await?;
```

#### 8. Return Response
```
{
  "offers": [
    {"campaign_id": "campaign-42", "score": 0.87},
    {"campaign_id": "campaign-17", "score": 0.76},
    {"campaign_id": "campaign-9", "score": 0.64}
  ]
}
```

---

## Your Development Workflow

### Daily ML Engineering Tasks

1. **Check Model Performance Metrics**
   ```bash
   # Open Grafana dashboard
   open http://localhost:3000
   
   # Or query Prometheus directly
   curl http://localhost:9091/metrics | grep ml_inference
   ```

2. **Analyze Feature Distributions**
   ```python
   # In Jupyter notebook
   import pandas as pd
   import clickhouse_driver
   
   client = clickhouse_driver.Client('localhost')
   
   # Query recent feature values
   query = """
   SELECT * FROM campaign_events
   WHERE timestamp > now() - INTERVAL 1 HOUR
   LIMIT 10000
   """
   
   df = pd.DataFrame(client.execute(query))
   df['time_of_day'].hist(bins=24)
   ```

3. **Test Model Changes Locally**
   ```bash
   # Build the NPU engine crate
   cargo build -p campaign-npu
   
   # Run unit tests
   cargo test -p campaign-npu
   
   # Run benchmarks
   cargo bench -p campaign-npu
   ```

4. **Deploy New Model Weights**
   ```bash
   # Models are stored in the models/ directory
   # To deploy a new model:
   cp path/to/new_model.bin models/colanet_v2.bin
   
   # Update config to point to new model
   vim config/model_config.toml
   
   # Restart the service
   kubectl rollout restart deployment/campaign-express
   ```

### Experimentation Workflow

#### Running an A/B Test

1. **Define the experiment**
   ```rust
   let experiment = Experiment {
       id: "campaign-scoring-v2".to_string(),
       variants: vec![
           Variant { id: "control", weight: 0.5 },
           Variant { id: "new-model", weight: 0.5 },
       ],
       start_date: Utc::now(),
       end_date: Utc::now() + Duration::days(7),
   };
   ```

2. **Implement variant logic**
   ```rust
   let variant = experiment.assign_user(&user_id);
   
   let predictions = match variant.as_str() {
       "control" => run_model_v1(features).await?,
       "new-model" => run_model_v2(features).await?,
       _ => unreachable!(),
   };
   ```

3. **Log experiment data**
   ```rust
   analytics.log_event(ExperimentEvent {
       experiment_id: "campaign-scoring-v2",
       variant: "new-model",
       user_id: user_id.clone(),
       outcome: if clicked { "click" } else { "no-click" },
       timestamp: Utc::now(),
   }).await?;
   ```

4. **Analyze results**
   ```python
   import pandas as pd
   from scipy import stats
   
   # Query experiment data
   control = df[df['variant'] == 'control']
   treatment = df[df['variant'] == 'new-model']
   
   # Calculate click-through rates
   ctr_control = control['clicked'].mean()
   ctr_treatment = treatment['clicked'].mean()
   
   # Statistical significance test
   statistic, p_value = stats.ttest_ind(
       control['clicked'], 
       treatment['clicked']
   )
   
   print(f"Control CTR: {ctr_control:.3%}")
   print(f"Treatment CTR: {ctr_treatment:.3%}")
   print(f"P-value: {p_value:.4f}")
   print(f"Significant: {p_value < 0.05}")
   ```

---

## Common Tasks & Examples

### Task 1: Add a New Feature

**Scenario**: Add "hour of week" as a cyclical feature.

```rust
// In crates/npu-engine/src/features.rs

pub fn extract_features(request: &BidRequest) -> Array1<f32> {
    let mut features = Vec::new();
    
    // ... existing features ...
    
    // New: Hour of week (0-167) encoded as sine/cosine
    let hour_of_week = request.timestamp.weekday().num_days_from_monday() * 24 
                     + request.timestamp.hour();
    
    let angle = 2.0 * std::f32::consts::PI * (hour_of_week as f32) / 168.0;
    features.push(angle.sin());  // Sine component
    features.push(angle.cos());  // Cosine component
    
    Array1::from_vec(features)
}
```

**Why sine/cosine?** Hour 167 (Sunday 11pm) is close to hour 0 (Monday midnight), but numerically they're far apart. Sine/cosine encoding makes them close in feature space.

### Task 2: Implement a New Inference Backend

**Scenario**: Add support for a new hardware accelerator.

```rust
// In crates/npu-engine/src/backends/my_accelerator.rs

use crate::provider::CoLaNetProvider;
use ndarray::Array2;
use anyhow::Result;

pub struct MyAcceleratorProvider {
    device_handle: MyAcceleratorDevice,
    model_weights: Vec<f32>,
}

#[async_trait::async_trait]
impl CoLaNetProvider for MyAcceleratorProvider {
    async fn initialize(&mut self) -> Result<()> {
        // Load model onto the device
        self.device_handle.load_model(&self.model_weights)?;
        Ok(())
    }
    
    async fn infer(&self, features: Array2<f32>) -> Result<Array2<f32>> {
        // Convert to device format
        let device_input = self.device_handle.copy_to_device(features.as_slice()?)?;
        
        // Run inference
        let device_output = self.device_handle.run_inference(device_input)?;
        
        // Convert back to ndarray
        let output = self.device_handle.copy_from_device(device_output)?;
        let shape = (features.nrows(), self.num_outputs());
        Ok(Array2::from_shape_vec(shape, output)?)
    }
    
    fn device_name(&self) -> &str {
        "my-accelerator"
    }
    
    fn is_available() -> bool {
        MyAcceleratorDevice::is_present()
    }
}
```

### Task 3: Optimize Inference Latency

**Scenario**: Reduce inference time from 8ms to 5ms.

**Approaches**:

1. **Profile to find bottlenecks**
   ```bash
   cargo flamegraph --bin campaign-express
   # Opens flamegraph.svg showing where time is spent
   ```

2. **Use smaller batch sizes** (trade throughput for latency)
   ```rust
   let batcher = InferenceBatcher {
       batch_timeout: Duration::from_micros(250),  // Was 500
       batch_size: 8,  // Was 16
       // ...
   };
   ```

3. **Optimize feature extraction**
   ```rust
   // ❌ Slow: Allocating on every call
   pub fn extract_features(request: &BidRequest) -> Array1<f32> {
       let mut features = Vec::new();
       features.push(/* ... */);
       Array1::from_vec(features)
   }
   
   // ✅ Fast: Reuse pre-allocated buffer
   pub fn extract_features(request: &BidRequest, buffer: &mut [f32]) {
       buffer[0] = /* ... */;
       buffer[1] = /* ... */;
       // ...
   }
   ```

4. **Use faster hardware backend**
   ```bash
   # Switch from CPU to Groq LPU
   export NPU_DEVICE=groq-lpu
   cargo run --release
   ```

### Task 4: Implement Feature Normalization

**Scenario**: Normalize numerical features to [0, 1] range.

```rust
pub struct FeatureNormalizer {
    // Store min and max values for each feature
    feature_mins: Vec<f32>,
    feature_maxs: Vec<f32>,
}

impl FeatureNormalizer {
    pub fn fit(&mut self, data: &Array2<f32>) {
        for col in 0..data.ncols() {
            let column = data.column(col);
            self.feature_mins.push(column.iter().copied().fold(f32::INFINITY, f32::min));
            self.feature_maxs.push(column.iter().copied().fold(f32::NEG_INFINITY, f32::max));
        }
    }
    
    pub fn transform(&self, features: &mut Array1<f32>) {
        for (i, value) in features.iter_mut().enumerate() {
            let min = self.feature_mins[i];
            let max = self.feature_maxs[i];
            
            if max > min {
                *value = (*value - min) / (max - min);
            }
        }
    }
}
```

### Task 5: Model Monitoring and Drift Detection

**Scenario**: Detect when model performance degrades.

```rust
pub struct ModelMonitor {
    baseline_ctr: f64,
    alert_threshold: f64,  // e.g., 0.1 = 10% drop
}

impl ModelMonitor {
    pub fn check_for_drift(&self, current_ctr: f64) -> bool {
        let drop = (self.baseline_ctr - current_ctr) / self.baseline_ctr;
        drop > self.alert_threshold
    }
}

// In your monitoring loop:
let current_ctr = calculate_recent_ctr().await?;
if monitor.check_for_drift(current_ctr) {
    warn!("Model drift detected! CTR dropped from {:.3}% to {:.3}%",
          monitor.baseline_ctr * 100.0,
          current_ctr * 100.0);
    // Send alert to Slack, PagerDuty, etc.
}
```

---

## Best Practices & Tips

### 1. Always Measure Performance

```rust
// ✅ Good: Measure before optimizing
let start = std::time::Instant::now();
let result = expensive_operation();
let duration = start.elapsed();
println!("Operation took {:?}", duration);

// ❌ Bad: Optimizing without measuring
// "I think this is slow, let me make it faster"
```

### 2. Validate Model Outputs

```rust
// ✅ Good: Check for invalid outputs
let predictions = model.infer(features).await?;

for &score in predictions.iter() {
    if !score.is_finite() || score < 0.0 || score > 1.0 {
        error!("Invalid prediction: {}", score);
        return Err(anyhow::anyhow!("Model returned invalid predictions"));
    }
}
```

### 3. Cache Intelligently

```rust
// ✅ Good: Cache stable predictions
let cache_key = format!("user:{}", user_id);
let ttl = 3600;  // 1 hour

// ❌ Bad: Caching time-sensitive features
let cache_key = format!("user:{}:{}", user_id, current_hour);
let ttl = 86400;  // 24 hours - by then, current_hour is wrong!
```

### 4. Handle Failures Gracefully

```rust
// ✅ Good: Fallback to simpler model
let predictions = match npu_provider.infer(features).await {
    Ok(pred) => pred,
    Err(e) => {
        warn!("NPU inference failed: {}, falling back to simple heuristic", e);
        fallback_heuristic(features)
    }
};

// ❌ Bad: Crashing on inference failure
let predictions = npu_provider.infer(features).await.unwrap();
```

### 5. Monitor Everything

```rust
// Track important metrics
INFERENCE_LATENCY.observe(duration.as_secs_f64());
PREDICTIONS_TOTAL.inc();
CACHE_HIT_RATE.set(hits as f64 / (hits + misses) as f64);

// Log feature distributions periodically
if request_count % 1000 == 0 {
    info!("Feature stats: mean={:.3}, std={:.3}", mean, std);
}
```

### 6. Version Your Models

```bash
models/
├── colanet_v1.bin         # Original model
├── colanet_v2.bin         # Updated model
├── colanet_v2_backup.bin  # Backup before deploying v3
└── colanet_v3.bin         # Latest model
```

### 7. Test on Representative Data

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_inference_on_real_data() {
        // Use actual production-like data, not toy examples
        let bid_request = load_sample_bid_request("samples/real_request.json");
        let features = extract_features(&bid_request);
        let predictions = npu_provider.infer(features).await.unwrap();
        
        // Verify reasonable outputs
        assert!(predictions.iter().all(|&x| x >= 0.0 && x <= 1.0));
    }
}
```

---

## Learning Resources

### Machine Learning Fundamentals

1. **[Fast.ai Course](https://course.fast.ai/)** - Practical deep learning for coders (free!)
2. **[Coursera ML Specialization](https://www.coursera.org/specializations/machine-learning-introduction)** - Andrew Ng's ML course
3. **[StatQuest YouTube](https://www.youtube.com/c/joshstarmer)** - Clear explanations of ML concepts

### Real-Time ML & Inference

1. **[Chip Huyen's Blog](https://huyenchip.com/blog/)** - ML systems and production ML
2. **[MLOps Guide](https://ml-ops.org/)** - Best practices for ML in production
3. **[Feature Store Summit](https://www.featurestore.org/)** - Feature engineering talks

### Spiking Neural Networks

1. **[Spiking Neural Networks Paper](https://arxiv.org/abs/1804.08150)** - Introduction to SNNs
2. **[Neuromorphic Computing](https://en.wikipedia.org/wiki/Neuromorphic_engineering)** - Brain-inspired computing

### Rust for ML

1. **[ndarray Documentation](https://docs.rs/ndarray/)** - NumPy-like arrays in Rust
2. **[Polars](https://www.pola.rs/)** - Fast DataFrame library in Rust
3. **[Burn](https://github.com/tracel-ai/burn)** - Rust-native deep learning framework

### Performance Optimization

1. **[The Rust Performance Book](https://nnethercote.github.io/perf-book/)** - Optimizing Rust code
2. **[Flamegraph Guide](https://www.brendangregg.com/flamegraphs.html)** - Understanding flamegraphs

### Books

1. **"Designing Machine Learning Systems" by Chip Huyen** - Production ML systems
2. **"Machine Learning Systems Design" by Ali Aminian** - System design for ML
3. **"Rust for Rustaceans" by Jon Gjengset** - Advanced Rust patterns

---

## Getting Help

### When You're Stuck

1. **Check Metrics First**
   ```bash
   # Is the model even being called?
   curl http://localhost:9091/metrics | grep ml_predictions_total
   
   # What's the latency?
   curl http://localhost:9091/metrics | grep ml_inference_duration
   
   # Cache hit rate?
   curl http://localhost:9091/metrics | grep ml_cache
   ```

2. **Look at Logs**
   ```bash
   # Application logs
   kubectl logs -f deployment/campaign-express | grep -i "inference"
   
   # Or locally
   RUST_LOG=debug cargo run
   ```

3. **Test in Isolation**
   ```rust
   #[tokio::test]
   async fn test_my_change() {
       let features = Array2::zeros((1, 100));
       let result = npu_provider.infer(features).await;
       assert!(result.is_ok());
   }
   ```

4. **Ask for Help**
   - Post in `#ml-engineering` Slack channel
   - Schedule a 1-on-1 with your mentor
   - Pair program with a senior ML engineer

### Common Issues

**"Inference is slow"**
- Check batch size: Too small? (use larger batches)
- Check cache: Is it working? (monitor hit rate)
- Profile the code: Where is time spent? (use flamegraph)

**"Model outputs look wrong"**
- Check feature extraction: Are features in the right range?
- Check normalization: Are you using the same normalization as training?
- Check model version: Are you loading the right model file?

**"Cache hit rate is low"**
- Check cache keys: Are they stable?
- Check TTL: Is it too short?
- Check Redis: Is it running and reachable?

**"Out of memory errors"**
- Check batch size: Too large?
- Check cache size: Is DashMap growing unbounded?
- Check for leaks: Are you clearing old data?

---

## Final Thoughts

As a fresher ML engineer, remember:

- **Production ML is different from research ML** - Focus on reliability, latency, and monitoring
- **Start simple, measure, then optimize** - Don't over-engineer early
- **Models are code** - Version them, test them, review them
- **Inference is a product** - Users depend on it being fast and accurate
- **Learn continuously** - ML and hardware are evolving rapidly

**Key Mindset**:
- 🔬 Experiment rigorously (A/B test everything)
- 📊 Measure obsessively (if you can't measure it, you can't improve it)
- 🚀 Optimize strategically (profile before optimizing)
- 🛡️ Handle failures gracefully (always have fallbacks)

Welcome to the ML team, and happy inferencing! 🤖

---

*For questions specific to CampaignExpress ML, reach out to your team lead or post in the #ml-engineering Slack channel.*
