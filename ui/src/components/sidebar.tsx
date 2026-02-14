"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Megaphone,
  Image,
  Activity,
  Settings,
  Zap,
  GitBranch,
  Palette,
  Database,
  FlaskConical,
  Shield,
  CreditCard,
  HeartPulse,
  Users,
} from "lucide-react";
import clsx from "clsx";

const navItems = [
  { href: "/", label: "Dashboard", icon: LayoutDashboard },
  { href: "/campaigns", label: "Campaigns", icon: Megaphone },
  { href: "/creatives", label: "Creatives", icon: Image },
  { href: "/journeys", label: "Journeys", icon: GitBranch },
  { href: "/dco", label: "DCO Templates", icon: Palette },
  { href: "/cdp", label: "CDP Integrations", icon: Database },
  { href: "/experiments", label: "Experiments", icon: FlaskConical },
  { href: "/monitoring", label: "Monitoring", icon: Activity },
  { href: "/users", label: "Users", icon: Users },
  { href: "/platform", label: "Platform", icon: Shield },
  { href: "/billing", label: "Billing", icon: CreditCard },
  { href: "/ops", label: "Operations", icon: HeartPulse },
  { href: "/settings", label: "Settings", icon: Settings },
];

export default function Sidebar() {
  const pathname = usePathname();

  const isActive = (href: string) => {
    if (href === "/") return pathname === "/";
    return pathname.startsWith(href);
  };

  return (
    <aside className="fixed left-0 top-0 z-40 h-screen w-64 bg-gray-900 border-r border-gray-800 flex flex-col">
      {/* Branding */}
      <div className="flex items-center gap-3 px-6 py-5 border-b border-gray-800">
        <div className="flex items-center justify-center w-9 h-9 rounded-lg bg-primary">
          <Zap className="w-5 h-5 text-white" />
        </div>
        <div>
          <h1 className="text-lg font-bold text-white leading-tight">
            Campaign
          </h1>
          <p className="text-xs text-primary-400 font-medium -mt-0.5">
            Express
          </p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-1 overflow-y-auto">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = isActive(item.href);
          return (
            <Link
              key={item.href}
              href={item.href}
              className={clsx(
                "flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors",
                active
                  ? "bg-primary/10 text-primary-400 border border-primary/20"
                  : "text-gray-400 hover:text-white hover:bg-gray-800/60"
              )}
            >
              <Icon
                className={clsx("w-5 h-5", active ? "text-primary-400" : "")}
              />
              {item.label}
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="px-4 py-3 border-t border-gray-800">
        <div className="flex items-center gap-2 text-xs text-gray-500">
          <div className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
          System Online
        </div>
      </div>
    </aside>
  );
}
