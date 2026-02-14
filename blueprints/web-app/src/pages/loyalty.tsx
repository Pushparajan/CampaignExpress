/**
 * Loyalty Page - Campaign Express Web Blueprint
 *
 * Shows loyalty tier, star balance, and earn/redeem actions.
 * Demonstrates integration with the Loyalty Program API.
 */

"use client";

import { useEffect, useState } from "react";
import { useLoyaltyBalance, useEarnStars, useRedeemStars } from "@/hooks/use-loyalty";
import { LoyaltyCard } from "@/components/loyalty-card";
import { tracker } from "@/lib/ce-tracker";

export default function LoyaltyPage() {
  // In a real app, userId comes from auth context
  const [userId] = useState("demo-user-001");
  const { data: balance, isLoading } = useLoyaltyBalance(userId);
  const earnStars = useEarnStars();
  const redeemStars = useRedeemStars();

  useEffect(() => {
    tracker.trackPageView("/loyalty", "Loyalty Program");
  }, []);

  const handleEarn = async () => {
    await earnStars.mutateAsync({
      user_id: userId,
      amount_cents: 1500,
      category: "coffee",
    });
    tracker.trackCustomEvent("loyalty_earn", { amount_cents: 1500 });
  };

  const handleRedeem = async () => {
    if (!balance || balance.stars_balance < 100) {
      alert("Not enough stars to redeem.");
      return;
    }
    await redeemStars.mutateAsync({
      user_id: userId,
      stars: 100,
      reward_id: "free-drink",
    });
    tracker.trackCustomEvent("loyalty_redeem", { stars: 100, reward: "free-drink" });
  };

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      <h1 className="mb-8 text-3xl font-bold text-gray-900">Loyalty Program</h1>

      {isLoading ? (
        <p className="text-gray-500">Loading loyalty balance...</p>
      ) : balance ? (
        <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
          <LoyaltyCard
            balance={balance}
            onEarn={handleEarn}
            onRedeem={handleRedeem}
          />

          {/* Transaction History Placeholder */}
          <div className="rounded-lg border bg-white p-6 shadow-sm">
            <h2 className="mb-4 text-lg font-semibold text-gray-800">
              How It Works
            </h2>
            <div className="space-y-4 text-sm text-gray-600">
              <div className="flex items-start gap-3">
                <span className="rounded-full bg-green-100 px-2 py-1 text-xs font-bold text-green-700">
                  1
                </span>
                <div>
                  <p className="font-medium text-gray-800">Earn Stars</p>
                  <p>
                    Every purchase earns stars. Green tier: 1x, Gold: 1.2x,
                    Reserve: 1.7x multiplier.
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <span className="rounded-full bg-yellow-100 px-2 py-1 text-xs font-bold text-yellow-700">
                  2
                </span>
                <div>
                  <p className="font-medium text-gray-800">Level Up</p>
                  <p>
                    Earn 500 qualifying stars/year for Gold, 2500 for Reserve.
                  </p>
                </div>
              </div>
              <div className="flex items-start gap-3">
                <span className="rounded-full bg-purple-100 px-2 py-1 text-xs font-bold text-purple-700">
                  3
                </span>
                <div>
                  <p className="font-medium text-gray-800">Redeem Rewards</p>
                  <p>
                    Use stars for free drinks, food items, and exclusive perks.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      ) : (
        <p className="text-gray-500">
          No loyalty data found. Start making purchases to earn stars!
        </p>
      )}
    </div>
  );
}
