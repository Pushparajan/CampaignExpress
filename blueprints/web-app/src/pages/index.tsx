/**
 * Dashboard Page - Campaign Express Web Blueprint
 *
 * Shows real-time platform metrics, campaign overview, and performance charts.
 * Demonstrates integration with the monitoring overview API.
 */

"use client";

import { useEffect } from "react";
import { useMonitoringOverview } from "@/hooks/use-monitoring";
import { useCampaigns } from "@/hooks/use-campaigns";
import { StatsCard } from "@/components/stats-card";
import { CampaignTable } from "@/components/campaign-table";
import { tracker } from "@/lib/ce-tracker";

export default function DashboardPage() {
  const { data: overview, isLoading: overviewLoading } = useMonitoringOverview();
  const { data: campaigns, isLoading: campaignsLoading } = useCampaigns();

  useEffect(() => {
    tracker.trackPageView("/", "Dashboard");
  }, []);

  return (
    <div className="mx-auto max-w-7xl px-4 py-8">
      <h1 className="mb-8 text-3xl font-bold text-gray-900">Dashboard</h1>

      {/* KPI Cards */}
      <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatsCard
          title="Active Campaigns"
          value={overviewLoading ? "..." : overview?.active_campaigns ?? 0}
          subtitle="Currently running"
        />
        <StatsCard
          title="Offers / Hour"
          value={
            overviewLoading
              ? "..."
              : `${((overview?.offers_per_hour ?? 0) / 1_000_000).toFixed(1)}M`
          }
          subtitle="Real-time throughput"
        />
        <StatsCard
          title="Avg Latency"
          value={
            overviewLoading
              ? "..."
              : `${((overview?.avg_latency_us ?? 0) / 1000).toFixed(1)}ms`
          }
          subtitle="End-to-end"
        />
        <StatsCard
          title="Cache Hit Rate"
          value={
            overviewLoading
              ? "..."
              : `${((overview?.cache_hit_rate ?? 0) * 100).toFixed(1)}%`
          }
          subtitle="L1 + L2 combined"
        />
      </div>

      {/* Spend and CTR row */}
      <div className="mb-8 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <StatsCard
          title="Total Spend"
          value={
            overviewLoading
              ? "..."
              : `$${(overview?.total_spend ?? 0).toLocaleString()}`
          }
        />
        <StatsCard
          title="Avg CTR"
          value={
            overviewLoading
              ? "..."
              : `${((overview?.avg_ctr ?? 0) * 100).toFixed(2)}%`
          }
        />
        <StatsCard
          title="Error Rate"
          value={
            overviewLoading
              ? "..."
              : `${((overview?.error_rate ?? 0) * 100).toFixed(3)}%`
          }
        />
      </div>

      {/* Recent Campaigns */}
      <div>
        <h2 className="mb-4 text-xl font-semibold text-gray-800">
          Recent Campaigns
        </h2>
        {campaignsLoading ? (
          <p className="text-gray-500">Loading campaigns...</p>
        ) : campaigns && campaigns.length > 0 ? (
          <CampaignTable campaigns={campaigns.slice(0, 10)} />
        ) : (
          <p className="text-gray-500">
            No campaigns yet. Create your first campaign to get started.
          </p>
        )}
      </div>
    </div>
  );
}
