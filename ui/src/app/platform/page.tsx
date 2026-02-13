"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { Tenant, Role, ComplianceStatus, DataSubjectRequest } from "@/lib/types";

export default function PlatformPage() {
  const { data: tenants = [] } = useQuery<Tenant[]>({
    queryKey: ["tenants"],
    queryFn: () => api.listTenants(),
  });

  const { data: roles = [] } = useQuery<Role[]>({
    queryKey: ["roles"],
    queryFn: () => api.listRoles(),
  });

  const { data: compliance = [] } = useQuery<ComplianceStatus[]>({
    queryKey: ["compliance"],
    queryFn: () => api.getComplianceStatus(),
  });

  const { data: dsrs = [] } = useQuery<DataSubjectRequest[]>({
    queryKey: ["dsrs"],
    queryFn: () => api.listDsrs(),
  });

  const tierColors: Record<string, string> = {
    free: "bg-gray-700 text-gray-300",
    starter: "bg-blue-900 text-blue-300",
    professional: "bg-purple-900 text-purple-300",
    enterprise: "bg-amber-900 text-amber-300",
    custom: "bg-emerald-900 text-emerald-300",
  };

  const complianceColors: Record<string, string> = {
    compliant: "text-emerald-400",
    in_progress: "text-amber-400",
    planned: "text-gray-400",
    non_compliant: "text-red-400",
  };

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-white">Platform Management</h1>
        <p className="text-gray-400 mt-1">
          Authentication, tenants, roles, compliance, and privacy
        </p>
      </div>

      {/* Tenants */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">Tenants</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {tenants.map((tenant) => (
            <div
              key={tenant.id}
              className="bg-gray-800 rounded-lg border border-gray-700 p-5"
            >
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-white font-medium">{tenant.name}</h3>
                <span
                  className={`px-2 py-0.5 rounded text-xs font-medium ${
                    tierColors[tenant.pricing_tier] || "bg-gray-700 text-gray-300"
                  }`}
                >
                  {tenant.pricing_tier}
                </span>
              </div>
              <p className="text-xs text-gray-500 mb-3">/{tenant.slug}</p>
              <div className="grid grid-cols-2 gap-3 text-sm">
                <div>
                  <span className="text-gray-500">Status</span>
                  <p className="text-white capitalize">{tenant.status}</p>
                </div>
                <div>
                  <span className="text-gray-500">Users</span>
                  <p className="text-white">{tenant.usage?.users_count ?? 0}</p>
                </div>
                <div>
                  <span className="text-gray-500">Campaigns</span>
                  <p className="text-white">
                    {tenant.usage?.campaigns_active ?? 0}
                  </p>
                </div>
                <div>
                  <span className="text-gray-500">Offers Today</span>
                  <p className="text-white">
                    {(tenant.usage?.offers_served_today ?? 0).toLocaleString()}
                  </p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Roles & Permissions */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          Roles & Permissions
        </h2>
        <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-900">
              <tr>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Role
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Description
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Permissions
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Type
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {roles.map((role) => (
                <tr key={role.id} className="hover:bg-gray-750">
                  <td className="px-4 py-3 text-white font-medium">
                    {role.name}
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {role.description}
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {role.permissions.length} permissions
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        role.is_system
                          ? "bg-blue-900 text-blue-300"
                          : "bg-gray-700 text-gray-300"
                      }`}
                    >
                      {role.is_system ? "System" : "Custom"}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Compliance */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          Compliance Status
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {compliance.map((c) => (
            <div
              key={c.framework}
              className="bg-gray-800 rounded-lg border border-gray-700 p-4 text-center"
            >
              <h3 className="text-white font-medium uppercase text-sm">
                {c.framework}
              </h3>
              <p
                className={`text-lg font-bold mt-1 capitalize ${
                  complianceColors[c.status] || "text-gray-400"
                }`}
              >
                {c.status.replace("_", " ")}
              </p>
              {c.last_audit && (
                <p className="text-xs text-gray-500 mt-1">
                  Last: {new Date(c.last_audit).toLocaleDateString()}
                </p>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Privacy / DSRs */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          Data Subject Requests
        </h2>
        <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-900">
              <tr>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  ID
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  User
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Type
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Status
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Requested
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {dsrs.map((dsr) => (
                <tr key={dsr.id} className="hover:bg-gray-750">
                  <td className="px-4 py-3 text-gray-400 font-mono text-xs">
                    {dsr.id.substring(0, 8)}...
                  </td>
                  <td className="px-4 py-3 text-white">
                    {dsr.user_identifier}
                  </td>
                  <td className="px-4 py-3 text-gray-400 capitalize">
                    {dsr.request_type}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        dsr.status === "completed"
                          ? "bg-emerald-900 text-emerald-300"
                          : dsr.status === "pending"
                          ? "bg-amber-900 text-amber-300"
                          : dsr.status === "failed"
                          ? "bg-red-900 text-red-300"
                          : "bg-blue-900 text-blue-300"
                      }`}
                    >
                      {dsr.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {new Date(dsr.requested_at).toLocaleDateString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
