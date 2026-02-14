"use client";

import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Plus,
  Trash2,
  Loader2,
  X,
  Ban,
  CheckCircle2,
} from "lucide-react";
import { api } from "@/lib/api";
import type { Tenant, Role, ComplianceStatus, DataSubjectRequest } from "@/lib/types";
import { formatDate } from "@/lib/format-date";

const TIERS = ["free", "starter", "professional", "enterprise", "custom"] as const;

export default function PlatformPage() {
  const queryClient = useQueryClient();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Tenant | null>(null);

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

  const suspendMutation = useMutation({
    mutationFn: (id: string) => api.suspendTenant(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tenants"] }),
  });

  const activateMutation = useMutation({
    mutationFn: (id: string) => api.activateTenant(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["tenants"] }),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteTenant(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tenants"] });
      setDeleteTarget(null);
    },
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
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Tenants</h2>
          <button
            onClick={() => setShowCreateModal(true)}
            className="flex items-center gap-2 px-3 py-1.5 bg-primary hover:bg-primary/80 text-white text-sm font-medium rounded-lg transition-colors"
          >
            <Plus className="w-4 h-4" /> Add Tenant
          </button>
        </div>
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
              {/* Tenant actions */}
              <div className="flex items-center gap-2 mt-4 pt-3 border-t border-gray-700">
                {tenant.status === "active" ? (
                  <button
                    onClick={() => suspendMutation.mutate(tenant.id)}
                    aria-label={`Suspend ${tenant.name}`}
                    className="flex items-center gap-1 px-2.5 py-1 text-xs text-amber-400 hover:bg-amber-400/10 rounded transition-colors"
                  >
                    <Ban className="w-3.5 h-3.5" /> Suspend
                  </button>
                ) : (
                  <button
                    onClick={() => activateMutation.mutate(tenant.id)}
                    aria-label={`Activate ${tenant.name}`}
                    className="flex items-center gap-1 px-2.5 py-1 text-xs text-emerald-400 hover:bg-emerald-400/10 rounded transition-colors"
                  >
                    <CheckCircle2 className="w-3.5 h-3.5" /> Activate
                  </button>
                )}
                <button
                  onClick={() => setDeleteTarget(tenant)}
                  aria-label={`Delete ${tenant.name}`}
                  className="flex items-center gap-1 px-2.5 py-1 text-xs text-red-400 hover:bg-red-400/10 rounded transition-colors ml-auto"
                >
                  <Trash2 className="w-3.5 h-3.5" /> Remove
                </button>
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
                  Last: {formatDate(c.last_audit)}
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
                    {formatDate(dsr.requested_at)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Create Tenant Modal */}
      {showCreateModal && (
        <CreateTenantModal onClose={() => setShowCreateModal(false)} />
      )}

      {/* Delete Confirmation */}
      {deleteTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div
            role="dialog"
            aria-label="Confirm tenant deletion"
            className="bg-gray-800 border border-gray-700 rounded-xl w-full max-w-sm p-6"
          >
            <h3 className="text-lg font-semibold text-white mb-2">Remove Tenant</h3>
            <p className="text-sm text-gray-400 mb-4">
              Are you sure you want to remove <span className="text-white font-medium">{deleteTarget.name}</span>?
              All associated data will be permanently deleted.
            </p>
            <div className="flex gap-3">
              <button
                onClick={() => setDeleteTarget(null)}
                className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm font-medium rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteMutation.mutate(deleteTarget.id)}
                disabled={deleteMutation.isPending}
                className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-500 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
              >
                {deleteMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                Remove
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function CreateTenantModal({ onClose }: { onClose: () => void }) {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [tier, setTier] = useState<typeof TIERS[number]>("starter");

  const createMutation = useMutation({
    mutationFn: () => api.createTenant({ name, slug, pricing_tier: tier }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tenants"] });
      onClose();
    },
  });

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
      <div
        role="dialog"
        aria-label="Create tenant"
        className="bg-gray-800 border border-gray-700 rounded-xl w-full max-w-md p-6"
      >
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Add Tenant</h3>
          <button
            onClick={onClose}
            aria-label="Close"
            className="p-1 text-gray-400 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            createMutation.mutate();
          }}
          className="space-y-4"
        >
          <div>
            <label htmlFor="tenant-name" className="block text-xs font-medium text-gray-400 mb-1">
              Organization Name
            </label>
            <input
              id="tenant-name"
              type="text"
              required
              value={name}
              onChange={(e) => {
                setName(e.target.value);
                if (!slug || slug === name.toLowerCase().replace(/\s+/g, "-")) {
                  setSlug(e.target.value.toLowerCase().replace(/\s+/g, "-"));
                }
              }}
              placeholder="Acme Corp"
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </div>
          <div>
            <label htmlFor="tenant-slug" className="block text-xs font-medium text-gray-400 mb-1">
              URL Slug
            </label>
            <input
              id="tenant-slug"
              type="text"
              required
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              placeholder="acme-corp"
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
          </div>
          <div>
            <label htmlFor="tenant-tier" className="block text-xs font-medium text-gray-400 mb-1">
              Pricing Tier
            </label>
            <select
              id="tenant-tier"
              value={tier}
              onChange={(e) => setTier(e.target.value as typeof TIERS[number])}
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary/50"
            >
              {TIERS.map((t) => (
                <option key={t} value={t}>
                  {t.charAt(0).toUpperCase() + t.slice(1)}
                </option>
              ))}
            </select>
          </div>
          <div className="flex gap-3 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm font-medium rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={createMutation.isPending}
              className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary hover:bg-primary/80 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
            >
              {createMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
              Create Tenant
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
