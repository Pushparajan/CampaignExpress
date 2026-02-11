"use client";

import { useQuery } from "@tanstack/react-query";
import {
  GitBranch,
  Plus,
  Users,
  CheckCircle2,
  Clock,
  Loader2,
  AlertCircle,
} from "lucide-react";
import StatusBadge from "@/components/status-badge";
import { api } from "@/lib/api";
import type { Journey } from "@/lib/types";

export default function JourneysPage() {
  const { data: journeys, isLoading, error } = useQuery({
    queryKey: ["journeys"],
    queryFn: () => api.listJourneys(),
  });

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <Loader2 className="w-8 h-8 text-primary animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-[60vh]">
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="w-8 h-8 text-red-400" />
          <p className="text-sm text-red-400">Failed to load journeys</p>
        </div>
      </div>
    );
  }

  const active = journeys?.filter((j: Journey) => j.status === "active").length ?? 0;
  const completed = journeys?.filter((j: Journey) => j.status === "completed").length ?? 0;
  const total = journeys?.length ?? 0;

  function formatTrigger(trigger: Journey["trigger"]): string {
    if (!trigger?.type) return "Unknown";
    return trigger.type.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">Journey Orchestration</h1>
        <button
          onClick={() => alert("Journey builder coming soon!")}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" /> Create Journey
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <StatCard icon={<GitBranch className="w-5 h-5 text-blue-400" />} label="Total Journeys" value={total} />
        <StatCard icon={<Users className="w-5 h-5 text-emerald-400" />} label="Active" value={active} />
        <StatCard icon={<CheckCircle2 className="w-5 h-5 text-purple-400" />} label="Completed" value={completed} />
        <StatCard icon={<Clock className="w-5 h-5 text-yellow-400" />} label="Avg Completion" value="24h" />
      </div>

      <div className="bg-gray-800 border border-gray-700/50 rounded-xl overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-700/50">
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Name</th>
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Status</th>
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Trigger</th>
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Steps</th>
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Version</th>
              <th className="px-5 py-3 text-left text-xs text-gray-500 uppercase font-medium">Created</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700/50">
            {journeys?.map((journey: Journey) => (
              <tr key={journey.id} className="hover:bg-gray-700/30 cursor-pointer transition-colors">
                <td className="px-5 py-4">
                  <p className="text-sm font-medium text-gray-200">{journey.name}</p>
                  <p className="text-xs text-gray-500 mt-0.5">{journey.description}</p>
                </td>
                <td className="px-5 py-4"><StatusBadge status={journey.status} /></td>
                <td className="px-5 py-4 text-sm text-gray-300">{formatTrigger(journey.trigger)}</td>
                <td className="px-5 py-4 text-sm text-gray-300">{journey.steps?.length ?? 0}</td>
                <td className="px-5 py-4 text-sm text-gray-400">v{journey.version}</td>
                <td className="px-5 py-4 text-sm text-gray-400">
                  {new Date(journey.created_at).toLocaleDateString()}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function StatCard({ icon, label, value }: { icon: React.ReactNode; label: string; value: string | number }) {
  return (
    <div className="bg-gray-800 border border-gray-700/50 rounded-xl p-5">
      <div className="flex items-center gap-3">
        {icon}
        <div>
          <p className="text-xs text-gray-500">{label}</p>
          <p className="text-xl font-bold text-white">{value}</p>
        </div>
      </div>
    </div>
  );
}
