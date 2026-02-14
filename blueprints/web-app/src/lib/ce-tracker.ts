/**
 * Campaign Express Web SDK Tracker
 *
 * Lightweight client-side event tracker that batches and sends
 * user behaviour events to Campaign Express for personalization.
 *
 * Usage:
 *   import { tracker } from "@/lib/ce-tracker";
 *   tracker.trackPageView("/products", "Products Page");
 *   tracker.trackClick("buy-btn", "Buy Now");
 *   tracker.trackPurchase("order-123", 49.99, "USD");
 */

interface TrackerConfig {
  apiKey: string;
  endpoint: string;
  flushIntervalMs: number;
  maxBatchSize: number;
}

interface TrackEvent {
  event_type: string;
  name?: string;
  properties: Record<string, unknown>;
  timestamp: string;
}

const DEFAULT_CONFIG: TrackerConfig = {
  apiKey: process.env.NEXT_PUBLIC_CE_API_KEY || "",
  endpoint: process.env.NEXT_PUBLIC_API_URL || "",
  flushIntervalMs: 10_000,
  maxBatchSize: 50,
};

class CampaignExpressTracker {
  private config: TrackerConfig;
  private queue: TrackEvent[] = [];
  private sessionId: string;
  private anonymousId: string;
  private flushTimer: ReturnType<typeof setInterval> | null = null;

  constructor(config: Partial<TrackerConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.sessionId = this.generateId();
    this.anonymousId = this.getOrCreateAnonymousId();
  }

  /** Initialize automatic page view tracking and periodic flushing. */
  init(): void {
    if (typeof window === "undefined") return;

    this.startFlushTimer();

    // Flush on page unload
    window.addEventListener("beforeunload", () => this.flush());
  }

  /** Stop the tracker and flush remaining events. */
  destroy(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    this.flush();
  }

  // --- Tracking Methods ---

  trackPageView(url: string, title: string, referrer?: string): void {
    this.enqueue("page_view", "Page View", {
      url,
      title,
      referrer: referrer || (typeof document !== "undefined" ? document.referrer : ""),
    });
  }

  trackClick(elementId: string, elementText: string, href?: string): void {
    this.enqueue("click", "Click", {
      element_id: elementId,
      element_text: elementText,
      href,
    });
  }

  trackFormSubmit(formId: string, formName: string, fieldCount: number): void {
    this.enqueue("form_submit", "Form Submit", {
      form_id: formId,
      form_name: formName,
      field_count: fieldCount,
    });
  }

  trackPurchase(
    orderId: string,
    amount: number,
    currency: string,
    items?: Array<{ sku: string; name: string; qty: number; price: number }>
  ): void {
    this.enqueue("purchase", "Purchase", {
      order_id: orderId,
      amount,
      currency,
      items: items || [],
    });
  }

  trackSearch(query: string, resultsCount: number): void {
    this.enqueue("site_search", "Search", {
      query,
      results_count: resultsCount,
    });
  }

  trackCustomEvent(name: string, properties: Record<string, unknown> = {}): void {
    this.enqueue("custom_event", name, properties);
  }

  /** Identify a logged-in user (links anonymous_id to user_id). */
  identify(userId: string, traits: Record<string, unknown> = {}): void {
    this.enqueue("identify", "Identify", { user_id: userId, ...traits });
  }

  // --- Internal ---

  private enqueue(eventType: string, name: string, properties: Record<string, unknown>): void {
    this.queue.push({
      event_type: eventType,
      name,
      properties,
      timestamp: new Date().toISOString(),
    });

    if (this.queue.length >= this.config.maxBatchSize) {
      this.flush();
    }
  }

  async flush(): Promise<void> {
    if (this.queue.length === 0) return;

    const events = [...this.queue];
    this.queue = [];

    const batch = {
      api_key: this.config.apiKey,
      anonymous_id: this.anonymousId,
      session_id: this.sessionId,
      events,
      sent_at: new Date().toISOString(),
    };

    try {
      await fetch(`${this.config.endpoint}/v1/channels/ingest`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(batch),
        keepalive: true,
      });
    } catch {
      // Re-queue on failure for retry
      this.queue.unshift(...events);
    }
  }

  private startFlushTimer(): void {
    this.flushTimer = setInterval(() => this.flush(), this.config.flushIntervalMs);
  }

  private getOrCreateAnonymousId(): string {
    if (typeof window === "undefined") return this.generateId();
    const key = "_ce_anon_id";
    let id = localStorage.getItem(key);
    if (!id) {
      id = this.generateId();
      localStorage.setItem(key, id);
    }
    return id;
  }

  private generateId(): string {
    // Use crypto.randomUUID if available, else fallback
    if (typeof crypto !== "undefined" && crypto.randomUUID) {
      return crypto.randomUUID();
    }
    return `${Date.now()}-${Math.random().toString(36).slice(2, 11)}`;
  }
}

export const tracker = new CampaignExpressTracker();
export { CampaignExpressTracker };
