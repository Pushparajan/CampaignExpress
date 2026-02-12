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

export interface CampaignDetailStats {
  campaign_id: string;
  impressions: number;
  clicks: number;
  conversions: number;
  spend: number;
  ctr: number;
  win_rate: number;
  avg_bid: number;
  avg_win_price: number;
  hourly_data: HourlyDataPoint[];
}

export interface AuditLogEntry {
  id: string;
  user: string;
  action: string;
  resource_type: string;
  resource_id: string;
  details: string;
  timestamp: string;
}

export interface LoginResponse {
  token: string;
}

export interface ApiError {
  error: string;
  message: string;
  status: number;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
}

export type CampaignCreatePayload = Omit<
  Campaign,
  "id" | "created_at" | "updated_at" | "stats" | "status"
>;

export type CampaignUpdatePayload = Partial<CampaignCreatePayload> & {
  status?: Campaign["status"];
};

export type CreativeCreatePayload = Omit<Creative, "id" | "created_at" | "status">;

export type CreativeUpdatePayload = Partial<CreativeCreatePayload> & {
  status?: Creative["status"];
};

// ─── Journey Types ──────────────────────────────────────────────────────

export interface Journey {
  id: string;
  name: string;
  description: string;
  status: "draft" | "active" | "paused" | "completed" | "archived";
  trigger: JourneyTrigger;
  steps: JourneyStep[];
  created_at: string;
  updated_at: string;
  version: number;
}

export interface JourneyTrigger {
  type: "event_based" | "segment_entry" | "schedule_based" | "api_based" | "bid_context";
  config: Record<string, unknown>;
}

export interface JourneyStep {
  id: string;
  step_type: string;
  config: Record<string, unknown>;
  position: number;
  next_steps: StepTransition[];
}

export interface StepTransition {
  target_step: string;
  condition?: string;
}

export interface JourneyStats {
  journey_id: string;
  total_entered: number;
  active: number;
  completed: number;
  exited: number;
  error: number;
  avg_completion_time_secs: number;
  step_conversion_rates: Record<string, number>;
}

// ─── DCO Types ──────────────────────────────────────────────────────────

export interface DcoTemplate {
  id: string;
  name: string;
  description: string;
  components: TemplateComponent[];
  rules: AssemblyRule[];
  status: "draft" | "active" | "paused" | "archived";
  created_at: string;
  updated_at: string;
}

export interface TemplateComponent {
  id: string;
  component_type: string;
  variants: ComponentVariant[];
  required: boolean;
}

export interface ComponentVariant {
  id: string;
  name: string;
  content: string;
  asset_url?: string;
  metadata: Record<string, unknown>;
  performance: VariantPerformance;
}

export interface VariantPerformance {
  impressions: number;
  clicks: number;
  conversions: number;
  ctr: number;
  cvr: number;
  revenue: number;
}

export interface AssemblyRule {
  id: string;
  name: string;
  condition: string;
  component_id: string;
  preferred_variants: string[];
  priority: number;
}

// ─── CDP Types ──────────────────────────────────────────────────────────

export interface CdpPlatformConfig {
  platform: "salesforce_data_cloud" | "adobe_real_time_cdp" | "twilio_segment" | "tealium" | "hightouch";
  api_endpoint: string;
  api_key: string;
  enabled: boolean;
  sync_interval_secs: number;
  batch_size: number;
  field_mappings: Record<string, string>;
}

export interface SyncEvent {
  id: string;
  platform: string;
  direction: "inbound" | "outbound" | "bidirectional";
  record_count: number;
  status: "pending" | "in_progress" | "completed" | "failed" | "partial_success";
  started_at: string;
  completed_at?: string;
  error?: string;
}

// ─── Experiment Types ───────────────────────────────────────────────────

export interface Experiment {
  id: string;
  name: string;
  description: string;
  status: "draft" | "running" | "paused" | "completed" | "cancelled";
  variants: ExperimentVariant[];
  traffic_allocation: number;
  metric: string;
  min_sample_size: number;
  created_at: string;
  updated_at: string;
}

export interface ExperimentVariant {
  id: string;
  name: string;
  weight: number;
  is_control: boolean;
  config: Record<string, unknown>;
  results: {
    sample_size: number;
    conversions: number;
    revenue: number;
    conversion_rate: number;
    confidence: number;
    lift: number;
  };
}

// ─── Platform Types ────────────────────────────────────────────────────

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  status: "active" | "suspended" | "trial" | "cancelled";
  pricing_tier: "free" | "starter" | "professional" | "enterprise" | "custom";
  owner_id: string;
  settings: TenantSettings;
  usage: TenantUsage;
  created_at: string;
  updated_at: string;
}

export interface TenantSettings {
  max_campaigns: number;
  max_users: number;
  max_offers_per_hour: number;
  max_api_calls_per_day: number;
  features_enabled: string[];
  data_retention_days: number;
}

export interface TenantUsage {
  campaigns_active: number;
  users_count: number;
  offers_served_today: number;
  api_calls_today: number;
  storage_bytes: number;
}

export interface Role {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  is_system: boolean;
  created_at: string;
}

export interface ComplianceStatus {
  framework: string;
  status: string;
  last_audit?: string;
  next_audit?: string;
  findings: string[];
}

export interface DataSubjectRequest {
  id: string;
  tenant_id: string;
  user_identifier: string;
  request_type: "erasure" | "access" | "rectification" | "restriction" | "portability";
  status: "pending" | "in_progress" | "completed" | "failed";
  requested_at: string;
  completed_at?: string;
}

// ─── Billing Types ─────────────────────────────────────────────────────

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

export interface Subscription {
  id: string;
  tenant_id: string;
  plan_id: string;
  provider: "stripe" | "chargebee" | "manual";
  status: "active" | "past_due" | "cancelled" | "trialing" | "paused";
  current_period_start: string;
  current_period_end: string;
  cancel_at_period_end: boolean;
  created_at: string;
}

export interface Invoice {
  id: string;
  tenant_id: string;
  subscription_id: string;
  amount: number;
  currency: string;
  status: string;
  line_items: InvoiceLineItem[];
  issued_at: string;
  due_at: string;
  paid_at?: string;
}

export interface InvoiceLineItem {
  description: string;
  quantity: number;
  unit_price: number;
  amount: number;
}

export interface UsageSummary {
  tenant_id: string;
  period: string;
  meters: MeterSummary[];
  total_cost: number;
}

export interface MeterSummary {
  meter_type: string;
  total_quantity: number;
  unit_price: number;
  line_total: number;
  quota?: number;
  usage_percent: number;
}

export interface OnboardingProgress {
  tenant_id: string;
  steps: OnboardingStep[];
  started_at: string;
  completed_at?: string;
  completion_percent: number;
}

export interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  status: "not_started" | "in_progress" | "completed" | "skipped";
  order: number;
  required: boolean;
  completed_at?: string;
}

// ─── Ops Types ─────────────────────────────────────────────────────────

export interface StatusComponent {
  id: string;
  name: string;
  description: string;
  status: "operational" | "degraded_performance" | "partial_outage" | "major_outage" | "maintenance";
  group: string;
  updated_at: string;
}

export interface Incident {
  id: string;
  title: string;
  description: string;
  severity: "critical" | "major" | "minor" | "info";
  status: "detected" | "investigating" | "identified" | "monitoring" | "resolved";
  affected_components: string[];
  created_at: string;
  resolved_at?: string;
}

export interface SlaTarget {
  name: string;
  target_percent: number;
  current_percent: number;
  measurement_window: string;
  last_incident?: string;
}

export interface BackupSchedule {
  id: string;
  target: string;
  cron_expression: string;
  retention_days: number;
  enabled: boolean;
  last_run?: string;
  next_run: string;
}
