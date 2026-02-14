/**
 * Campaigns Page - Campaign Express Web Blueprint
 *
 * Full campaign management with create, pause, resume, delete.
 * Demonstrates CRUD operations via the Campaign Express API.
 */

"use client";

import { useState, useEffect } from "react";
import {
  useCampaigns,
  useCreateCampaign,
  usePauseCampaign,
  useResumeCampaign,
  useDeleteCampaign,
} from "@/hooks/use-campaigns";
import { CampaignTable } from "@/components/campaign-table";
import { tracker } from "@/lib/ce-tracker";

export default function CampaignsPage() {
  const { data: campaigns, isLoading } = useCampaigns();
  const createCampaign = useCreateCampaign();
  const pauseCampaign = usePauseCampaign();
  const resumeCampaign = useResumeCampaign();
  const deleteCampaign = useDeleteCampaign();
  const [showCreate, setShowCreate] = useState(false);

  useEffect(() => {
    tracker.trackPageView("/campaigns", "Campaigns");
  }, []);

  const handleCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = new FormData(e.currentTarget);
    await createCampaign.mutateAsync({
      name: form.get("name") as string,
      budget: Number(form.get("budget")),
      daily_budget: Number(form.get("daily_budget")),
      pacing: "even",
      targeting: {
        geo: ["US"],
        segments: [],
        devices: ["desktop", "mobile"],
        floor_price: 0.5,
      },
      schedule_start: form.get("start_date") as string,
      schedule_end: form.get("end_date") as string,
    });
    setShowCreate(false);
    tracker.trackCustomEvent("campaign_created");
  };

  return (
    <div className="mx-auto max-w-7xl px-4 py-8">
      <div className="mb-6 flex items-center justify-between">
        <h1 className="text-3xl font-bold text-gray-900">Campaigns</h1>
        <button
          onClick={() => setShowCreate(!showCreate)}
          className="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white hover:bg-primary-700"
        >
          Create Campaign
        </button>
      </div>

      {/* Create Campaign Form */}
      {showCreate && (
        <div className="mb-6 rounded-lg border bg-white p-6 shadow-sm">
          <h2 className="mb-4 text-lg font-semibold">New Campaign</h2>
          <form onSubmit={handleCreate} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Campaign Name
              </label>
              <input
                name="name"
                required
                className="mt-1 w-full rounded-md border px-3 py-2 text-sm"
                placeholder="Summer Sale 2026"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Total Budget ($)
              </label>
              <input
                name="budget"
                type="number"
                required
                className="mt-1 w-full rounded-md border px-3 py-2 text-sm"
                placeholder="10000"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Daily Budget ($)
              </label>
              <input
                name="daily_budget"
                type="number"
                required
                className="mt-1 w-full rounded-md border px-3 py-2 text-sm"
                placeholder="500"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">
                Start Date
              </label>
              <input
                name="start_date"
                type="date"
                required
                className="mt-1 w-full rounded-md border px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700">
                End Date
              </label>
              <input
                name="end_date"
                type="date"
                required
                className="mt-1 w-full rounded-md border px-3 py-2 text-sm"
              />
            </div>
            <div className="flex items-end">
              <button
                type="submit"
                disabled={createCampaign.isPending}
                className="rounded-lg bg-green-600 px-6 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50"
              >
                {createCampaign.isPending ? "Creating..." : "Create"}
              </button>
            </div>
          </form>
        </div>
      )}

      {/* Campaign List */}
      {isLoading ? (
        <p className="text-gray-500">Loading campaigns...</p>
      ) : campaigns && campaigns.length > 0 ? (
        <CampaignTable
          campaigns={campaigns}
          onPause={(id) => pauseCampaign.mutate(id)}
          onResume={(id) => resumeCampaign.mutate(id)}
          onDelete={(id) => {
            if (confirm("Delete this campaign?")) deleteCampaign.mutate(id);
          }}
        />
      ) : (
        <div className="rounded-lg border bg-white p-12 text-center">
          <p className="text-lg text-gray-500">No campaigns yet</p>
          <p className="mt-2 text-sm text-gray-400">
            Click &quot;Create Campaign&quot; to get started.
          </p>
        </div>
      )}
    </div>
  );
}
