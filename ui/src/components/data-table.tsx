"use client";

import { useState, useMemo } from "react";
import { ChevronUp, ChevronDown, ChevronLeft, ChevronRight } from "lucide-react";
import clsx from "clsx";

export interface Column<T> {
  key: string;
  label: string;
  sortable?: boolean;
  render?: (row: T) => React.ReactNode;
  className?: string;
}

interface DataTableProps<T> {
  columns: Column<T>[];
  data: T[];
  onRowClick?: (row: T) => void;
  pageSize?: number;
  emptyMessage?: string;
  rowKey?: (row: T) => string;
}

type SortDirection = "asc" | "desc";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export default function DataTable<T extends Record<string, any>>({
  columns,
  data,
  onRowClick,
  pageSize = 10,
  emptyMessage = "No data available",
  rowKey,
}: DataTableProps<T>) {
  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortDir, setSortDir] = useState<SortDirection>("asc");
  const [currentPage, setCurrentPage] = useState(0);

  const sortedData = useMemo(() => {
    if (!sortKey) return data;
    return [...data].sort((a, b) => {
      const aVal = a[sortKey];
      const bVal = b[sortKey];
      if (aVal === bVal) return 0;
      if (aVal == null) return 1;
      if (bVal == null) return -1;

      let cmp: number;
      if (typeof aVal === "number" && typeof bVal === "number") {
        cmp = aVal - bVal;
      } else {
        cmp = String(aVal).localeCompare(String(bVal));
      }
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [data, sortKey, sortDir]);

  const totalPages = Math.max(1, Math.ceil(sortedData.length / pageSize));
  const paginatedData = sortedData.slice(
    currentPage * pageSize,
    (currentPage + 1) * pageSize
  );

  const handleSort = (key: string) => {
    if (sortKey === key) {
      setSortDir(sortDir === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
    setCurrentPage(0);
  };

  return (
    <div className="bg-gray-800 border border-gray-700/50 rounded-xl overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b border-gray-700">
              {columns.map((col) => (
                <th
                  key={col.key}
                  className={clsx(
                    "px-4 py-3 text-left text-xs font-semibold text-gray-400 uppercase tracking-wider",
                    col.sortable && "cursor-pointer select-none hover:text-gray-200",
                    col.className
                  )}
                  onClick={col.sortable ? () => handleSort(col.key) : undefined}
                  onKeyDown={col.sortable ? (e) => { if (e.key === "Enter" || e.key === " ") handleSort(col.key); } : undefined}
                  tabIndex={col.sortable ? 0 : undefined}
                  role={col.sortable ? "button" : undefined}
                  aria-sort={col.sortable && sortKey === col.key ? (sortDir === "asc" ? "ascending" : "descending") : undefined}
                >
                  <div className="flex items-center gap-1">
                    {col.label}
                    {col.sortable && sortKey === col.key && (
                      sortDir === "asc" ? (
                        <ChevronUp className="w-3.5 h-3.5" />
                      ) : (
                        <ChevronDown className="w-3.5 h-3.5" />
                      )
                    )}
                  </div>
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700/50">
            {paginatedData.length === 0 ? (
              <tr>
                <td
                  colSpan={columns.length}
                  className="px-4 py-12 text-center text-sm text-gray-500"
                >
                  {emptyMessage}
                </td>
              </tr>
            ) : (
              paginatedData.map((row, idx) => (
                <tr
                  key={rowKey ? rowKey(row) : idx}
                  onClick={onRowClick ? () => onRowClick(row) : undefined}
                  onKeyDown={onRowClick ? (e) => { if (e.key === "Enter" || e.key === " ") onRowClick(row); } : undefined}
                  tabIndex={onRowClick ? 0 : undefined}
                  role={onRowClick ? "button" : undefined}
                  className={clsx(
                    "transition-colors",
                    onRowClick
                      ? "cursor-pointer hover:bg-gray-700/40 focus:bg-gray-700/40 focus:outline-none focus:ring-1 focus:ring-primary/50"
                      : "hover:bg-gray-750/20"
                  )}
                >
                  {columns.map((col) => (
                    <td
                      key={col.key}
                      className={clsx(
                        "px-4 py-3 text-sm text-gray-300",
                        col.className
                      )}
                    >
                      {col.render
                        ? col.render(row)
                        : String(row[col.key] ?? "")}
                    </td>
                  ))}
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {sortedData.length > pageSize && (
        <nav aria-label="Table pagination" className="flex items-center justify-between px-4 py-3 border-t border-gray-700">
          <p className="text-xs text-gray-500">
            Showing {currentPage * pageSize + 1} to{" "}
            {Math.min((currentPage + 1) * pageSize, sortedData.length)} of{" "}
            {sortedData.length} results
          </p>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
              disabled={currentPage === 0}
              aria-label="Previous page"
              className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronLeft className="w-4 h-4" />
            </button>
            {Array.from({ length: totalPages }, (_, i) => i).map((page) => (
              <button
                key={page}
                onClick={() => setCurrentPage(page)}
                aria-label={`Page ${page + 1}`}
                aria-current={page === currentPage ? "page" : undefined}
                className={clsx(
                  "w-8 h-8 rounded-lg text-xs font-medium transition-colors",
                  page === currentPage
                    ? "bg-primary text-white"
                    : "text-gray-400 hover:text-white hover:bg-gray-700"
                )}
              >
                {page + 1}
              </button>
            ))}
            <button
              onClick={() =>
                setCurrentPage(Math.min(totalPages - 1, currentPage + 1))
              }
              disabled={currentPage >= totalPages - 1}
              aria-label="Next page"
              className="p-1.5 rounded-lg text-gray-400 hover:text-white hover:bg-gray-700 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
            >
              <ChevronRight className="w-4 h-4" />
            </button>
          </div>
        </nav>
      )}
    </div>
  );
}
