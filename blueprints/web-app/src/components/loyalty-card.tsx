/**
 * Loyalty program card showing tier, balance, and progress.
 */

"use client";

import type { LoyaltyBalance } from "@/lib/types";

interface LoyaltyCardProps {
  balance: LoyaltyBalance;
  onEarn?: () => void;
  onRedeem?: () => void;
}

const tierColors: Record<string, { bg: string; text: string; accent: string }> = {
  green: { bg: "bg-green-50", text: "text-green-800", accent: "bg-green-500" },
  gold: { bg: "bg-yellow-50", text: "text-yellow-800", accent: "bg-yellow-500" },
  reserve: { bg: "bg-purple-50", text: "text-purple-800", accent: "bg-purple-500" },
};

export function LoyaltyCard({ balance, onEarn, onRedeem }: LoyaltyCardProps) {
  const tier = tierColors[balance.tier] || tierColors.green;

  return (
    <div className={`rounded-xl ${tier.bg} p-6 shadow-md`}>
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium text-gray-500">Loyalty Tier</p>
          <p className={`text-2xl font-bold capitalize ${tier.text}`}>
            {balance.tier}
          </p>
        </div>
        <div className={`h-12 w-12 rounded-full ${tier.accent} flex items-center justify-center`}>
          <span className="text-xl text-white">&#9733;</span>
        </div>
      </div>

      <div className="mt-4 grid grid-cols-2 gap-4">
        <div>
          <p className="text-sm text-gray-500">Stars Balance</p>
          <p className="text-xl font-bold text-gray-900">
            {balance.stars_balance.toLocaleString()}
          </p>
        </div>
        <div>
          <p className="text-sm text-gray-500">Lifetime Stars</p>
          <p className="text-xl font-bold text-gray-900">
            {balance.lifetime_stars.toLocaleString()}
          </p>
        </div>
      </div>

      {/* Progress to next tier */}
      <div className="mt-4">
        <div className="flex justify-between text-xs text-gray-500">
          <span>Progress to Next Tier</span>
          <span>{Math.round(balance.next_tier_progress * 100)}%</span>
        </div>
        <div className="mt-1 h-2 w-full rounded-full bg-white">
          <div
            className={`h-2 rounded-full ${tier.accent} transition-all duration-500`}
            style={{ width: `${Math.min(balance.next_tier_progress * 100, 100)}%` }}
          />
        </div>
      </div>

      <div className="mt-4 flex gap-2">
        {onEarn && (
          <button
            onClick={onEarn}
            className="flex-1 rounded-lg bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50"
          >
            Earn Stars
          </button>
        )}
        {onRedeem && (
          <button
            onClick={onRedeem}
            className={`flex-1 rounded-lg ${tier.accent} px-4 py-2 text-sm font-medium text-white shadow-sm hover:opacity-90`}
          >
            Redeem
          </button>
        )}
      </div>
    </div>
  );
}
