"use client";

import { useQuery } from "@tanstack/react-query";
import {
  Database,
  Plus,
  RefreshCw,
  CheckCircle2,
  XCircle,
  Loader2,
  AlertCircle,
} from "lucide-react";
import StatusBadge from "@/components/status-badge";
import { api } from "@/lib/api";
import type { CdpPlatformConfig, SyncEvent } from "@/lib/types";
import { formatDateTime } from "@/lib/format-date";

function formatPlatformName(platform: string): string {
  return platform
    .replace(/_/g, " ")
    .replace(/\b\w/g, (c) => c.toUpperCase())
    .replace("Cdp", "CDP");
}

export default function CdpPage() {
  const { data: platforms, isLoading: platformsLoading } = useQuery({
    queryKey: ["cdp-platforms"],
    queryFn: () => api.listCdpPlatforms(),
  });

  const { data: syncHistory, isLoading: syncLoading } = useQuery({
    queryKey: ["cdp-sync-history"],
    queryFn: () => api.getCdpSyncHistory(),
  });

  const isLoading = platformsLoading || syncLoading;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <Loader2 className="w-8 h-8 text-primary animate-spin" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">CDP Integrations</h1>
        <button
          onClick={() => alert("Platform configuration coming soon!")}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" /> Add Platform
        </button>
      </div>

      {/* Connected Platforms */}
      <div>
        <h2 className="text-sm font-semibold text-white mb-3">Connected Platforms</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {platforms?.map((platform: CdpPlatformConfig) => (
            <div
              key={platform.platform}
              className="bg-gray-800 border border-gray-700/50 rounded-xl p-5 hover:border-gray-600 transition-colors"
            >
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <Database className="w-4 h-4 text-blue-400" />
                  <h3 className="text-sm font-semibold text-white">
                    {formatPlatformName(platform.platform)}
                  </h3>
                </div>
                <div className="flex items-center gap-1.5">
                  <div
                    className={`w-2 h-2 rounded-full ${
                      platform.enabled ? "bg-emerald-500" : "bg-red-500"
                    }`}
                  />
                  <span className="text-xs text-gray-400">
                    {platform.enabled ? "Active" : "Disabled"}
                  </span>
                </div>
              </div>
              <div className="space-y-2 text-xs text-gray-400">
                <p className="truncate">Endpoint: {platform.api_endpoint}</p>
                <div className="flex justify-between">
                  <span>Sync: {platform.sync_interval_secs}s</span>
                  <span>Batch: {platform.batch_size?.toLocaleString()}</span>
                </div>
                <p>{Object.keys(platform.field_mappings || {}).length} field mappings</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Sync History */}
      <div>
        <h2 className="text-sm font-semibold text-white mb-3">Sync History</h2>
        <div className="bg-gray-800 border border-gray-700/50 rounded-xl overflow-hidden">
          <table className="w-full">
            <thead>
              <tr className="border-b border-gray-700/50">
                <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Platform</th>
                <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Direction</th>
                <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Records</th>
                <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Status</th>
                <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Started</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700/50">
              {syncHistory?.map((sync: SyncEvent) => (
                <tr key={sync.id} className="hover:bg-gray-700/30 transition-colors">
                  <td className="px-5 py-3 text-sm text-gray-300">
                    {formatPlatformName(sync.platform)}
                  </td>
                  <td className="px-5 py-3">
                    <span className="inline-flex items-center gap-1 text-xs">
                      <RefreshCw className="w-3 h-3 text-gray-400" />
                      <span className="text-gray-300 capitalize">{sync.direction}</span>
                    </span>
                  </td>
                  <td className="px-5 py-3 text-sm text-gray-300">
                    {sync.record_count?.toLocaleString()}
                  </td>
                  <td className="px-5 py-3">
                    <span
                      className={`inline-flex items-center gap-1 text-xs font-medium ${
                        sync.status === "completed"
                          ? "text-emerald-400"
                          : sync.status === "failed"
                          ? "text-red-400"
                          : "text-yellow-400"
                      }`}
                    >
                      {sync.status === "completed" ? (
                        <CheckCircle2 className="w-3 h-3" />
                      ) : sync.status === "failed" ? (
                        <XCircle className="w-3 h-3" />
                      ) : (
                        <AlertCircle className="w-3 h-3" />
                      )}
                      {sync.status?.replace(/_/g, " ")}
                    </span>
                  </td>
                  <td className="px-5 py-3 text-sm text-gray-400">
                    {formatDateTime(sync.started_at)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
