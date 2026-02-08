//! Integration test for the full bid request/response flow.
//! Requires: Redis and ClickHouse running locally (or use --ignored to skip).

#[cfg(test)]
mod tests {
    use campaign_core::openrtb::*;

    /// Construct a sample OpenRTB bid request for testing.
    fn sample_bid_request() -> BidRequest {
        BidRequest {
            id: "test-req-001".to_string(),
            imp: vec![Impression {
                id: "imp-1".to_string(),
                banner: Some(Banner {
                    w: Some(300),
                    h: Some(250),
                    pos: 1,
                }),
                video: None,
                bidfloor: 0.5,
                bidfloorcur: "USD".to_string(),
                ext: None,
            }],
            site: Some(Site {
                id: Some("site-1".to_string()),
                domain: Some("example.com".to_string()),
                cat: Some(vec!["IAB1".to_string()]),
                page: Some("https://example.com/article".to_string()),
            }),
            app: None,
            device: Some(Device {
                ua: Some("Mozilla/5.0".to_string()),
                ip: Some("203.0.113.1".to_string()),
                geo: Some(Geo {
                    lat: Some(37.7749),
                    lon: Some(-122.4194),
                    country: Some("US".to_string()),
                    region: Some("CA".to_string()),
                    city: Some("San Francisco".to_string()),
                }),
                devicetype: Some(2),
                os: Some("iOS".to_string()),
                osv: Some("17.0".to_string()),
                ifa: None,
            }),
            user: Some(User {
                id: Some("user-12345".to_string()),
                buyeruid: None,
                gender: None,
                keywords: Some("tech,programming".to_string()),
                ext: None,
            }),
            tmax: 100,
            at: 1,
            cur: vec!["USD".to_string()],
            ext: None,
        }
    }

    #[test]
    fn test_bid_request_serialization() {
        let request = sample_bid_request();
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: BidRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "test-req-001");
        assert_eq!(deserialized.imp.len(), 1);
        assert_eq!(deserialized.imp[0].bidfloor, 0.5);
    }

    #[test]
    fn test_bid_response_no_bid() {
        let response = BidResponse::no_bid("test-req-001".to_string());
        assert_eq!(response.id, "test-req-001");
        assert!(response.seatbid.is_empty());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-req-001"));
    }

    #[test]
    fn test_bid_response_with_bid() {
        let response = BidResponse {
            id: "test-req-001".to_string(),
            seatbid: vec![SeatBid {
                bid: vec![Bid {
                    id: "bid-1".to_string(),
                    impid: "imp-1".to_string(),
                    price: 1.50,
                    adid: Some("offer-001".to_string()),
                    nurl: Some("https://example.com/win".to_string()),
                    adm: Some("<img src='ad.jpg' />".to_string()),
                    crid: Some("creative-001".to_string()),
                    w: 300,
                    h: 250,
                    ext: None,
                }],
                seat: Some("campaign-express".to_string()),
                group: 0,
            }],
            bidid: Some("bidid-1".to_string()),
            cur: "USD".to_string(),
            ext: None,
        };

        assert!(!response.seatbid.is_empty());
        assert_eq!(response.seatbid[0].bid[0].price, 1.50);

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: BidResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.seatbid[0].bid[0].price, 1.50);
    }

    #[test]
    fn test_openrtb_roundtrip() {
        let request = sample_bid_request();
        let json = serde_json::to_string_pretty(&request).unwrap();
        let roundtripped: BidRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.id, roundtripped.id);
        assert_eq!(request.imp.len(), roundtripped.imp.len());
        assert_eq!(
            request.user.as_ref().unwrap().id,
            roundtripped.user.as_ref().unwrap().id
        );
        assert_eq!(
            request.device.as_ref().unwrap().os,
            roundtripped.device.as_ref().unwrap().os
        );
    }
}
