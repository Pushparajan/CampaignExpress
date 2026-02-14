/**
 * Loyalty Screen - Campaign Express Mobile Blueprint
 *
 * Shows loyalty tier, star balance, progress, and earn/redeem actions.
 */

import React, { useEffect, useState } from "react";
import {
  View,
  Text,
  ScrollView,
  TouchableOpacity,
  StyleSheet,
  Alert,
} from "react-native";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { mobileClient } from "../services/api-client";
import { ceSdk } from "../services/ce-sdk";

const tierStyles: Record<string, { bg: string; accent: string; text: string }> = {
  green: { bg: "#f0fdf4", accent: "#22c55e", text: "#166534" },
  gold: { bg: "#fefce8", accent: "#eab308", text: "#854d0e" },
  reserve: { bg: "#faf5ff", accent: "#a855f7", text: "#6b21a8" },
};

export function LoyaltyScreen() {
  const userId = "demo-user-001";
  const queryClient = useQueryClient();

  const { data: balance, isLoading } = useQuery({
    queryKey: ["loyalty", userId],
    queryFn: () => mobileClient.getLoyaltyBalance(userId),
  });

  const earnMutation = useMutation({
    mutationFn: () => mobileClient.earnStars(userId, 1500, "coffee"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["loyalty", userId] });
      ceSdk.trackPurchase(`order-${Date.now()}`, 15.0, "USD");
    },
  });

  const redeemMutation = useMutation({
    mutationFn: () => mobileClient.redeemStars(userId, 100, "free-drink"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["loyalty", userId] });
      ceSdk.trackEvent("loyalty_redeem", { stars: 100, reward: "free-drink" });
    },
  });

  useEffect(() => {
    ceSdk.trackScreenView("Loyalty");
  }, []);

  const tier = balance ? tierStyles[balance.tier] || tierStyles.green : tierStyles.green;

  const handleRedeem = () => {
    if (!balance || balance.stars_balance < 100) {
      Alert.alert("Not Enough Stars", "You need at least 100 stars to redeem.");
      return;
    }
    Alert.alert("Redeem Stars", "Redeem 100 stars for a free drink?", [
      { text: "Cancel", style: "cancel" },
      { text: "Redeem", onPress: () => redeemMutation.mutate() },
    ]);
  };

  if (isLoading) {
    return (
      <View style={styles.loading}>
        <Text style={styles.loadingText}>Loading loyalty info...</Text>
      </View>
    );
  }

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      {/* Tier Card */}
      <View style={[styles.tierCard, { backgroundColor: tier.bg }]}>
        <View style={styles.tierHeader}>
          <View>
            <Text style={styles.tierLabel}>Your Tier</Text>
            <Text style={[styles.tierName, { color: tier.text }]}>
              {balance?.tier?.toUpperCase() || "GREEN"}
            </Text>
          </View>
          <View style={[styles.tierIcon, { backgroundColor: tier.accent }]}>
            <Text style={styles.tierStar}>&#9733;</Text>
          </View>
        </View>

        <View style={styles.balanceRow}>
          <View>
            <Text style={styles.balanceLabel}>Stars Balance</Text>
            <Text style={styles.balanceValue}>
              {balance?.stars_balance?.toLocaleString() || "0"}
            </Text>
          </View>
          <View>
            <Text style={styles.balanceLabel}>Lifetime</Text>
            <Text style={styles.balanceValue}>
              {balance?.lifetime_stars?.toLocaleString() || "0"}
            </Text>
          </View>
        </View>

        {/* Progress Bar */}
        <View style={styles.progressContainer}>
          <View style={styles.progressHeader}>
            <Text style={styles.progressLabel}>Next Tier Progress</Text>
            <Text style={styles.progressPercent}>
              {Math.round((balance?.next_tier_progress || 0) * 100)}%
            </Text>
          </View>
          <View style={styles.progressBar}>
            <View
              style={[
                styles.progressFill,
                {
                  backgroundColor: tier.accent,
                  width: `${Math.min((balance?.next_tier_progress || 0) * 100, 100)}%`,
                },
              ]}
            />
          </View>
        </View>
      </View>

      {/* Action Buttons */}
      <View style={styles.actionsRow}>
        <TouchableOpacity
          style={styles.earnBtn}
          onPress={() => earnMutation.mutate()}
          disabled={earnMutation.isPending}
        >
          <Text style={styles.earnText}>
            {earnMutation.isPending ? "Earning..." : "Earn Stars (+$15)"}
          </Text>
        </TouchableOpacity>
        <TouchableOpacity
          style={[styles.redeemBtn, { backgroundColor: tier.accent }]}
          onPress={handleRedeem}
          disabled={redeemMutation.isPending}
        >
          <Text style={styles.redeemText}>
            {redeemMutation.isPending ? "Redeeming..." : "Redeem 100 Stars"}
          </Text>
        </TouchableOpacity>
      </View>

      {/* Tier Info */}
      <View style={styles.infoCard}>
        <Text style={styles.infoTitle}>Tier Benefits</Text>
        <View style={styles.tierRow}>
          <View style={[styles.tierDot, { backgroundColor: "#22c55e" }]} />
          <Text style={styles.tierInfo}>Green: 1x earn rate</Text>
        </View>
        <View style={styles.tierRow}>
          <View style={[styles.tierDot, { backgroundColor: "#eab308" }]} />
          <Text style={styles.tierInfo}>
            Gold: 1.2x earn rate (500 stars/year)
          </Text>
        </View>
        <View style={styles.tierRow}>
          <View style={[styles.tierDot, { backgroundColor: "#a855f7" }]} />
          <Text style={styles.tierInfo}>
            Reserve: 1.7x earn rate (2500 stars/year)
          </Text>
        </View>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f9fafb" },
  content: { padding: 16 },
  loading: { flex: 1, justifyContent: "center", alignItems: "center" },
  loadingText: { color: "#6b7280", fontSize: 16 },
  tierCard: {
    borderRadius: 16,
    padding: 20,
    marginBottom: 16,
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.08,
    shadowRadius: 8,
    elevation: 3,
  },
  tierHeader: { flexDirection: "row", justifyContent: "space-between", alignItems: "center" },
  tierLabel: { fontSize: 12, color: "#6b7280" },
  tierName: { fontSize: 28, fontWeight: "bold", marginTop: 2 },
  tierIcon: { width: 48, height: 48, borderRadius: 24, justifyContent: "center", alignItems: "center" },
  tierStar: { fontSize: 24, color: "#fff" },
  balanceRow: { flexDirection: "row", justifyContent: "space-between", marginTop: 20 },
  balanceLabel: { fontSize: 12, color: "#6b7280" },
  balanceValue: { fontSize: 22, fontWeight: "bold", color: "#111827", marginTop: 2 },
  progressContainer: { marginTop: 16 },
  progressHeader: { flexDirection: "row", justifyContent: "space-between" },
  progressLabel: { fontSize: 11, color: "#6b7280" },
  progressPercent: { fontSize: 11, color: "#6b7280", fontWeight: "600" },
  progressBar: { height: 8, backgroundColor: "#fff", borderRadius: 4, marginTop: 4 },
  progressFill: { height: 8, borderRadius: 4 },
  actionsRow: { flexDirection: "row", gap: 12, marginBottom: 16 },
  earnBtn: {
    flex: 1, backgroundColor: "#fff", borderRadius: 12, paddingVertical: 14,
    alignItems: "center", borderWidth: 1, borderColor: "#d1d5db",
  },
  earnText: { fontSize: 14, fontWeight: "600", color: "#374151" },
  redeemBtn: { flex: 1, borderRadius: 12, paddingVertical: 14, alignItems: "center" },
  redeemText: { fontSize: 14, fontWeight: "600", color: "#fff" },
  infoCard: { backgroundColor: "#fff", borderRadius: 12, padding: 16 },
  infoTitle: { fontSize: 16, fontWeight: "600", color: "#111827", marginBottom: 12 },
  tierRow: { flexDirection: "row", alignItems: "center", marginBottom: 8 },
  tierDot: { width: 10, height: 10, borderRadius: 5, marginRight: 8 },
  tierInfo: { fontSize: 14, color: "#4b5563" },
});
