/**
 * Campaign Express Mobile SDK
 *
 * Handles event tracking, session management, push token registration,
 * and in-app messaging for mobile applications.
 *
 * Usage:
 *   import { ceSdk } from "@/services/ce-sdk";
 *   await ceSdk.initialize();
 *   ceSdk.trackScreenView("HomeScreen");
 *   ceSdk.trackPurchase("order-123", 29.99, "USD");
 */

import * as Device from "expo-device";
import * as Notifications from "expo-notifications";
import { mobileClient } from "./api-client";

interface SdkConfig {
  apiKey: string;
  flushIntervalMs: number;
  sessionTimeoutMs: number;
  maxBatchSize: number;
}

interface TrackEvent {
  event_type: string;
  name?: string;
  properties: Record<string, unknown>;
  timestamp: string;
}

const DEFAULT_CONFIG: SdkConfig = {
  apiKey: process.env.CE_API_KEY || "",
  flushIntervalMs: 30_000,
  sessionTimeoutMs: 300_000, // 5 minutes
  maxBatchSize: 25,
};

class CampaignExpressSdk {
  private config: SdkConfig;
  private queue: TrackEvent[] = [];
  private sessionId: string | null = null;
  private userId: string | null = null;
  private deviceId: string;
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private lastActivity: number = Date.now();

  constructor(config: Partial<SdkConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.deviceId = `device-${Date.now()}`;
  }

  /** Initialize the SDK — call once at app startup. */
  async initialize(): Promise<void> {
    // Get device info
    if (Device.isDevice) {
      this.deviceId = Device.modelId || Device.deviceName || this.deviceId;
    }

    // Start session
    this.startSession();

    // Start periodic flush
    this.flushTimer = setInterval(() => this.flush(), this.config.flushIntervalMs);

    // Register for push notifications
    await this.registerForPush();
  }

  /** Shut down the SDK — call on app termination. */
  async shutdown(): Promise<void> {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
    this.endSession();
    await this.flush();
  }

  /** Set the authenticated user ID (links device to user). */
  setUserId(userId: string): void {
    this.userId = userId;
    this.enqueue("identify", "Identify", { user_id: userId });
  }

  /** Clear user ID on logout. */
  clearUserId(): void {
    this.userId = null;
  }

  // --- Tracking Methods ---

  trackScreenView(screenName: string, properties: Record<string, unknown> = {}): void {
    this.touchActivity();
    this.enqueue("screen_view", screenName, {
      screen_name: screenName,
      ...properties,
    });
  }

  trackPurchase(orderId: string, amount: number, currency: string, items?: unknown[]): void {
    this.touchActivity();
    this.enqueue("purchase", "Purchase", {
      order_id: orderId,
      amount,
      currency,
      items: items || [],
    });
  }

  trackEvent(name: string, properties: Record<string, unknown> = {}): void {
    this.touchActivity();
    this.enqueue("custom_event", name, properties);
  }

  trackAppOpen(): void {
    this.touchActivity();
    this.enqueue("app_open", "App Open", {
      device_model: Device.modelName,
      os_version: Device.osVersion,
    });
  }

  trackCartAdd(productId: string, productName: string, price: number): void {
    this.touchActivity();
    this.enqueue("cart_add", "Cart Add", {
      product_id: productId,
      product_name: productName,
      price,
    });
  }

  trackSearch(query: string, resultsCount: number): void {
    this.touchActivity();
    this.enqueue("search", "Search", {
      query,
      results_count: resultsCount,
    });
  }

  // --- Session Management ---

  private startSession(): void {
    this.sessionId = `session-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
    this.lastActivity = Date.now();
    this.enqueue("session_start", "Session Start", {
      session_id: this.sessionId,
    });
  }

  private endSession(): void {
    if (this.sessionId) {
      this.enqueue("session_end", "Session End", {
        session_id: this.sessionId,
        duration_ms: Date.now() - this.lastActivity,
      });
      this.sessionId = null;
    }
  }

  private touchActivity(): void {
    const now = Date.now();
    if (now - this.lastActivity > this.config.sessionTimeoutMs) {
      this.endSession();
      this.startSession();
    }
    this.lastActivity = now;
  }

  // --- Push Notifications ---

  private async registerForPush(): Promise<void> {
    try {
      const { status } = await Notifications.getPermissionsAsync();
      if (status !== "granted") {
        const { status: newStatus } = await Notifications.requestPermissionsAsync();
        if (newStatus !== "granted") return;
      }

      const token = await Notifications.getExpoPushTokenAsync();
      const platform = Device.osName?.toLowerCase() === "ios" ? "ios" : "android";

      await mobileClient.registerPushToken(
        this.deviceId,
        token.data,
        platform as "ios" | "android"
      );
    } catch {
      // Push registration is best-effort
    }
  }

  // --- Event Queue ---

  private enqueue(eventType: string, name: string, properties: Record<string, unknown>): void {
    this.queue.push({
      event_type: eventType,
      name,
      properties: {
        ...properties,
        device_id: this.deviceId,
        user_id: this.userId,
        session_id: this.sessionId,
      },
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

    try {
      await mobileClient.ingestEvent({
        source: "mobile_app",
        event_type: "batch",
        user_id: this.userId || undefined,
        payload: {
          api_key: this.config.apiKey,
          device_id: this.deviceId,
          session_id: this.sessionId,
          events,
          sent_at: new Date().toISOString(),
        },
      });
    } catch {
      // Re-queue on failure
      this.queue.unshift(...events);
    }
  }
}

export const ceSdk = new CampaignExpressSdk();
