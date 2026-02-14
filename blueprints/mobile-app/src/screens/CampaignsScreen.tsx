/**
 * Campaigns Screen - Campaign Express Mobile Blueprint
 *
 * Lists campaigns with status badges and pause/resume actions.
 */

import React, { useEffect } from "react";
import {
  View,
  Text,
  FlatList,
  TouchableOpacity,
  StyleSheet,
  RefreshControl,
  Alert,
} from "react-native";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { mobileClient, Campaign } from "../services/api-client";
import { ceSdk } from "../services/ce-sdk";

const statusColors: Record<string, { bg: string; text: string }> = {
  active: { bg: "#dcfce7", text: "#166534" },
  paused: { bg: "#fef9c3", text: "#854d0e" },
  draft: { bg: "#f3f4f6", text: "#374151" },
  completed: { bg: "#dbeafe", text: "#1e40af" },
  error: { bg: "#fee2e2", text: "#991b1b" },
};

function CampaignCard({
  campaign,
  onPause,
  onResume,
}: {
  campaign: Campaign;
  onPause: () => void;
  onResume: () => void;
}) {
  const color = statusColors[campaign.status] || statusColors.draft;

  return (
    <View style={styles.card}>
      <View style={styles.cardHeader}>
        <Text style={styles.campaignName} numberOfLines={1}>
          {campaign.name}
        </Text>
        <View style={[styles.badge, { backgroundColor: color.bg }]}>
          <Text style={[styles.badgeText, { color: color.text }]}>
            {campaign.status}
          </Text>
        </View>
      </View>

      <View style={styles.statsRow}>
        <View style={styles.statItem}>
          <Text style={styles.statLabel}>Budget</Text>
          <Text style={styles.statValue}>${campaign.budget.toLocaleString()}</Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statLabel}>Spend</Text>
          <Text style={styles.statValue}>
            ${campaign.stats.spend.toLocaleString()}
          </Text>
        </View>
        <View style={styles.statItem}>
          <Text style={styles.statLabel}>CTR</Text>
          <Text style={styles.statValue}>
            {(campaign.stats.ctr * 100).toFixed(2)}%
          </Text>
        </View>
      </View>

      <View style={styles.actions}>
        {campaign.status === "active" && (
          <TouchableOpacity style={styles.actionBtn} onPress={onPause}>
            <Text style={styles.actionPause}>Pause</Text>
          </TouchableOpacity>
        )}
        {campaign.status === "paused" && (
          <TouchableOpacity style={styles.actionBtn} onPress={onResume}>
            <Text style={styles.actionResume}>Resume</Text>
          </TouchableOpacity>
        )}
      </View>
    </View>
  );
}

export function CampaignsScreen() {
  const queryClient = useQueryClient();
  const { data: campaigns, isLoading, refetch } = useQuery({
    queryKey: ["campaigns"],
    queryFn: () => mobileClient.listCampaigns(),
  });

  const pauseMutation = useMutation({
    mutationFn: (id: string) => mobileClient.pauseCampaign(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["campaigns"] }),
  });

  const resumeMutation = useMutation({
    mutationFn: (id: string) => mobileClient.resumeCampaign(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["campaigns"] }),
  });

  useEffect(() => {
    ceSdk.trackScreenView("Campaigns");
  }, []);

  const handlePause = (id: string) => {
    Alert.alert("Pause Campaign", "Are you sure?", [
      { text: "Cancel", style: "cancel" },
      {
        text: "Pause",
        onPress: () => {
          pauseMutation.mutate(id);
          ceSdk.trackEvent("campaign_paused", { campaign_id: id });
        },
      },
    ]);
  };

  const handleResume = (id: string) => {
    resumeMutation.mutate(id);
    ceSdk.trackEvent("campaign_resumed", { campaign_id: id });
  };

  return (
    <View style={styles.container}>
      <FlatList
        data={campaigns || []}
        keyExtractor={(item) => item.id}
        renderItem={({ item }) => (
          <CampaignCard
            campaign={item}
            onPause={() => handlePause(item.id)}
            onResume={() => handleResume(item.id)}
          />
        )}
        contentContainerStyle={styles.list}
        refreshControl={
          <RefreshControl refreshing={isLoading} onRefresh={refetch} />
        }
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyText}>No campaigns yet</Text>
            <Text style={styles.emptySubtext}>
              Create campaigns from the web dashboard.
            </Text>
          </View>
        }
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f9fafb" },
  list: { padding: 16 },
  card: {
    backgroundColor: "#fff",
    borderRadius: 12,
    padding: 16,
    marginBottom: 12,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 4,
    elevation: 2,
  },
  cardHeader: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: 12,
  },
  campaignName: { fontSize: 16, fontWeight: "600", color: "#111827", flex: 1 },
  badge: { borderRadius: 12, paddingHorizontal: 8, paddingVertical: 4, marginLeft: 8 },
  badgeText: { fontSize: 11, fontWeight: "600", textTransform: "uppercase" },
  statsRow: { flexDirection: "row", justifyContent: "space-between", marginBottom: 12 },
  statItem: { alignItems: "center" },
  statLabel: { fontSize: 11, color: "#6b7280" },
  statValue: { fontSize: 15, fontWeight: "600", color: "#111827", marginTop: 2 },
  actions: { flexDirection: "row", justifyContent: "flex-end", gap: 8 },
  actionBtn: { paddingHorizontal: 12, paddingVertical: 6 },
  actionPause: { color: "#ca8a04", fontWeight: "600", fontSize: 13 },
  actionResume: { color: "#16a34a", fontWeight: "600", fontSize: 13 },
  empty: { alignItems: "center", paddingVertical: 60 },
  emptyText: { fontSize: 18, color: "#6b7280", fontWeight: "500" },
  emptySubtext: { fontSize: 14, color: "#9ca3af", marginTop: 4 },
});
