"use client";

import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { Palette, Plus, Layers, Sparkles, Loader2, AlertCircle } from "lucide-react";
import StatusBadge from "@/components/status-badge";
import { api } from "@/lib/api";
import type { DcoTemplate } from "@/lib/types";
import { formatDate } from "@/lib/format-date";

export default function DcoPage() {
  const { data: templates, isLoading, error } = useQuery({
    queryKey: ["dco-templates"],
    queryFn: () => api.listDcoTemplates(),
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
          <p className="text-sm text-red-400">Failed to load DCO templates</p>
        </div>
      </div>
    );
  }

  const { active, totalComponents, totalVariants } = useMemo(() => {
    const list = templates ?? [];
    return {
      active: list.filter((t: DcoTemplate) => t.status === "active").length,
      totalComponents: list.reduce((sum: number, t: DcoTemplate) => sum + (t.components?.length ?? 0), 0),
      totalVariants: list.reduce(
        (sum: number, t: DcoTemplate) =>
          sum + (t.components?.reduce((vs: number, c) => vs + (c.variants?.length ?? 0), 0) ?? 0),
        0
      ),
    };
  }, [templates]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">Dynamic Creative Optimization</h1>
        <button
          onClick={() => alert("DCO template builder coming soon!")}
          className="flex items-center gap-2 px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium rounded-lg transition-colors"
        >
          <Plus className="w-4 h-4" /> New Template
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <StatCard icon={<Palette className="w-5 h-5 text-blue-400" />} label="Templates" value={templates?.length ?? 0} />
        <StatCard icon={<Sparkles className="w-5 h-5 text-emerald-400" />} label="Active" value={active} />
        <StatCard icon={<Layers className="w-5 h-5 text-purple-400" />} label="Components" value={totalComponents} />
        <StatCard icon={<Sparkles className="w-5 h-5 text-yellow-400" />} label="Variants" value={totalVariants} />
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {templates?.map((template: DcoTemplate) => (
          <div
            key={template.id}
            className="bg-gray-800 border border-gray-700/50 rounded-xl p-5 hover:border-gray-600 transition-colors cursor-pointer"
          >
            <div className="flex items-start justify-between mb-3">
              <h3 className="text-sm font-semibold text-white">{template.name}</h3>
              <StatusBadge status={template.status} />
            </div>
            <p className="text-xs text-gray-500 mb-4">{template.description}</p>
            <div className="grid grid-cols-3 gap-3">
              <div>
                <p className="text-xs text-gray-500">Components</p>
                <p className="text-lg font-bold text-gray-200">{template.components?.length ?? 0}</p>
              </div>
              <div>
                <p className="text-xs text-gray-500">Variants</p>
                <p className="text-lg font-bold text-gray-200">
                  {template.components?.reduce((s: number, c) => s + (c.variants?.length ?? 0), 0) ?? 0}
                </p>
              </div>
              <div>
                <p className="text-xs text-gray-500">Rules</p>
                <p className="text-lg font-bold text-gray-200">{template.rules?.length ?? 0}</p>
              </div>
            </div>
            <p className="text-xs text-gray-500 mt-3">
              Created {formatDate(template.created_at)}
            </p>
          </div>
        ))}
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
