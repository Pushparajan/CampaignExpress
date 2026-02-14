/**
 * Campaign Express Mobile API Client
 *
 * Typed client for all Campaign Express REST and SDK endpoints.
 * Uses SecureStore for token persistence on mobile.
 */

import * as SecureStore from "expo-secure-store";

const API_URL = process.env.CE_API_URL || "https://api.campaignexpress.io";
const TOKEN_KEY = "ce_auth_token";

export interface Campaign {
  id: string;
  name: string;
  status: "active" | "paused" | "draft" | "completed" | "error";
  budget: number;
  daily_budget: number;
  stats: {
    impressions: number;
    clicks: number;
    conversions: number;
    spend: number;
    ctr: number;
    win_rate: number;
  };
  schedule_start: string;
  schedule_end: string;
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

export interface MonitoringOverview {
  total_campaigns: number;
  active_campaigns: number;
  total_impressions: number;
  total_clicks: number;
  total_spend: number;
  avg_ctr: number;
  avg_latency_us: number;
  offers_per_hour: number;
  cache_hit_rate: number;
}

export interface Offer {
  offer_id: string;
  campaign_id: string;
  creative_url: string;
  landing_url: string;
  bid_floor: number;
  priority: number;
}

class CampaignExpressMobileClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || API_URL;
  }

  // --- Token Management ---

  async getToken(): Promise<string | null> {
    try {
      return await SecureStore.getItemAsync(TOKEN_KEY);
    } catch {
      return null;
    }
  }

  async setToken(token: string): Promise<void> {
    await SecureStore.setItemAsync(TOKEN_KEY, token);
  }

  async clearToken(): Promise<void> {
    await SecureStore.deleteItemAsync(TOKEN_KEY);
  }

  async isAuthenticated(): Promise<boolean> {
    const token = await this.getToken();
    return token !== null;
  }

  // --- HTTP ---

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const token = await this.getToken();
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(options.headers as Record<string, string>),
    };

    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    const response = await fetch(`${this.baseUrl}${path}`, {
      ...options,
      headers,
    });

    if (response.status === 401) {
      await this.clearToken();
      throw new Error("Unauthorized");
    }

    if (!response.ok) {
      const errorBody = await response.json().catch(() => null);
      throw new Error(errorBody?.message || `Request failed: ${response.status}`);
    }

    if (response.status === 204) return undefined as T;
    return response.json();
  }

  // --- Auth ---

  async login(username: string, password: string): Promise<void> {
    const result = await this.request<{ token: string }>(
      "/api/v1/management/auth/login",
      { method: "POST", body: JSON.stringify({ username, password }) }
    );
    await this.setToken(result.token);
  }

  async logout(): Promise<void> {
    await this.clearToken();
  }

  // --- Campaigns ---

  async listCampaigns(): Promise<Campaign[]> {
    return this.request("/api/v1/management/campaigns");
  }

  async getCampaign(id: string): Promise<Campaign> {
    return this.request(`/api/v1/management/campaigns/${id}`);
  }

  async pauseCampaign(id: string): Promise<Campaign> {
    return this.request(`/api/v1/management/campaigns/${id}/pause`, {
      method: "POST",
    });
  }

  async resumeCampaign(id: string): Promise<Campaign> {
    return this.request(`/api/v1/management/campaigns/${id}/resume`, {
      method: "POST",
    });
  }

  // --- Monitoring ---

  async getOverview(): Promise<MonitoringOverview> {
    return this.request("/api/v1/management/monitoring/overview");
  }

  // --- Loyalty ---

  async getLoyaltyBalance(userId: string): Promise<LoyaltyBalance> {
    return this.request(`/v1/loyalty/balance/${userId}`);
  }

  async earnStars(userId: string, amountCents: number, category: string): Promise<LoyaltyBalance> {
    return this.request("/v1/loyalty/earn", {
      method: "POST",
      body: JSON.stringify({
        user_id: userId,
        amount_cents: amountCents,
        category,
      }),
    });
  }

  async redeemStars(userId: string, stars: number, rewardId: string): Promise<LoyaltyBalance> {
    return this.request("/v1/loyalty/redeem", {
      method: "POST",
      body: JSON.stringify({
        user_id: userId,
        stars,
        reward_id: rewardId,
      }),
    });
  }

  // --- Omnichannel Events ---

  async ingestEvent(event: {
    source: string;
    event_type: string;
    user_id?: string;
    payload: Record<string, unknown>;
  }): Promise<{ status: string }> {
    return this.request("/v1/channels/ingest", {
      method: "POST",
      body: JSON.stringify({
        event_id: `mobile-${Date.now()}`,
        source: "mobile_app",
        occurred_at: new Date().toISOString(),
        ...event,
      }),
    });
  }

  // --- Push Registration ---

  async registerPushToken(
    deviceId: string,
    pushToken: string,
    platform: "ios" | "android"
  ): Promise<void> {
    await this.request("/v1/sdk/devices/register", {
      method: "POST",
      body: JSON.stringify({
        device_id: deviceId,
        push_token: pushToken,
        platform,
        push_enabled: true,
        sdk_version: "0.1.0",
      }),
    });
  }
}

export const mobileClient = new CampaignExpressMobileClient();
