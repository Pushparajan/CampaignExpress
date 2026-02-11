//! gRPC service implementation for the Bidding service.
//! Uses tonic with code generated from bidding.proto.

use campaign_agents::BidProcessor;
use campaign_core::openrtb::BidRequest;
use std::sync::Arc;
use std::time::Instant;
use tonic::{Request, Response, Status};
use tracing::error;

// Include the generated protobuf code.
// In CI/production, proto compilation generates this module.
// For development, we provide a manual definition.
pub mod bidding_proto {
    // When proto compilation works:
    // tonic::include_proto!("campaign.bidding.v1");

    // Manual definitions matching the proto file:
    #[derive(Clone, prost::Message)]
    pub struct BidRequestProto {
        #[prost(string, tag = "1")]
        pub openrtb_json: String,
        #[prost(string, tag = "2")]
        pub request_id: String,
        #[prost(string, tag = "3")]
        pub user_id: String,
        #[prost(uint32, tag = "4")]
        pub timeout_ms: u32,
    }

    #[derive(Clone, prost::Message)]
    pub struct BidResponseProto {
        #[prost(string, tag = "1")]
        pub openrtb_json: String,
        #[prost(string, tag = "2")]
        pub request_id: String,
        #[prost(bool, tag = "3")]
        pub has_bid: bool,
        #[prost(uint64, tag = "4")]
        pub processing_time_us: u64,
        #[prost(string, tag = "5")]
        pub agent_id: String,
    }

    #[derive(Clone, prost::Message)]
    pub struct HealthCheckRequest {
        #[prost(string, tag = "1")]
        pub service: String,
    }

    #[derive(Clone, prost::Message)]
    pub struct HealthCheckResponse {
        #[prost(enumeration = "ServingStatus", tag = "1")]
        pub status: i32,
        #[prost(string, tag = "2")]
        pub node_id: String,
        #[prost(uint32, tag = "3")]
        pub active_agents: u32,
        #[prost(uint64, tag = "4")]
        pub uptime_secs: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, prost::Enumeration)]
    #[repr(i32)]
    pub enum ServingStatus {
        Unknown = 0,
        Serving = 1,
        NotServing = 2,
    }
}

use bidding_proto::*;

/// gRPC Bidding service implementation.
pub struct BiddingServiceImpl {
    processor: Arc<BidProcessor>,
    node_id: String,
    active_agents: u32,
    start_time: Instant,
}

impl BiddingServiceImpl {
    pub fn new(processor: Arc<BidProcessor>, node_id: String, active_agents: u32) -> Self {
        Self {
            processor,
            node_id,
            active_agents,
            start_time: Instant::now(),
        }
    }
}

#[tonic::async_trait]
impl BiddingServiceServer for BiddingServiceImpl {
    async fn process_bid(
        &self,
        request: Request<BidRequestProto>,
    ) -> Result<Response<BidResponseProto>, Status> {
        let start = Instant::now();
        let proto_req = request.into_inner();
        let agent_id = format!("{}-grpc", self.node_id);

        let bid_request: BidRequest = serde_json::from_str(&proto_req.openrtb_json)
            .map_err(|e| Status::invalid_argument(format!("Invalid OpenRTB JSON: {}", e)))?;

        let bid_response = self
            .processor
            .process(&bid_request, &agent_id)
            .await
            .map_err(|e| {
                error!(error = %e, "gRPC bid processing failed");
                Status::internal(format!("Processing failed: {}", e))
            })?;

        let has_bid = !bid_response.seatbid.is_empty();
        let openrtb_json = serde_json::to_string(&bid_response)
            .map_err(|e| Status::internal(format!("Serialization failed: {}", e)))?;

        Ok(Response::new(BidResponseProto {
            openrtb_json,
            request_id: bid_response.id,
            has_bid,
            processing_time_us: start.elapsed().as_micros() as u64,
            agent_id,
        }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: ServingStatus::Serving as i32,
            node_id: self.node_id.clone(),
            active_agents: self.active_agents,
            uptime_secs: self.start_time.elapsed().as_secs(),
        }))
    }

    type StreamBidsStream =
        tokio_stream::wrappers::ReceiverStream<Result<BidResponseProto, Status>>;

    async fn stream_bids(
        &self,
        request: Request<tonic::Streaming<BidRequestProto>>,
    ) -> Result<Response<Self::StreamBidsStream>, Status> {
        let processor = self.processor.clone();
        let node_id = self.node_id.clone();
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            while let Ok(Some(proto_req)) = stream.message().await {
                let agent_id = format!("{}-grpc-stream", node_id);
                let start = Instant::now();

                let result = match serde_json::from_str::<BidRequest>(&proto_req.openrtb_json) {
                    Ok(bid_request) => match processor.process(&bid_request, &agent_id).await {
                        Ok(bid_response) => {
                            let has_bid = !bid_response.seatbid.is_empty();
                            let openrtb_json =
                                serde_json::to_string(&bid_response).unwrap_or_default();
                            Ok(BidResponseProto {
                                openrtb_json,
                                request_id: bid_response.id,
                                has_bid,
                                processing_time_us: start.elapsed().as_micros() as u64,
                                agent_id,
                            })
                        }
                        Err(e) => Err(Status::internal(e.to_string())),
                    },
                    Err(e) => Err(Status::invalid_argument(e.to_string())),
                };

                if tx.send(result).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
            rx,
        )))
    }
}

/// Trait definition for the gRPC service (normally auto-generated by tonic).
#[tonic::async_trait]
pub trait BiddingServiceServer: Send + Sync + 'static {
    async fn process_bid(
        &self,
        request: Request<BidRequestProto>,
    ) -> Result<Response<BidResponseProto>, Status>;

    async fn health_check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status>;

    type StreamBidsStream: tokio_stream::Stream<Item = Result<BidResponseProto, Status>>
        + Send
        + 'static;

    async fn stream_bids(
        &self,
        request: Request<tonic::Streaming<BidRequestProto>>,
    ) -> Result<Response<Self::StreamBidsStream>, Status>;
}
