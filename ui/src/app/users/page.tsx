"use client";

import { useState, useMemo } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Users,
  Plus,
  Search,
  Loader2,
  AlertCircle,
  Shield,
  UserX,
  UserCheck,
  Trash2,
  Mail,
  X,
} from "lucide-react";
import StatusBadge from "@/components/status-badge";
import { api } from "@/lib/api";
import type { ManagedUser, UserInvitation } from "@/lib/types";
import { formatDate, formatDateTime } from "@/lib/format-date";

const ROLES = ["Admin", "Campaign Manager", "Analyst", "Viewer"];

export default function UsersPage() {
  const queryClient = useQueryClient();
  const [searchQuery, setSearchQuery] = useState("");
  const [showInviteModal, setShowInviteModal] = useState(false);
  const [inviteEmail, setInviteEmail] = useState("");
  const [inviteRole, setInviteRole] = useState("Viewer");
  const [roleEditTarget, setRoleEditTarget] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ManagedUser | null>(null);

  const {
    data: users,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["users"],
    queryFn: () => api.listUsers(),
  });

  const { data: invitations } = useQuery({
    queryKey: ["invitations"],
    queryFn: () => api.listInvitations(),
  });

  const disableMutation = useMutation({
    mutationFn: (id: string) => api.disableUser(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["users"] }),
  });

  const enableMutation = useMutation({
    mutationFn: (id: string) => api.enableUser(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["users"] }),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
      setDeleteTarget(null);
    },
  });

  const roleMutation = useMutation({
    mutationFn: ({ id, role }: { id: string; role: string }) =>
      api.updateUserRole(id, role),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
      setRoleEditTarget(null);
    },
  });

  const inviteMutation = useMutation({
    mutationFn: (data: { email: string; role: string }) =>
      api.createInvitation(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["invitations"] });
      setShowInviteModal(false);
      setInviteEmail("");
      setInviteRole("Viewer");
    },
  });

  const revokeMutation = useMutation({
    mutationFn: (id: string) => api.revokeInvitation(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["invitations"] }),
  });

  const filteredUsers = useMemo(
    () =>
      (users ?? []).filter(
        (u) =>
          !searchQuery ||
          u.display_name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          u.email.toLowerCase().includes(searchQuery.toLowerCase())
      ),
    [users, searchQuery]
  );

  const pendingInvitations = useMemo(
    () => (invitations ?? []).filter((i) => i.status === "pending"),
    [invitations]
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="w-8 h-8 text-primary animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3 text-center">
          <AlertCircle className="w-8 h-8 text-red-400" />
          <p className="text-sm text-red-400">Failed to load users</p>
        </div>
      </div>
    );
  }

  const activeCount = filteredUsers.filter((u) => u.status === "active").length;
  const disabledCount = filteredUsers.filter((u) => u.status === "disabled").length;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-white">User Management</h1>
        <button
          onClick={() => setShowInviteModal(true)}
          className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/80 text-white text-sm font-medium rounded-lg transition-colors"
        >
          <Mail className="w-4 h-4" /> Invite User
        </button>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <StatCard
          label="Total Users"
          value={filteredUsers.length}
          icon={<Users className="w-5 h-5 text-blue-400" />}
        />
        <StatCard
          label="Active"
          value={activeCount}
          icon={<UserCheck className="w-5 h-5 text-emerald-400" />}
        />
        <StatCard
          label="Disabled"
          value={disabledCount}
          icon={<UserX className="w-5 h-5 text-red-400" />}
        />
        <StatCard
          label="Pending Invites"
          value={pendingInvitations.length}
          icon={<Mail className="w-5 h-5 text-amber-400" />}
        />
      </div>

      {/* Search */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
        <input
          type="text"
          placeholder="Search users by name or email..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          aria-label="Search users"
          className="w-full pl-10 pr-4 py-2.5 bg-gray-800 border border-gray-700 rounded-lg text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
        />
      </div>

      {/* Users Table */}
      <div className="bg-gray-800/50 border border-gray-700/50 rounded-xl overflow-hidden">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-700/50">
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">User</th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Role</th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Status</th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Auth</th>
              <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Last Login</th>
              <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase">Actions</th>
            </tr>
          </thead>
          <tbody>
            {filteredUsers.map((user) => (
              <tr
                key={user.id}
                className="border-b border-gray-700/30 hover:bg-gray-700/20 transition-colors"
              >
                <td className="px-4 py-3">
                  <div>
                    <p className="text-sm font-medium text-white">{user.display_name}</p>
                    <p className="text-xs text-gray-500">{user.email}</p>
                  </div>
                </td>
                <td className="px-4 py-3">
                  {roleEditTarget === user.id ? (
                    <select
                      defaultValue={user.role}
                      onChange={(e) =>
                        roleMutation.mutate({ id: user.id, role: e.target.value })
                      }
                      onBlur={() => setRoleEditTarget(null)}
                      aria-label={`Change role for ${user.display_name}`}
                      className="bg-gray-700 text-white text-xs rounded px-2 py-1 border border-gray-600 focus:outline-none focus:ring-1 focus:ring-primary"
                    >
                      {ROLES.map((r) => (
                        <option key={r} value={r}>
                          {r}
                        </option>
                      ))}
                    </select>
                  ) : (
                    <button
                      onClick={() => setRoleEditTarget(user.id)}
                      aria-label={`Edit role for ${user.display_name}`}
                      className="flex items-center gap-1.5 text-xs text-gray-300 hover:text-white transition-colors"
                    >
                      <Shield className="w-3 h-3" />
                      {user.role}
                    </button>
                  )}
                </td>
                <td className="px-4 py-3">
                  <StatusBadge status={user.status === "active" ? "active" : user.status === "disabled" ? "paused" : "draft"} />
                </td>
                <td className="px-4 py-3">
                  <span className="text-xs text-gray-400">{user.auth_provider}</span>
                </td>
                <td className="px-4 py-3">
                  <span className="text-xs text-gray-400">
                    {user.last_login ? formatDateTime(user.last_login) : "Never"}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center justify-end gap-2">
                    {user.status === "active" ? (
                      <button
                        onClick={() => disableMutation.mutate(user.id)}
                        aria-label={`Disable ${user.display_name}`}
                        className="p-1.5 text-gray-400 hover:text-amber-400 hover:bg-gray-700 rounded transition-colors"
                        title="Disable user"
                      >
                        <UserX className="w-4 h-4" />
                      </button>
                    ) : (
                      <button
                        onClick={() => enableMutation.mutate(user.id)}
                        aria-label={`Enable ${user.display_name}`}
                        className="p-1.5 text-gray-400 hover:text-emerald-400 hover:bg-gray-700 rounded transition-colors"
                        title="Enable user"
                      >
                        <UserCheck className="w-4 h-4" />
                      </button>
                    )}
                    <button
                      onClick={() => setDeleteTarget(user)}
                      aria-label={`Delete ${user.display_name}`}
                      className="p-1.5 text-gray-400 hover:text-red-400 hover:bg-gray-700 rounded transition-colors"
                      title="Remove user"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pending Invitations */}
      {pendingInvitations.length > 0 && (
        <div>
          <h2 className="text-sm font-semibold text-white mb-3">Pending Invitations</h2>
          <div className="bg-gray-800/50 border border-gray-700/50 rounded-xl overflow-hidden">
            <table className="w-full">
              <thead>
                <tr className="border-b border-gray-700/50">
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Email</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Role</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Invited</th>
                  <th className="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase">Expires</th>
                  <th className="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase">Actions</th>
                </tr>
              </thead>
              <tbody>
                {pendingInvitations.map((inv) => (
                  <tr
                    key={inv.id}
                    className="border-b border-gray-700/30 hover:bg-gray-700/20 transition-colors"
                  >
                    <td className="px-4 py-3 text-sm text-white">{inv.email}</td>
                    <td className="px-4 py-3 text-xs text-gray-300">{inv.role}</td>
                    <td className="px-4 py-3 text-xs text-gray-400">{formatDate(inv.created_at)}</td>
                    <td className="px-4 py-3 text-xs text-gray-400">{formatDate(inv.expires_at)}</td>
                    <td className="px-4 py-3 text-right">
                      <button
                        onClick={() => revokeMutation.mutate(inv.id)}
                        aria-label={`Revoke invitation for ${inv.email}`}
                        className="text-xs text-red-400 hover:text-red-300 transition-colors"
                      >
                        Revoke
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Invite Modal */}
      {showInviteModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div
            role="dialog"
            aria-label="Invite user"
            className="bg-gray-800 border border-gray-700 rounded-xl w-full max-w-md p-6"
          >
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold text-white">Invite User</h3>
              <button
                onClick={() => setShowInviteModal(false)}
                aria-label="Close invite modal"
                className="p-1 text-gray-400 hover:text-white transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>
            <form
              onSubmit={(e) => {
                e.preventDefault();
                inviteMutation.mutate({ email: inviteEmail, role: inviteRole });
              }}
              className="space-y-4"
            >
              <div>
                <label htmlFor="invite-email" className="block text-xs font-medium text-gray-400 mb-1">
                  Email Address
                </label>
                <input
                  id="invite-email"
                  type="email"
                  required
                  value={inviteEmail}
                  onChange={(e) => setInviteEmail(e.target.value)}
                  placeholder="user@company.com"
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-sm text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
              </div>
              <div>
                <label htmlFor="invite-role" className="block text-xs font-medium text-gray-400 mb-1">
                  Role
                </label>
                <select
                  id="invite-role"
                  value={inviteRole}
                  onChange={(e) => setInviteRole(e.target.value)}
                  className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-sm text-white focus:outline-none focus:ring-2 focus:ring-primary/50"
                >
                  {ROLES.map((r) => (
                    <option key={r} value={r}>
                      {r}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex gap-3 pt-2">
                <button
                  type="button"
                  onClick={() => setShowInviteModal(false)}
                  className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 text-sm font-medium rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={inviteMutation.isPending}
                  className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-primary hover:bg-primary/80 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
                >
                  {inviteMutation.isPending && <Loader2 className="w-4 h-4 animate-spin" />}
                  Send Invitation
                </button>
              </div>
            </form>
          </div>
        </div>
      )}

      {/* Delete Confirmation */}
      {deleteTarget && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div
            role="dialog"
            aria-label="Confirm user deletion"
            className="bg-gray-800 border border-gray-700 rounded-xl w-full max-w-sm p-6"
          >
            <h3 className="text-lg font-semibold text-white mb-2">Remove User</h3>
            <p className="text-sm text-gray-400 mb-4">
              Are you sure you want to remove <span className="text-white font-medium">{deleteTarget.display_name}</span>?
              This action cannot be undone.
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

function StatCard({ label, value, icon }: { label: string; value: number; icon: React.ReactNode }) {
  return (
    <div className="bg-gray-800/50 border border-gray-700/50 rounded-xl p-4 flex items-center gap-3">
      {icon}
      <div>
        <p className="text-xs text-gray-500">{label}</p>
        <p className="text-xl font-bold text-white">{value}</p>
      </div>
    </div>
  );
}
