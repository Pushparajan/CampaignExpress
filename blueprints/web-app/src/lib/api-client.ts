/**
 * Campaign Express API Client
 *
 * Provides typed methods for all Campaign Express REST API endpoints.
 * Handles authentication, error handling, and token lifecycle.
 *
 * Usage:
 *   import { apiClient } from "@/lib/api-client";
 *   const campaigns = await apiClient.listCampaigns();
 */

import type {
  Campaign,
  Creative,
  MonitoringOverview,
  LoyaltyBalance,
  LoyaltyEarnRequest,
  LoyaltyRedeemRequest,
  IngestEvent,
  ActivationRequest,
  LoginResponse,
  ApiError,
  PricingPlan,
} from "./types";

class ApiClientError extends Error {
  status: number;
  body: ApiError | null;

  constructor(message: string, status: number, body: ApiError | null = null) {
    super(message);
    this.name = "ApiClientError";
    this.status = status;
    this.body = body;
  }
}

class CampaignExpressClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl =
      baseUrl ||
      (typeof window !== "undefined"
        ? process.env.NEXT_PUBLIC_API_URL || ""
        : "");
  }

  // --- Auth ---

  private getToken(): string | null {
    if (typeof window === "undefined") return null;
    return localStorage.getItem("campaign_express_token");
  }

  private setToken(token: string): void {
    if (typeof window === "undefined") return;
    localStorage.setItem("campaign_express_token", token);
  }

  clearToken(): void {
    if (typeof window === "undefined") return;
    localStorage.removeItem("campaign_express_token");
  }

  isAuthenticated(): boolean {
    return this.getToken() !== null;
  }

  // --- HTTP ---

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const token = this.getToken();
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
      this.clearToken();
      if (typeof window !== "undefined") {
        window.location.href = "/login";
      }
      throw new ApiClientError("Unauthorized", 401);
    }

    if (!response.ok) {
      let errorBody: ApiError | null = null;
      try {
        errorBody = await response.json();
      } catch {
        // not JSON
      }
      throw new ApiClientError(
        errorBody?.message || `Request failed: ${response.status}`,
        response.status,
        errorBody
      );
    }

    if (response.status === 204) return undefined as T;
    return response.json();
  }

  // --- Authentication ---

  async login(username: string, password: string): Promise<LoginResponse> {
    const result = await this.request<LoginResponse>(
      "/api/v1/management/auth/login",
      { method: "POST", body: JSON.stringify({ username, password }) }
    );
    this.setToken(result.token);
    return result;
  }

  logout(): void {
    this.clearToken();
  }

  // --- Campaigns ---

  async listCampaigns(): Promise<Campaign[]> {
    return this.request("/api/v1/management/campaigns");
  }

  async getCampaign(id: string): Promise<Campaign> {
    return this.request(`/api/v1/management/campaigns/${id}`);
  }

  async createCampaign(
    data: Omit<Campaign, "id" | "created_at" | "updated_at" | "stats" | "status">
  ): Promise<Campaign> {
    return this.request("/api/v1/management/campaigns", {
      method: "POST",
      body: JSON.stringify(data),
    });
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

  async deleteCampaign(id: string): Promise<void> {
    return this.request(`/api/v1/management/campaigns/${id}`, {
      method: "DELETE",
    });
  }

  // --- Creatives ---

  async listCreatives(): Promise<Creative[]> {
    return this.request("/api/v1/management/creatives");
  }

  async createCreative(
    data: Omit<Creative, "id" | "created_at" | "status">
  ): Promise<Creative> {
    return this.request("/api/v1/management/creatives", {
      method: "POST",
      body: JSON.stringify(data),
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

  async earnStars(data: LoyaltyEarnRequest): Promise<LoyaltyBalance> {
    return this.request("/v1/loyalty/earn", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async redeemStars(data: LoyaltyRedeemRequest): Promise<LoyaltyBalance> {
    return this.request("/v1/loyalty/redeem", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  // --- Omnichannel ---

  async ingestEvent(event: IngestEvent): Promise<{ status: string }> {
    return this.request("/v1/channels/ingest", {
      method: "POST",
      body: JSON.stringify(event),
    });
  }

  async activateChannel(
    request: ActivationRequest
  ): Promise<{ activation_id: string }> {
    return this.request("/v1/channels/activate", {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  // --- Billing ---

  async listPlans(): Promise<PricingPlan[]> {
    return this.request("/api/v1/management/billing/plans");
  }
}

export const apiClient = new CampaignExpressClient();
export { CampaignExpressClient, ApiClientError };
