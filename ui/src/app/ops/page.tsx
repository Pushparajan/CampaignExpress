"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { Incident, BackupSchedule } from "@/lib/types";

export default function OpsPage() {
  const { data: statusPage } = useQuery<Record<string, unknown>>({
    queryKey: ["statusPage"],
    queryFn: () => api.getStatusPage(),
  });

  const { data: incidents = [] } = useQuery<Incident[]>({
    queryKey: ["incidents"],
    queryFn: () => api.listIncidents(),
  });

  const { data: slaReport } = useQuery<Record<string, unknown>>({
    queryKey: ["slaReport"],
    queryFn: () => api.getSlaReport(),
  });

  const { data: backups = [] } = useQuery<BackupSchedule[]>({
    queryKey: ["backups"],
    queryFn: () => api.listBackups(),
  });

  const statusColors: Record<string, string> = {
    operational: "bg-emerald-500",
    degraded_performance: "bg-amber-500",
    partial_outage: "bg-orange-500",
    major_outage: "bg-red-500",
    maintenance: "bg-blue-500",
  };

  const severityColors: Record<string, string> = {
    critical: "bg-red-900 text-red-300",
    major: "bg-orange-900 text-orange-300",
    minor: "bg-amber-900 text-amber-300",
    info: "bg-blue-900 text-blue-300",
  };

  const components = (statusPage as { components?: Array<{ id: string; name: string; status: string; group: string }> })?.components || [];
  const slaTargets = (slaReport as { targets?: Array<{ name: string; target_percent: number; current_percent: number; measurement_window: string }> })?.targets || [];

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-white">Operations</h1>
        <p className="text-gray-400 mt-1">
          System status, SLA tracking, incidents, and backups
        </p>
      </div>

      {/* Status Page */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          System Status
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
          {components.map((comp) => (
            <div
              key={comp.id}
              className="bg-gray-800 rounded-lg border border-gray-700 p-4 flex items-center gap-3"
            >
              <div
                className={`w-3 h-3 rounded-full ${
                  statusColors[comp.status] || "bg-gray-500"
                }`}
              />
              <div>
                <p className="text-white text-sm font-medium">{comp.name}</p>
                <p className="text-gray-500 text-xs capitalize">
                  {comp.status.replace(/_/g, " ")}
                </p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* SLA Tracking */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          SLA Tracking
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {slaTargets.map((sla) => {
            const met = sla.current_percent >= sla.target_percent;
            return (
              <div
                key={sla.name}
                className="bg-gray-800 rounded-lg border border-gray-700 p-4"
              >
                <h3 className="text-white font-medium text-sm">{sla.name}</h3>
                <div className="flex items-baseline gap-2 mt-2">
                  <span
                    className={`text-2xl font-bold ${
                      met ? "text-emerald-400" : "text-red-400"
                    }`}
                  >
                    {sla.current_percent.toFixed(2)}%
                  </span>
                  <span className="text-gray-500 text-sm">
                    / {sla.target_percent}% target
                  </span>
                </div>
                <div className="mt-2 h-2 bg-gray-700 rounded-full overflow-hidden">
                  <div
                    className={`h-full rounded-full ${
                      met ? "bg-emerald-500" : "bg-red-500"
                    }`}
                    style={{
                      width: `${Math.min(
                        (sla.current_percent / sla.target_percent) * 100,
                        100
                      )}%`,
                    }}
                  />
                </div>
                <p className="text-xs text-gray-500 mt-1">
                  Window: {sla.measurement_window}
                </p>
              </div>
            );
          })}
        </div>
      </section>

      {/* Incidents */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">Incidents</h2>
        <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-900">
              <tr>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Title
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Severity
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Status
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Affected
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Created
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {incidents.map((inc) => (
                <tr key={inc.id} className="hover:bg-gray-750">
                  <td className="px-4 py-3 text-white">{inc.title}</td>
                  <td className="px-4 py-3">
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        severityColors[inc.severity] || ""
                      }`}
                    >
                      {inc.severity}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-gray-400 capitalize">
                    {inc.status}
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {inc.affected_components.join(", ")}
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {new Date(inc.created_at).toLocaleDateString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Backups */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          Backup Schedules
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {backups.map((backup) => (
            <div
              key={backup.id}
              className="bg-gray-800 rounded-lg border border-gray-700 p-4"
            >
              <div className="flex items-center justify-between">
                <h3 className="text-white font-medium capitalize">
                  {backup.target}
                </h3>
                <span
                  className={`text-xs px-2 py-0.5 rounded ${
                    backup.enabled
                      ? "bg-emerald-900 text-emerald-300"
                      : "bg-gray-700 text-gray-400"
                  }`}
                >
                  {backup.enabled ? "Active" : "Disabled"}
                </span>
              </div>
              <div className="mt-3 grid grid-cols-2 gap-2 text-sm">
                <div>
                  <span className="text-gray-500">Schedule</span>
                  <p className="text-gray-300 font-mono text-xs">
                    {backup.cron_expression}
                  </p>
                </div>
                <div>
                  <span className="text-gray-500">Retention</span>
                  <p className="text-gray-300">{backup.retention_days} days</p>
                </div>
                <div>
                  <span className="text-gray-500">Last Run</span>
                  <p className="text-gray-300">
                    {backup.last_run
                      ? new Date(backup.last_run).toLocaleDateString()
                      : "Never"}
                  </p>
                </div>
                <div>
                  <span className="text-gray-500">Next Run</span>
                  <p className="text-gray-300">
                    {new Date(backup.next_run).toLocaleDateString()}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
