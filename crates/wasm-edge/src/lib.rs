//! WASM edge worker stub for Cloudflare Workers.
//!
//! This module provides a minimal edge-side bid request preprocessor
//! that can run on Cloudflare Workers (WASM). In production, it would:
//! - Validate and enrich bid requests at the edge
//! - Route requests to the nearest Campaign Express cluster
//! - Implement edge-side caching for frequently seen user segments
//!
//! Currently a stub — full implementation deferred.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeRequest {
    pub request_id: String,
    pub openrtb_json: String,
    pub edge_region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeResponse {
    pub request_id: String,
    pub openrtb_json: String,
    pub edge_latency_ms: u64,
    pub routed_to: String,
}

/// Preprocess a bid request at the edge.
/// Stub implementation — returns the request unchanged.
pub fn preprocess_request(request: &EdgeRequest) -> EdgeResponse {
    EdgeResponse {
        request_id: request.request_id.clone(),
        openrtb_json: request.openrtb_json.clone(),
        edge_latency_ms: 0,
        routed_to: "default-cluster".to_string(),
    }
}

/// Validate an OpenRTB bid request JSON at the edge.
/// Returns true if the JSON is valid and contains required fields.
pub fn validate_openrtb(json: &str) -> bool {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    match parsed {
        Ok(v) => v.get("id").is_some() && v.get("imp").is_some(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openrtb_valid() {
        let json = r#"{"id":"req-1","imp":[{"id":"imp-1"}]}"#;
        assert!(validate_openrtb(json));
    }

    #[test]
    fn test_validate_openrtb_invalid() {
        assert!(!validate_openrtb("{}"));
        assert!(!validate_openrtb("not json"));
    }

    #[test]
    fn test_preprocess_request() {
        let req = EdgeRequest {
            request_id: "test-1".to_string(),
            openrtb_json: "{}".to_string(),
            edge_region: "us-east-1".to_string(),
        };
        let resp = preprocess_request(&req);
        assert_eq!(resp.request_id, "test-1");
    }
}
