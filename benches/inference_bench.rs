//! Benchmarks for the NPU inference engine.
//! Run with: cargo bench

#![allow(unused)]

use campaign_core::config::NpuConfig;
use campaign_core::types::UserProfile;
use campaign_npu::NpuEngine;

fn create_test_profile() -> UserProfile {
    UserProfile {
        user_id: "bench-user".to_string(),
        segments: vec![1, 5, 12, 45, 100],
        interests: (0..64).map(|i| (i as f32) * 0.01).collect(),
        geo_region: Some("US-CA".to_string()),
        device_type: Some(campaign_core::types::DeviceType::Mobile),
        recency_score: 0.85,
        ..Default::default()
    }
}

fn main() {
    let config = NpuConfig {
        device: "cpu".to_string(),
        ..Default::default()
    };

    let engine = NpuEngine::new(&config).expect("Failed to create engine");
    let profile = create_test_profile();
    let offer_ids: Vec<String> = (0..64).map(|i| format!("offer-{:04}", i)).collect();

    // Warmup
    for _ in 0..10 {
        engine.score_offers(&profile, &offer_ids).unwrap();
    }

    // Benchmark
    let iterations = 10_000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = engine.score_offers(&profile, &offer_ids).unwrap();
    }

    let elapsed = start.elapsed();
    let per_iter = elapsed / iterations;

    println!("=== Inference Benchmark ===");
    println!("Iterations:  {}", iterations);
    println!("Total time:  {:?}", elapsed);
    println!("Per call:    {:?}", per_iter);
    println!("Throughput:  {:.0} inferences/sec", iterations as f64 / elapsed.as_secs_f64());
    println!("Batch size:  {}", offer_ids.len());
}
