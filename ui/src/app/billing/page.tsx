"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import type { PricingPlan, Invoice } from "@/lib/types";

export default function BillingPage() {
  const { data: plans = [] } = useQuery<PricingPlan[]>({
    queryKey: ["plans"],
    queryFn: () => api.listPlans(),
  });

  const { data: invoices = [] } = useQuery<Invoice[]>({
    queryKey: ["invoices"],
    queryFn: () => api.listInvoices(),
  });

  return (
    <div className="space-y-8">
      <div>
        <h1 className="text-2xl font-bold text-white">Billing & Plans</h1>
        <p className="text-gray-400 mt-1">
          Pricing plans, subscriptions, invoices, and usage metering
        </p>
      </div>

      {/* Pricing Plans */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">
          Pricing Plans
        </h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {plans.map((plan) => (
            <div
              key={plan.id}
              className={`bg-gray-800 rounded-lg border p-5 ${
                plan.tier === "professional"
                  ? "border-primary ring-1 ring-primary/30"
                  : "border-gray-700"
              }`}
            >
              {plan.tier === "professional" && (
                <span className="text-xs bg-primary text-white px-2 py-0.5 rounded mb-2 inline-block">
                  Most Popular
                </span>
              )}
              <h3 className="text-white font-semibold text-lg">{plan.name}</h3>
              <div className="mt-2">
                <span className="text-3xl font-bold text-white">
                  ${plan.monthly_price}
                </span>
                <span className="text-gray-400 text-sm">/mo</span>
              </div>
              <p className="text-gray-500 text-xs mt-1">
                ${plan.annual_price}/yr (save{" "}
                {Math.round(
                  (1 - plan.annual_price / (plan.monthly_price * 12)) * 100
                )}
                %)
              </p>
              <ul className="mt-4 space-y-2 text-sm text-gray-400">
                <li>
                  {plan.included_offers === 0
                    ? "Unlimited"
                    : plan.included_offers.toLocaleString()}{" "}
                  offers/hr
                </li>
                <li>
                  {plan.included_api_calls === 0
                    ? "Unlimited"
                    : plan.included_api_calls.toLocaleString()}{" "}
                  API calls/day
                </li>
                {plan.features.slice(0, 4).map((f) => (
                  <li key={f}>{f}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>

      {/* Invoices */}
      <section>
        <h2 className="text-lg font-semibold text-white mb-4">Invoices</h2>
        <div className="bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
          <table className="w-full text-sm">
            <thead className="bg-gray-900">
              <tr>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Invoice
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Amount
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Status
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Items
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Issued
                </th>
                <th className="text-left px-4 py-3 text-gray-400 font-medium">
                  Paid
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-700">
              {invoices.map((inv) => (
                <tr key={inv.id} className="hover:bg-gray-750">
                  <td className="px-4 py-3 text-white font-mono text-xs">
                    {inv.id.substring(0, 8)}...
                  </td>
                  <td className="px-4 py-3 text-white font-medium">
                    ${inv.amount.toFixed(2)} {inv.currency}
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        inv.status === "paid"
                          ? "bg-emerald-900 text-emerald-300"
                          : inv.status === "pending"
                          ? "bg-amber-900 text-amber-300"
                          : "bg-red-900 text-red-300"
                      }`}
                    >
                      {inv.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {inv.line_items.length} items
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {new Date(inv.issued_at).toLocaleDateString()}
                  </td>
                  <td className="px-4 py-3 text-gray-400">
                    {inv.paid_at
                      ? new Date(inv.paid_at).toLocaleDateString()
                      : "—"}
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
