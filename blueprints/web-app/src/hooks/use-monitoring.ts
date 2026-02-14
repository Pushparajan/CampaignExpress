/**
 * React hooks for Campaign Express real-time monitoring dashboard.
 */

"use client";

import { useQuery } from "@tanstack/react-query";
import { apiClient } from "@/lib/api-client";

export function useMonitoringOverview() {
  return useQuery({
    queryKey: ["monitoring", "overview"],
    queryFn: () => apiClient.getOverview(),
    refetchInterval: 15_000, // Real-time: refresh every 15 seconds
    staleTime: 10_000,
  });
}
