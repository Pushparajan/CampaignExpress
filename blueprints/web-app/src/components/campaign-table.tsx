/**
 * Campaign table component with status badges and actions.
 */

"use client";

import type { Campaign } from "@/lib/types";

interface CampaignTableProps {
  campaigns: Campaign[];
  onPause?: (id: string) => void;
  onResume?: (id: string) => void;
  onDelete?: (id: string) => void;
}

const statusColors: Record<string, string> = {
  active: "bg-green-100 text-green-800",
  paused: "bg-yellow-100 text-yellow-800",
  draft: "bg-gray-100 text-gray-800",
  completed: "bg-blue-100 text-blue-800",
  error: "bg-red-100 text-red-800",
};

export function CampaignTable({
  campaigns,
  onPause,
  onResume,
  onDelete,
}: CampaignTableProps) {
  return (
    <div className="overflow-x-auto rounded-lg border">
      <table className="min-w-full divide-y divide-gray-200">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-6 py-3 text-left text-xs font-medium uppercase text-gray-500">
              Name
            </th>
            <th className="px-6 py-3 text-left text-xs font-medium uppercase text-gray-500">
              Status
            </th>
            <th className="px-6 py-3 text-right text-xs font-medium uppercase text-gray-500">
              Budget
            </th>
            <th className="px-6 py-3 text-right text-xs font-medium uppercase text-gray-500">
              Spend
            </th>
            <th className="px-6 py-3 text-right text-xs font-medium uppercase text-gray-500">
              CTR
            </th>
            <th className="px-6 py-3 text-right text-xs font-medium uppercase text-gray-500">
              Actions
            </th>
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200 bg-white">
          {campaigns.map((campaign) => (
            <tr key={campaign.id} className="hover:bg-gray-50">
              <td className="whitespace-nowrap px-6 py-4 font-medium text-gray-900">
                {campaign.name}
              </td>
              <td className="whitespace-nowrap px-6 py-4">
                <span
                  className={`inline-flex rounded-full px-2 py-1 text-xs font-semibold ${
                    statusColors[campaign.status] || "bg-gray-100"
                  }`}
                >
                  {campaign.status}
                </span>
              </td>
              <td className="whitespace-nowrap px-6 py-4 text-right text-gray-700">
                ${campaign.budget.toLocaleString()}
              </td>
              <td className="whitespace-nowrap px-6 py-4 text-right text-gray-700">
                ${campaign.stats.spend.toLocaleString()}
              </td>
              <td className="whitespace-nowrap px-6 py-4 text-right text-gray-700">
                {(campaign.stats.ctr * 100).toFixed(2)}%
              </td>
              <td className="whitespace-nowrap px-6 py-4 text-right">
                <div className="flex justify-end gap-2">
                  {campaign.status === "active" && onPause && (
                    <button
                      onClick={() => onPause(campaign.id)}
                      className="text-sm text-yellow-600 hover:text-yellow-800"
                    >
                      Pause
                    </button>
                  )}
                  {campaign.status === "paused" && onResume && (
                    <button
                      onClick={() => onResume(campaign.id)}
                      className="text-sm text-green-600 hover:text-green-800"
                    >
                      Resume
                    </button>
                  )}
                  {onDelete && (
                    <button
                      onClick={() => onDelete(campaign.id)}
                      className="text-sm text-red-600 hover:text-red-800"
                    >
                      Delete
                    </button>
                  )}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
