import { ArrowUp, ArrowDown } from "lucide-react";
import clsx from "clsx";

interface StatsCardProps {
  label: string;
  value: string | number;
  change?: number;
  changeLabel?: string;
  icon?: React.ReactNode;
  format?: "number" | "currency" | "percent" | "latency";
}

function formatValue(
  value: string | number,
  format?: StatsCardProps["format"]
): string {
  if (typeof value === "string") return value;

  switch (format) {
    case "currency":
      return `$${value.toLocaleString("en-US", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2,
      })}`;
    case "percent":
      return `${value.toFixed(2)}%`;
    case "latency":
      if (value >= 1000) {
        return `${(value / 1000).toFixed(1)}ms`;
      }
      return `${value.toFixed(0)}us`;
    case "number":
    default:
      if (value >= 1_000_000) {
        return `${(value / 1_000_000).toFixed(1)}M`;
      }
      if (value >= 1_000) {
        return `${(value / 1_000).toFixed(1)}K`;
      }
      return value.toLocaleString();
  }
}

export default function StatsCard({
  label,
  value,
  change,
  changeLabel,
  icon,
  format,
}: StatsCardProps) {
  const isPositive = change !== undefined && change >= 0;

  return (
    <div className="bg-gray-800 border border-gray-700/50 rounded-xl p-5 hover:border-gray-600/50 transition-colors">
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm font-medium text-gray-400">{label}</p>
          <p className="mt-2 text-2xl font-bold text-white">
            {formatValue(value, format)}
          </p>
        </div>
        {icon && (
          <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-gray-700/50 text-gray-400">
            {icon}
          </div>
        )}
      </div>

      {change !== undefined && (
        <div className="mt-3 flex items-center gap-1.5">
          <span
            className={clsx(
              "inline-flex items-center gap-0.5 text-xs font-medium px-1.5 py-0.5 rounded",
              isPositive
                ? "text-emerald-400 bg-emerald-400/10"
                : "text-red-400 bg-red-400/10"
            )}
          >
            {isPositive ? (
              <ArrowUp className="w-3 h-3" />
            ) : (
              <ArrowDown className="w-3 h-3" />
            )}
            {Math.abs(change).toFixed(1)}%
          </span>
          {changeLabel && (
            <span className="text-xs text-gray-500">{changeLabel}</span>
          )}
        </div>
      )}
    </div>
  );
}
