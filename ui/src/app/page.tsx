"use client";

import { useQuery } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { useMemo } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Area,
  AreaChart,
} from "recharts";
import {
  Megaphone,
  Zap,
  Clock,
  Database,
  AlertCircle,
  CheckCircle2,
  Loader2,
} from "lucide-react";
import StatsCard from "@/components/stats-card";
import StatusBadge from "@/components/status-badge";
import { api } from "@/lib/api";
import type { Campaign, MonitoringOverview } from "@/lib/types";

// Seeded PRNG for deterministic chart data (avoids SSR/client hydration mismatch)
function seededRandom(seed: number): () => number {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

function generateThroughputData(overview: MonitoringOverview | undefined) {
  const hours = [];
  const rand = seededRandom(42);
  for (let i = 23; i >= 0; i--) {
    const hour = `${String(23 - i).padStart(2, "0")}:00`;
    const baseRate = overview ? overview.offers_per_hour / 24 : 2_000_000;
    const variance = 0.8 + rand() * 0.4;
    hours.push({
      time: hour,
      throughput: Math.round(baseRate * variance),
      latency: overview
        ? Math.round(overview.avg_latency_us * (0.7 + rand() * 0.6))
        : Math.round(800 + rand() * 400),
    });
  }
  return hours;
}

export default function DashboardPage() {
  const router = useRouter();

  const {
    data: overview,
    isLoading: overviewLoading,
    error: overviewError,
  } = useQuery({
    queryKey: ["monitoring-overview"],
    queryFn: () => api.getMonitoringOverview(),
    refetchInterval: 15_000,
  });

  const {
    data: campaigns,
    isLoading: campaignsLoading,
    error: campaignsError,
  } = useQuery({
    queryKey: ["campaigns"],
    queryFn: () => api.listCampaigns(),
  });

  const throughputData = useMemo(() => generateThroughputData(overview), [overview]);
  const recentCampaigns = campaigns?.slice(0, 5) ?? [];

  const isLoading = overviewLoading || campaignsLoading;
  const hasError = overviewError || campaignsError;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <div className="flex flex-col items-center gap-3">
          <Loader2 className="w-8 h-8 text-primary animate-spin" />
          <p className="text-sm text-gray-400">Loading dashboard...</p>
        </div>
      </div>
    );
  }

  if (hasError) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="w-8 h-8 text-red-400" />
          <p className="text-sm text-red-400">Failed to load dashboard data</p>
          <p className="text-xs text-gray-500">
            Make sure the API server is running at localhost:8080
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatsCard
          label="Active Campaigns"
          value={overview?.active_campaigns ?? 0}
          change={5.2}
          changeLabel="vs last week"
          icon={<Megaphone className="w-5 h-5" />}
        />
        <StatsCard
          label="Offers/Hour"
          value={overview?.offers_per_hour ?? 0}
          format="number"
          change={12.3}
          changeLabel="vs last hour"
          icon={<Zap className="w-5 h-5" />}
        />
        <StatsCard
          label="Avg Latency"
          value={overview?.avg_latency_us ?? 0}
          format="latency"
          change={-3.1}
          changeLabel="vs yesterday"
          icon={<Clock className="w-5 h-5" />}
        />
        <StatsCard
          label="Cache Hit Rate"
          value={(overview?.cache_hit_rate ?? 0) * 100}
          format="percent"
          change={1.5}
          changeLabel="vs yesterday"
          icon={<Database className="w-5 h-5" />}
        />
      </div>

      {/* Charts Row */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Throughput Chart */}
        <div className="bg-gray-800 border border-gray-700/50 rounded-xl p-5">
          <h3 className="text-sm font-semibold text-white mb-4">
            Throughput (offers/hour)
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={throughputData}>
                <defs>
                  <linearGradient
                    id="throughputGrad"
                    x1="0"
                    y1="0"
                    x2="0"
                    y2="1"
                  >
                    <stop offset="5%" stopColor="#2563eb" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="#2563eb" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  dataKey="time"
                  tick={{ fill: "#9ca3af", fontSize: 11 }}
                  axisLine={{ stroke: "#4b5563" }}
                  tickLine={false}
                />
                <YAxis
                  tick={{ fill: "#9ca3af", fontSize: 11 }}
                  axisLine={{ stroke: "#4b5563" }}
                  tickLine={false}
                  tickFormatter={(v) =>
                    v >= 1_000_000
                      ? `${(v / 1_000_000).toFixed(1)}M`
                      : `${(v / 1_000).toFixed(0)}K`
                  }
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1f2937",
                    border: "1px solid #374151",
                    borderRadius: "8px",
                    color: "#e5e7eb",
                    fontSize: "12px",
                  }}
                  formatter={(value: number) => [
                    value.toLocaleString(),
                    "Offers",
                  ]}
                />
                <Area
                  type="monotone"
                  dataKey="throughput"
                  stroke="#2563eb"
                  strokeWidth={2}
                  fill="url(#throughputGrad)"
                />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Latency Chart */}
        <div className="bg-gray-800 border border-gray-700/50 rounded-xl p-5">
          <h3 className="text-sm font-semibold text-white mb-4">
            Avg Latency (microseconds)
          </h3>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={throughputData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                <XAxis
                  dataKey="time"
                  tick={{ fill: "#9ca3af", fontSize: 11 }}
                  axisLine={{ stroke: "#4b5563" }}
                  tickLine={false}
                />
                <YAxis
                  tick={{ fill: "#9ca3af", fontSize: 11 }}
                  axisLine={{ stroke: "#4b5563" }}
                  tickLine={false}
                  tickFormatter={(v) => `${v}us`}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "#1f2937",
                    border: "1px solid #374151",
                    borderRadius: "8px",
                    color: "#e5e7eb",
                    fontSize: "12px",
                  }}
                  formatter={(value: number) => [`${value}us`, "Latency"]}
                />
                <Line
                  type="monotone"
                  dataKey="latency"
                  stroke="#10b981"
                  strokeWidth={2}
                  dot={false}
                />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>
      </div>

      {/* Bottom Row */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Recent Campaigns */}
        <div className="lg:col-span-2 bg-gray-800 border border-gray-700/50 rounded-xl">
          <div className="flex items-center justify-between px-5 py-4 border-b border-gray-700/50">
            <h3 className="text-sm font-semibold text-white">
              Recent Campaigns
            </h3>
            <button
              onClick={() => router.push("/campaigns")}
              className="text-xs text-primary-400 hover:text-primary-300 font-medium transition-colors"
            >
              View All
            </button>
          </div>
          <div className="divide-y divide-gray-700/50">
            {recentCampaigns.length === 0 ? (
              <div className="px-5 py-8 text-center text-sm text-gray-500">
                No campaigns yet
              </div>
            ) : (
              recentCampaigns.map((campaign: Campaign) => (
                <div
                  key={campaign.id}
                  role="button"
                  tabIndex={0}
                  onClick={() => router.push(`/campaigns/${campaign.id}`)}
                  onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") router.push(`/campaigns/${campaign.id}`); }}
                  className="flex items-center justify-between px-5 py-3 hover:bg-gray-700/30 cursor-pointer transition-colors"
                >
                  <div className="flex items-center gap-3 min-w-0">
                    <div className="min-w-0">
                      <p className="text-sm font-medium text-gray-200 truncate">
                        {campaign.name}
                      </p>
                      <p className="text-xs text-gray-500">
                        {campaign.stats.impressions.toLocaleString()}{" "}
                        impressions
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-3 shrink-0">
                    <span className="text-xs text-gray-400">
                      ${campaign.stats.spend.toLocaleString()}
                    </span>
                    <StatusBadge status={campaign.status} />
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* System Health */}
        <div className="bg-gray-800 border border-gray-700/50 rounded-xl">
          <div className="px-5 py-4 border-b border-gray-700/50">
            <h3 className="text-sm font-semibold text-white">System Health</h3>
          </div>
          <div className="p-5 space-y-4">
            <HealthItem
              label="Active Pods"
              value={`${overview?.active_pods ?? 0} / 20`}
              healthy={(overview?.active_pods ?? 0) >= 18}
            />
            <HealthItem
              label="Error Rate"
              value={`${((overview?.error_rate ?? 0) * 100).toFixed(2)}%`}
              healthy={(overview?.error_rate ?? 0) < 0.01}
            />
            <HealthItem
              label="No-Bid Rate"
              value={`${((overview?.no_bid_rate ?? 0) * 100).toFixed(1)}%`}
              healthy={(overview?.no_bid_rate ?? 0) < 0.3}
            />
            <HealthItem
              label="Cache Hit Rate"
              value={`${((overview?.cache_hit_rate ?? 0) * 100).toFixed(1)}%`}
              healthy={(overview?.cache_hit_rate ?? 0) > 0.85}
            />
            <HealthItem
              label="Total Campaigns"
              value={`${overview?.total_campaigns ?? 0}`}
              healthy={true}
            />
            <HealthItem
              label="Total Spend"
              value={`$${(overview?.total_spend ?? 0).toLocaleString()}`}
              healthy={true}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function HealthItem({
  label,
  value,
  healthy,
}: {
  label: string;
  value: string;
  healthy: boolean;
}) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-2">
        {healthy ? (
          <CheckCircle2 className="w-4 h-4 text-emerald-400" />
        ) : (
          <AlertCircle className="w-4 h-4 text-yellow-400" />
        )}
        <span className="text-sm text-gray-400">{label}</span>
      </div>
      <span className="text-sm font-medium text-gray-200">{value}</span>
    </div>
  );
}
