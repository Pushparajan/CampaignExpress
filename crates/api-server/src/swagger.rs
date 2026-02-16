//! OpenAPI specification and Swagger UI configuration.

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Campaign Express API",
        version = "0.1.0",
        description = "High-throughput real-time ad offer personalization platform.\n\nSupports OpenRTB 2.6 bidding, loyalty programs, DSP integration, and omnichannel activation.",
        license(name = "MIT"),
    ),
    tags(
        (name = "Bidding", description = "OpenRTB 2.6 bid request/response endpoints"),
        (name = "Operations", description = "Health, readiness, and liveness probes"),
        (name = "Loyalty", description = "Loyalty program — earn/redeem stars, tier management"),
        (name = "DSP", description = "Demand-Side Platform bid routing and win notifications"),
        (name = "Channels", description = "Omnichannel ingest and activation endpoints"),
    ),
    paths(
        // Bidding
        crate::rest::handle_bid,
        // Operations
        crate::rest::health_check,
        crate::rest::readiness,
        crate::rest::liveness,
        // Loyalty
        crate::loyalty_rest::handle_earn_stars,
        crate::loyalty_rest::handle_redeem,
        crate::loyalty_rest::handle_balance,
        crate::loyalty_rest::handle_reward_signal,
        // DSP
        crate::dsp_rest::handle_dsp_bid,
        crate::dsp_rest::handle_dsp_win,
        crate::dsp_rest::handle_dsp_status,
        // Channels
        crate::channel_rest::handle_ingest,
        crate::channel_rest::handle_activate,
        crate::channel_rest::handle_sendgrid_webhook,
        crate::channel_rest::handle_email_analytics,
        crate::channel_rest::handle_all_email_analytics,
    ),
    components(schemas(
        // OpenRTB types
        campaign_core::openrtb::BidRequest,
        campaign_core::openrtb::BidResponse,
        campaign_core::openrtb::Impression,
        campaign_core::openrtb::Banner,
        campaign_core::openrtb::Video,
        campaign_core::openrtb::Site,
        campaign_core::openrtb::App,
        campaign_core::openrtb::Device,
        campaign_core::openrtb::Geo,
        campaign_core::openrtb::User,
        campaign_core::openrtb::SeatBid,
        campaign_core::openrtb::Bid,
        // REST error/health types
        crate::rest::ErrorResponse,
        crate::rest::HealthResponse,
        // Loyalty types
        campaign_core::loyalty::LoyaltyTier,
        campaign_core::loyalty::LoyaltyChannel,
        campaign_core::loyalty::EarnStarsRequest,
        campaign_core::loyalty::EarnStarsResponse,
        campaign_core::loyalty::RedemptionTier,
        campaign_core::loyalty::RedeemRequest,
        campaign_core::loyalty::RedeemResponse,
        campaign_core::loyalty::LoyaltyRewardSignal,
        campaign_core::loyalty::LoyaltySignalType,
        crate::loyalty_rest::LoyaltyBalanceResponse,
        // DSP types
        campaign_core::dsp::DspPlatform,
        campaign_core::dsp::DspBidResponse,
        campaign_core::dsp::DspBid,
        crate::dsp_rest::DspBidApiRequest,
        crate::dsp_rest::DspBidApiResponse,
        crate::dsp_rest::DspWinRequest,
        crate::dsp_rest::DspStatusResponse,
        // Channel types
        campaign_core::channels::IngestSource,
        campaign_core::channels::IngestEvent,
        campaign_core::channels::IngestEventType,
        campaign_core::channels::GeoLocation,
        campaign_core::channels::ActivationChannel,
        campaign_core::channels::ActivationRequest,
        campaign_core::channels::ActivationContent,
        campaign_core::channels::ActivationResult,
        campaign_core::channels::ActivationStatus,
        campaign_core::channels::EmailWebhookEvent,
        campaign_core::channels::EmailEventType,
        campaign_core::channels::EmailAnalytics,
        crate::channel_rest::IngestResponse,
        crate::channel_rest::ChannelErrorResponse,
    ))
)]
pub struct ApiDoc;
