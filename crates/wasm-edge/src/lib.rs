#![warn(clippy::unwrap_used)]

//! AWS Lambda@Edge bid request preprocessor.
//!
//! Runs on CloudFront edge locations via Lambda@Edge to:
//! - Validate and enrich OpenRTB bid requests before they reach the origin
//! - Route requests to the nearest Campaign Express cluster
//! - Reject malformed requests at the edge (saves origin compute)
//! - Inject edge metadata (region, latency) into the request
//!
//! Deployment: Built as a Lambda function, attached to a CloudFront
//! distribution's viewer-request or origin-request event.

use serde::{Deserialize, Serialize};

/// Incoming request at the CloudFront edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRequest {
    /// Unique request identifier.
    pub request_id: String,
    /// Raw OpenRTB JSON body.
    pub openrtb_json: String,
    /// AWS region of the edge location (e.g., "us-east-1").
    pub edge_region: String,
}

/// Response from edge preprocessing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeResponse {
    /// Echoed request identifier.
    pub request_id: String,
    /// Processed OpenRTB JSON (validated + enriched).
    pub openrtb_json: String,
    /// Edge-side processing latency in milliseconds.
    pub edge_latency_ms: u64,
    /// Origin cluster the request was routed to.
    pub routed_to: String,
}

/// CloudFront Lambda@Edge event structures (simplified).
/// Maps to the CloudFront viewer-request/origin-request event format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontEvent {
    #[serde(rename = "Records")]
    pub records: Vec<CloudFrontRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontRecord {
    pub cf: CloudFrontData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontData {
    pub request: CloudFrontRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontRequest {
    pub uri: String,
    pub method: String,
    #[serde(default)]
    pub body: Option<CloudFrontBody>,
    #[serde(default)]
    pub headers: serde_json::Value,
    #[serde(default)]
    pub origin: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontBody {
    pub data: String,
    pub encoding: String,
}

/// Edge response to return to CloudFront (reject with 400 or forward).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFrontResponse {
    pub status: String,
    #[serde(rename = "statusDescription")]
    pub status_description: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub headers: serde_json::Value,
}

/// Route table: maps AWS edge regions to the nearest Campaign Express origin.
fn route_to_origin(edge_region: &str) -> &'static str {
    match edge_region {
        r if r.starts_with("us-east") => "https://api-east.campaign-express.io",
        r if r.starts_with("us-west") => "https://api-west.campaign-express.io",
        r if r.starts_with("eu-") => "https://api-eu.campaign-express.io",
        r if r.starts_with("ap-") => "https://api-apac.campaign-express.io",
        _ => "https://api.campaign-express.io",
    }
}

/// Preprocess a bid request at the edge.
///
/// Validates the OpenRTB JSON, injects edge metadata, and selects the
/// nearest origin. Invalid requests are rejected here instead of
/// consuming origin compute.
pub fn preprocess_request(request: &EdgeRequest) -> Result<EdgeResponse, String> {
    if !validate_openrtb(&request.openrtb_json) {
        return Err("Invalid OpenRTB: missing 'id' or 'imp' field".to_string());
    }

    let enriched_json = enrich_with_edge_metadata(&request.openrtb_json, &request.edge_region);

    let origin = route_to_origin(&request.edge_region);

    Ok(EdgeResponse {
        request_id: request.request_id.clone(),
        openrtb_json: enriched_json,
        edge_latency_ms: 0,
        routed_to: origin.to_string(),
    })
}

/// Handle a CloudFront Lambda@Edge event (viewer-request trigger).
///
/// Returns either the modified request (forward to origin) or a
/// rejection response (400) for invalid bid requests.
pub fn handle_cloudfront_event(
    event: &CloudFrontEvent,
) -> Result<CloudFrontRequest, CloudFrontResponse> {
    let record = event.records.first().ok_or_else(|| CloudFrontResponse {
        status: "400".to_string(),
        status_description: "Bad Request".to_string(),
        body: Some(r#"{"error":"empty event"}"#.to_string()),
        headers: serde_json::json!({}),
    })?;

    let cf_request = &record.cf.request;

    // Only process POST /v1/bid requests; pass everything else through
    if cf_request.method != "POST" || cf_request.uri != "/v1/bid" {
        return Ok(cf_request.clone());
    }

    let body_data = cf_request
        .body
        .as_ref()
        .map(|b| b.data.clone())
        .unwrap_or_default();

    if !validate_openrtb(&body_data) {
        return Err(CloudFrontResponse {
            status: "400".to_string(),
            status_description: "Bad Request".to_string(),
            body: Some(r#"{"error":"invalid OpenRTB: missing id or imp"}"#.to_string()),
            headers: serde_json::json!({
                "content-type": [{"key": "Content-Type", "value": "application/json"}]
            }),
        });
    }

    Ok(cf_request.clone())
}

/// Validate an OpenRTB bid request JSON at the edge.
/// Returns true if the JSON is valid and contains required fields.
pub fn validate_openrtb(json: &str) -> bool {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json);
    match parsed {
        Ok(v) => {
            let has_id = v.get("id").is_some_and(|id| id.is_string());
            let has_imp = v
                .get("imp")
                .is_some_and(|imp| imp.as_array().is_some_and(|arr| !arr.is_empty()));
            has_id && has_imp
        }
        Err(_) => false,
    }
}

/// Enrich OpenRTB JSON with edge metadata in the `ext` field.
fn enrich_with_edge_metadata(json: &str, edge_region: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(mut v) => {
            let ext = v
                .as_object_mut()
                .and_then(|obj| {
                    obj.entry("ext")
                        .or_insert_with(|| serde_json::json!({}))
                        .as_object_mut()
                });
            if let Some(ext) = ext {
                ext.insert(
                    "edge_region".to_string(),
                    serde_json::json!(edge_region),
                );
                ext.insert("edge_processed".to_string(), serde_json::json!(true));
            }
            serde_json::to_string(&v).unwrap_or_else(|_| json.to_string())
        }
        Err(_) => json.to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_openrtb_valid() {
        let json = r#"{"id":"req-1","imp":[{"id":"imp-1"}]}"#;
        assert!(validate_openrtb(json));
    }

    #[test]
    fn test_validate_openrtb_invalid_missing_id() {
        assert!(!validate_openrtb(r#"{"imp":[{"id":"imp-1"}]}"#));
    }

    #[test]
    fn test_validate_openrtb_invalid_empty_imp() {
        assert!(!validate_openrtb(r#"{"id":"req-1","imp":[]}"#));
    }

    #[test]
    fn test_validate_openrtb_invalid_json() {
        assert!(!validate_openrtb("not json"));
    }

    #[test]
    fn test_validate_openrtb_empty_object() {
        assert!(!validate_openrtb("{}"));
    }

    #[test]
    fn test_preprocess_request_valid() {
        let req = EdgeRequest {
            request_id: "test-1".to_string(),
            openrtb_json: r#"{"id":"req-1","imp":[{"id":"imp-1"}]}"#.to_string(),
            edge_region: "us-east-1".to_string(),
        };
        let resp = preprocess_request(&req).unwrap();
        assert_eq!(resp.request_id, "test-1");
        assert_eq!(resp.routed_to, "https://api-east.campaign-express.io");
        assert!(resp.openrtb_json.contains("edge_region"));
    }

    #[test]
    fn test_preprocess_request_invalid() {
        let req = EdgeRequest {
            request_id: "test-2".to_string(),
            openrtb_json: "{}".to_string(),
            edge_region: "us-east-1".to_string(),
        };
        assert!(preprocess_request(&req).is_err());
    }

    #[test]
    fn test_route_to_origin() {
        assert_eq!(
            route_to_origin("us-east-1"),
            "https://api-east.campaign-express.io"
        );
        assert_eq!(
            route_to_origin("us-west-2"),
            "https://api-west.campaign-express.io"
        );
        assert_eq!(
            route_to_origin("eu-west-1"),
            "https://api-eu.campaign-express.io"
        );
        assert_eq!(
            route_to_origin("ap-southeast-1"),
            "https://api-apac.campaign-express.io"
        );
        assert_eq!(
            route_to_origin("sa-east-1"),
            "https://api.campaign-express.io"
        );
    }

    #[test]
    fn test_enrich_with_edge_metadata() {
        let json = r#"{"id":"req-1","imp":[{"id":"imp-1"}]}"#;
        let enriched = enrich_with_edge_metadata(json, "us-east-1");
        let parsed: serde_json::Value = serde_json::from_str(&enriched).unwrap();
        assert_eq!(parsed["ext"]["edge_region"], "us-east-1");
        assert_eq!(parsed["ext"]["edge_processed"], true);
    }

    #[test]
    fn test_handle_cloudfront_event_valid_bid() {
        let event = CloudFrontEvent {
            records: vec![CloudFrontRecord {
                cf: CloudFrontData {
                    request: CloudFrontRequest {
                        uri: "/v1/bid".to_string(),
                        method: "POST".to_string(),
                        body: Some(CloudFrontBody {
                            data: r#"{"id":"req-1","imp":[{"id":"imp-1"}]}"#.to_string(),
                            encoding: "text".to_string(),
                        }),
                        headers: serde_json::json!({}),
                        origin: None,
                    },
                },
            }],
        };
        assert!(handle_cloudfront_event(&event).is_ok());
    }

    #[test]
    fn test_handle_cloudfront_event_invalid_body() {
        let event = CloudFrontEvent {
            records: vec![CloudFrontRecord {
                cf: CloudFrontData {
                    request: CloudFrontRequest {
                        uri: "/v1/bid".to_string(),
                        method: "POST".to_string(),
                        body: Some(CloudFrontBody {
                            data: "{}".to_string(),
                            encoding: "text".to_string(),
                        }),
                        headers: serde_json::json!({}),
                        origin: None,
                    },
                },
            }],
        };
        let result = handle_cloudfront_event(&event);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().status, "400");
    }

    #[test]
    fn test_handle_cloudfront_event_passthrough_non_bid() {
        let event = CloudFrontEvent {
            records: vec![CloudFrontRecord {
                cf: CloudFrontData {
                    request: CloudFrontRequest {
                        uri: "/health".to_string(),
                        method: "GET".to_string(),
                        body: None,
                        headers: serde_json::json!({}),
                        origin: None,
                    },
                },
            }],
        };
        assert!(handle_cloudfront_event(&event).is_ok());
    }
}
