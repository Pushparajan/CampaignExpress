use thiserror::Error;

pub type CampaignResult<T> = Result<T, CampaignError>;

#[derive(Error, Debug)]
pub enum CampaignError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("NPU inference error: {0}")]
    Inference(String),

    #[error("Model loading error: {0}")]
    ModelLoad(String),

    #[error("NATS messaging error: {0}")]
    Nats(String),

    #[error("Redis cache error: {0}")]
    Cache(String),

    #[error("ClickHouse analytics error: {0}")]
    Analytics(String),

    #[error("OpenRTB validation error: {0}")]
    Validation(String),

    #[error("Bid processing error: {0}")]
    BidProcessing(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
