/**
 * Dashboard Screen - Campaign Express Mobile Blueprint
 *
 * Real-time platform metrics with auto-refreshing stats cards.
 */

import React, { useEffect } from "react";
import {
  View,
  Text,
  ScrollView,
  StyleSheet,
  RefreshControl,
} from "react-native";
import { useQuery } from "@tanstack/react-query";
import { mobileClient } from "../services/api-client";
import { ceSdk } from "../services/ce-sdk";

function StatCard({
  title,
  value,
  subtitle,
}: {
  title: string;
  value: string;
  subtitle?: string;
}) {
  return (
    <View style={styles.statCard}>
      <Text style={styles.statTitle}>{title}</Text>
      <Text style={styles.statValue}>{value}</Text>
      {subtitle && <Text style={styles.statSubtitle}>{subtitle}</Text>}
    </View>
  );
}

export function DashboardScreen() {
  const {
    data: overview,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["overview"],
    queryFn: () => mobileClient.getOverview(),
    refetchInterval: 15_000,
  });

  useEffect(() => {
    ceSdk.trackScreenView("Dashboard");
  }, []);

  return (
    <ScrollView
      style={styles.container}
      contentContainerStyle={styles.content}
      refreshControl={
        <RefreshControl refreshing={isLoading} onRefresh={refetch} />
      }
    >
      <Text style={styles.heading}>Platform Overview</Text>

      <View style={styles.statsGrid}>
        <StatCard
          title="Active Campaigns"
          value={overview ? String(overview.active_campaigns) : "..."}
          subtitle="Currently running"
        />
        <StatCard
          title="Offers / Hour"
          value={
            overview
              ? `${(overview.offers_per_hour / 1_000_000).toFixed(1)}M`
              : "..."
          }
          subtitle="Throughput"
        />
        <StatCard
          title="Avg Latency"
          value={
            overview
              ? `${(overview.avg_latency_us / 1000).toFixed(1)}ms`
              : "..."
          }
          subtitle="End-to-end"
        />
        <StatCard
          title="Cache Hit Rate"
          value={
            overview
              ? `${(overview.cache_hit_rate * 100).toFixed(1)}%`
              : "..."
          }
          subtitle="L1 + L2"
        />
      </View>

      <Text style={styles.sectionTitle}>Performance</Text>
      <View style={styles.statsGrid}>
        <StatCard
          title="Total Spend"
          value={
            overview
              ? `$${overview.total_spend.toLocaleString()}`
              : "..."
          }
        />
        <StatCard
          title="Impressions"
          value={
            overview
              ? overview.total_impressions.toLocaleString()
              : "..."
          }
        />
        <StatCard
          title="Clicks"
          value={
            overview
              ? overview.total_clicks.toLocaleString()
              : "..."
          }
        />
        <StatCard
          title="Avg CTR"
          value={
            overview
              ? `${(overview.avg_ctr * 100).toFixed(2)}%`
              : "..."
          }
        />
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f9fafb" },
  content: { padding: 16 },
  heading: {
    fontSize: 22,
    fontWeight: "bold",
    color: "#111827",
    marginBottom: 16,
  },
  sectionTitle: {
    fontSize: 18,
    fontWeight: "600",
    color: "#374151",
    marginTop: 24,
    marginBottom: 12,
  },
  statsGrid: {
    flexDirection: "row",
    flexWrap: "wrap",
    gap: 12,
  },
  statCard: {
    backgroundColor: "#fff",
    borderRadius: 12,
    padding: 16,
    width: "47%",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 4,
    elevation: 2,
  },
  statTitle: { fontSize: 12, color: "#6b7280", fontWeight: "500" },
  statValue: { fontSize: 24, fontWeight: "bold", color: "#111827", marginTop: 4 },
  statSubtitle: { fontSize: 11, color: "#9ca3af", marginTop: 2 },
});
