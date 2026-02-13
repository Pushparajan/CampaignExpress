import type {
  Campaign,
  CampaignCreatePayload,
  CampaignUpdatePayload,
  CampaignDetailStats,
  Creative,
  CreativeCreatePayload,
  CreativeUpdatePayload,
  MonitoringOverview,
  AuditLogEntry,
  LoginResponse,
  ApiError,
  Journey,
  JourneyStats,
  DcoTemplate,
  CdpPlatformConfig,
  SyncEvent,
  Experiment,
  Tenant,
  Role,
  ComplianceStatus,
  DataSubjectRequest,
  PricingPlan,
  Subscription,
  Invoice,
  UsageSummary,
  OnboardingProgress,
  Incident,
  BackupSchedule,
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

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = "") {
    this.baseUrl = baseUrl;
  }

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

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
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
        // response body is not JSON
      }
      throw new ApiClientError(
        errorBody?.message || `Request failed with status ${response.status}`,
        response.status,
        errorBody
      );
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  // Auth
  async login(username: string, password: string): Promise<LoginResponse> {
    const result = await this.request<LoginResponse>(
      "/api/v1/management/auth/login",
      {
        method: "POST",
        body: JSON.stringify({ username, password }),
      }
    );
    this.setToken(result.token);
    return result;
  }

  logout(): void {
    this.clearToken();
  }

  // Campaigns
  async listCampaigns(): Promise<Campaign[]> {
    return this.request<Campaign[]>("/api/v1/management/campaigns");
  }

  async getCampaign(id: string): Promise<Campaign> {
    return this.request<Campaign>(`/api/v1/management/campaigns/${id}`);
  }

  async createCampaign(data: CampaignCreatePayload): Promise<Campaign> {
    return this.request<Campaign>("/api/v1/management/campaigns", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateCampaign(
    id: string,
    data: CampaignUpdatePayload
  ): Promise<Campaign> {
    return this.request<Campaign>(`/api/v1/management/campaigns/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    });
  }

  async deleteCampaign(id: string): Promise<void> {
    return this.request<void>(`/api/v1/management/campaigns/${id}`, {
      method: "DELETE",
    });
  }

  async pauseCampaign(id: string): Promise<Campaign> {
    return this.request<Campaign>(
      `/api/v1/management/campaigns/${id}/pause`,
      {
        method: "POST",
      }
    );
  }

  async resumeCampaign(id: string): Promise<Campaign> {
    return this.request<Campaign>(
      `/api/v1/management/campaigns/${id}/resume`,
      {
        method: "POST",
      }
    );
  }

  // Creatives
  async listCreatives(): Promise<Creative[]> {
    return this.request<Creative[]>("/api/v1/management/creatives");
  }

  async getCreative(id: string): Promise<Creative> {
    return this.request<Creative>(`/api/v1/management/creatives/${id}`);
  }

  async createCreative(data: CreativeCreatePayload): Promise<Creative> {
    return this.request<Creative>("/api/v1/management/creatives", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async updateCreative(
    id: string,
    data: CreativeUpdatePayload
  ): Promise<Creative> {
    return this.request<Creative>(`/api/v1/management/creatives/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    });
  }

  async deleteCreative(id: string): Promise<void> {
    return this.request<void>(`/api/v1/management/creatives/${id}`, {
      method: "DELETE",
    });
  }

  // Monitoring
  async getMonitoringOverview(): Promise<MonitoringOverview> {
    return this.request<MonitoringOverview>(
      "/api/v1/management/monitoring/overview"
    );
  }

  async getCampaignStats(campaignId: string): Promise<CampaignDetailStats> {
    return this.request<CampaignDetailStats>(
      `/api/v1/management/monitoring/campaigns/${campaignId}/stats`
    );
  }

  // Models
  async reloadModel(): Promise<{ status: string }> {
    return this.request<{ status: string }>(
      "/api/v1/management/models/reload",
      {
        method: "POST",
      }
    );
  }

  // Audit Log
  async getAuditLog(): Promise<AuditLogEntry[]> {
    return this.request<AuditLogEntry[]>("/api/v1/management/audit-log");
  }

  // Journeys
  async listJourneys(): Promise<Journey[]> {
    return this.request<Journey[]>("/api/v1/management/journeys");
  }

  async getJourney(id: string): Promise<Journey> {
    return this.request<Journey>(`/api/v1/management/journeys/${id}`);
  }

  async createJourney(data: Partial<Journey>): Promise<Journey> {
    return this.request<Journey>("/api/v1/management/journeys", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async deleteJourney(id: string): Promise<void> {
    return this.request<void>(`/api/v1/management/journeys/${id}`, {
      method: "DELETE",
    });
  }

  async getJourneyStats(id: string): Promise<JourneyStats> {
    return this.request<JourneyStats>(`/api/v1/management/journeys/${id}/stats`);
  }

  // DCO Templates
  async listDcoTemplates(): Promise<DcoTemplate[]> {
    return this.request<DcoTemplate[]>("/api/v1/management/dco/templates");
  }

  async getDcoTemplate(id: string): Promise<DcoTemplate> {
    return this.request<DcoTemplate>(`/api/v1/management/dco/templates/${id}`);
  }

  async createDcoTemplate(data: Partial<DcoTemplate>): Promise<DcoTemplate> {
    return this.request<DcoTemplate>("/api/v1/management/dco/templates", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  async deleteDcoTemplate(id: string): Promise<void> {
    return this.request<void>(`/api/v1/management/dco/templates/${id}`, {
      method: "DELETE",
    });
  }

  // CDP
  async listCdpPlatforms(): Promise<CdpPlatformConfig[]> {
    return this.request<CdpPlatformConfig[]>("/api/v1/management/cdp/platforms");
  }

  async getCdpSyncHistory(): Promise<SyncEvent[]> {
    return this.request<SyncEvent[]>("/api/v1/management/cdp/sync-history");
  }

  // Experiments
  async listExperiments(): Promise<Experiment[]> {
    return this.request<Experiment[]>("/api/v1/management/experiments");
  }

  async getExperiment(id: string): Promise<Experiment> {
    return this.request<Experiment>(`/api/v1/management/experiments/${id}`);
  }

  async createExperiment(data: Partial<Experiment>): Promise<Experiment> {
    return this.request<Experiment>("/api/v1/management/experiments", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  // Platform — Tenants
  async listTenants(): Promise<Tenant[]> {
    return this.request<Tenant[]>("/api/v1/management/platform/tenants");
  }

  // Platform — Roles
  async listRoles(): Promise<Role[]> {
    return this.request<Role[]>("/api/v1/management/platform/roles");
  }

  // Platform — Compliance
  async getComplianceStatus(): Promise<ComplianceStatus[]> {
    return this.request<ComplianceStatus[]>("/api/v1/management/platform/compliance");
  }

  // Platform — Privacy (DSR)
  async listDsrs(): Promise<DataSubjectRequest[]> {
    return this.request<DataSubjectRequest[]>("/api/v1/management/platform/privacy/dsrs");
  }

  // Billing — Plans
  async listPlans(): Promise<PricingPlan[]> {
    return this.request<PricingPlan[]>("/api/v1/management/billing/plans");
  }

  // Billing — Subscriptions
  async getSubscription(tenantId: string): Promise<Subscription> {
    return this.request<Subscription>(`/api/v1/management/billing/subscriptions/${tenantId}`);
  }

  // Billing — Invoices
  async listInvoices(): Promise<Invoice[]> {
    return this.request<Invoice[]>("/api/v1/management/billing/invoices");
  }

  // Billing — Usage
  async getUsageSummary(tenantId: string): Promise<UsageSummary> {
    return this.request<UsageSummary>(`/api/v1/management/billing/usage/${tenantId}`);
  }

  // Billing — Onboarding
  async getOnboardingProgress(tenantId: string): Promise<OnboardingProgress> {
    return this.request<OnboardingProgress>(`/api/v1/management/billing/onboarding/${tenantId}`);
  }

  // Ops — Status Page
  async getStatusPage(): Promise<Record<string, unknown>> {
    return this.request<Record<string, unknown>>("/api/v1/management/ops/status");
  }

  // Ops — Incidents
  async listIncidents(): Promise<Incident[]> {
    return this.request<Incident[]>("/api/v1/management/ops/incidents");
  }

  // Ops — SLA
  async getSlaReport(): Promise<Record<string, unknown>> {
    return this.request<Record<string, unknown>>("/api/v1/management/ops/sla");
  }

  // Ops — Backups
  async listBackups(): Promise<BackupSchedule[]> {
    return this.request<BackupSchedule[]>("/api/v1/management/ops/backups");
  }
}

export const api = new ApiClient();
export { ApiClientError };
