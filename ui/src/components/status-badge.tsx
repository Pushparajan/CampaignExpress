import clsx from "clsx";

type Status =
  | "active"
  | "paused"
  | "draft"
  | "completed"
  | "error"
  | "pending_review"
  | "rejected"
  | "archived"
  | "running"
  | "cancelled";

interface StatusBadgeProps {
  status: Status;
  className?: string;
}

const statusStyles: Record<Status, { bg: string; text: string; dot: string }> = {
  active: {
    bg: "bg-emerald-400/10",
    text: "text-emerald-400",
    dot: "bg-emerald-400",
  },
  paused: {
    bg: "bg-yellow-400/10",
    text: "text-yellow-400",
    dot: "bg-yellow-400",
  },
  draft: {
    bg: "bg-gray-400/10",
    text: "text-gray-400",
    dot: "bg-gray-400",
  },
  completed: {
    bg: "bg-blue-400/10",
    text: "text-blue-400",
    dot: "bg-blue-400",
  },
  error: {
    bg: "bg-red-400/10",
    text: "text-red-400",
    dot: "bg-red-400",
  },
  pending_review: {
    bg: "bg-orange-400/10",
    text: "text-orange-400",
    dot: "bg-orange-400",
  },
  rejected: {
    bg: "bg-red-400/10",
    text: "text-red-400",
    dot: "bg-red-400",
  },
  archived: {
    bg: "bg-gray-400/10",
    text: "text-gray-500",
    dot: "bg-gray-500",
  },
  running: {
    bg: "bg-cyan-400/10",
    text: "text-cyan-400",
    dot: "bg-cyan-400",
  },
  cancelled: {
    bg: "bg-gray-400/10",
    text: "text-gray-400",
    dot: "bg-gray-400",
  },
};

const statusLabels: Record<Status, string> = {
  active: "Active",
  paused: "Paused",
  draft: "Draft",
  completed: "Completed",
  error: "Error",
  pending_review: "Pending Review",
  rejected: "Rejected",
  archived: "Archived",
  running: "Running",
  cancelled: "Cancelled",
};

export default function StatusBadge({ status, className }: StatusBadgeProps) {
  const style = statusStyles[status] || statusStyles.draft;
  const label = statusLabels[status] || status;

  return (
    <span
      className={clsx(
        "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium",
        style.bg,
        style.text,
        className
      )}
    >
      <span className={clsx("w-1.5 h-1.5 rounded-full", style.dot)} />
      {label}
    </span>
  );
}
