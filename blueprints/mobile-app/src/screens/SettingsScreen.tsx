/**
 * Settings Screen - Campaign Express Mobile Blueprint
 *
 * Account settings, notification preferences, and logout.
 */

import React from "react";
import {
  View,
  Text,
  TouchableOpacity,
  StyleSheet,
  Switch,
  Alert,
  ScrollView,
} from "react-native";
import { mobileClient } from "../services/api-client";
import { ceSdk } from "../services/ce-sdk";

export function SettingsScreen({ navigation }: { navigation: any }) {
  const [pushEnabled, setPushEnabled] = React.useState(true);
  const [locationEnabled, setLocationEnabled] = React.useState(false);

  React.useEffect(() => {
    ceSdk.trackScreenView("Settings");
  }, []);

  const handleLogout = () => {
    Alert.alert("Sign Out", "Are you sure you want to sign out?", [
      { text: "Cancel", style: "cancel" },
      {
        text: "Sign Out",
        style: "destructive",
        onPress: async () => {
          ceSdk.trackEvent("logout");
          await mobileClient.logout();
          ceSdk.clearUserId();
          navigation.replace("Login");
        },
      },
    ]);
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      {/* Account Section */}
      <Text style={styles.sectionTitle}>Account</Text>
      <View style={styles.card}>
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Username</Text>
          <Text style={styles.rowValue}>demo-user-001</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Tier</Text>
          <Text style={styles.rowValue}>Professional</Text>
        </View>
      </View>

      {/* Notifications Section */}
      <Text style={styles.sectionTitle}>Notifications</Text>
      <View style={styles.card}>
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Push Notifications</Text>
          <Switch
            value={pushEnabled}
            onValueChange={(val) => {
              setPushEnabled(val);
              ceSdk.trackEvent("settings_push_toggle", { enabled: val });
            }}
            trackColor={{ false: "#d1d5db", true: "#93c5fd" }}
            thumbColor={pushEnabled ? "#2563eb" : "#f4f3f4"}
          />
        </View>
        <View style={styles.divider} />
        <View style={styles.row}>
          <Text style={styles.rowLabel}>Location Tracking</Text>
          <Switch
            value={locationEnabled}
            onValueChange={(val) => {
              setLocationEnabled(val);
              ceSdk.trackEvent("settings_location_toggle", { enabled: val });
            }}
            trackColor={{ false: "#d1d5db", true: "#93c5fd" }}
            thumbColor={locationEnabled ? "#2563eb" : "#f4f3f4"}
          />
        </View>
      </View>

      {/* SDK Info */}
      <Text style={styles.sectionTitle}>SDK Information</Text>
      <View style={styles.card}>
        <View style={styles.row}>
          <Text style={styles.rowLabel}>SDK Version</Text>
          <Text style={styles.rowValue}>0.1.0</Text>
        </View>
        <View style={styles.divider} />
        <View style={styles.row}>
          <Text style={styles.rowLabel}>API Endpoint</Text>
          <Text style={styles.rowValue}>campaignexpress.io</Text>
        </View>
      </View>

      {/* Logout */}
      <TouchableOpacity style={styles.logoutBtn} onPress={handleLogout}>
        <Text style={styles.logoutText}>Sign Out</Text>
      </TouchableOpacity>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: "#f9fafb" },
  content: { padding: 16, paddingBottom: 40 },
  sectionTitle: {
    fontSize: 13,
    fontWeight: "600",
    color: "#6b7280",
    textTransform: "uppercase",
    marginBottom: 8,
    marginTop: 20,
    marginLeft: 4,
  },
  card: {
    backgroundColor: "#fff",
    borderRadius: 12,
    overflow: "hidden",
    shadowColor: "#000",
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.05,
    shadowRadius: 4,
    elevation: 2,
  },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "center",
    paddingHorizontal: 16,
    paddingVertical: 14,
  },
  rowLabel: { fontSize: 15, color: "#111827" },
  rowValue: { fontSize: 15, color: "#6b7280" },
  divider: { height: 1, backgroundColor: "#f3f4f6", marginLeft: 16 },
  logoutBtn: {
    marginTop: 32,
    backgroundColor: "#fee2e2",
    borderRadius: 12,
    paddingVertical: 14,
    alignItems: "center",
  },
  logoutText: { fontSize: 16, fontWeight: "600", color: "#dc2626" },
});
