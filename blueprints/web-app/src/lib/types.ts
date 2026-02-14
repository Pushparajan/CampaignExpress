// Campaign Express API Types
// Auto-generated from Campaign Express platform types

export interface Campaign {
  id: string;
  name: string;
  status: "active" | "paused" | "draft" | "completed" | "error";
  budget: number;
  daily_budget: number;
  pacing: "even" | "accelerated" | "asap";
  targeting: CampaignTargeting;
  schedule_start: string;
  schedule_end: string;
  created_at: string;
  updated_at: string;
  stats: CampaignStats;
}

export interface CampaignTargeting {
  geo: string[];
  segments: string[];
  devices: string[];
  floor_price: number;
}

export interface CampaignStats {
  impressions: number;
  clicks: number;
  conversions: number;
  spend: number;
  ctr: number;
  win_rate: number;
}

export interface Creative {
  id: string;
  campaign_id: string;
  name: string;
  format: "banner" | "native" | "video" | "interstitial";
  asset_url: string;
  width: number;
  height: number;
  status: "active" | "paused" | "pending_review" | "rejected";
  metadata: Record<string, string>;
  created_at: string;
}

export interface MonitoringOverview {
  total_campaigns: number;
  active_campaigns: number;
  total_impressions: number;
  total_clicks: number;
  total_spend: number;
  avg_ctr: number;
  avg_latency_us: number;
  active_pods: number;
  offers_per_hour: number;
  cache_hit_rate: number;
  no_bid_rate: number;
  error_rate: number;
}

export interface HourlyDataPoint {
  hour: string;
  impressions: number;
  clicks: number;
  spend: number;
  avg_bid: number;
  win_rate: number;
}

export interface LoyaltyBalance {
  user_id: string;
  tier: "green" | "gold" | "reserve";
  stars_balance: number;
  stars_qualifying: number;
  lifetime_stars: number;
  total_redemptions: number;
  next_tier_progress: number;
}

export interface LoyaltyEarnRequest {
  user_id: string;
  amount_cents: number;
  category: string;
  store_id?: string;
}

export interface LoyaltyRedeemRequest {
  user_id: string;
  stars: number;
  reward_id: string;
}

export interface IngestEvent {
  event_id: string;
  source: "mobile_app" | "pos" | "kiosk" | "web" | "call_center" | "partner_api";
  event_type: "purchase" | "product_view" | "cart_add" | "cart_abandon" | "app_open" | "page_view" | "search";
  user_id?: string;
  payload: Record<string, unknown>;
  occurred_at: string;
}

export interface ActivationRequest {
  user_id: string;
  channel: "push_notification" | "sms" | "email" | "web_personalization" | "in_app_message";
  message_template: string;
  personalization_data: Record<string, unknown>;
}

export interface LoginResponse {
  token: string;
}

export interface ApiError {
  error: string;
  message: string;
  status: number;
}

export interface PricingPlan {
  id: string;
  name: string;
  tier: string;
  monthly_price: number;
  annual_price: number;
  included_offers: number;
  included_api_calls: number;
  features: string[];
}

export interface WebEvent {
  event_type: string;
  name?: string;
  properties: Record<string, unknown>;
  timestamp: string;
}

export interface WebEventBatch {
  api_key: string;
  anonymous_id: string;
  session_id: string;
  events: WebEvent[];
  sent_at: string;
}
