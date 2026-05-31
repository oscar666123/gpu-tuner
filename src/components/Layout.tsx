import type { LucideIcon } from "lucide-react";
import type { GPUInfo } from "../lib/types";

export interface NavItem<T extends string> {
  key: T;
  label: string;
  icon: LucideIcon;
}

interface LayoutProps<T extends string> {
  navItems: NavItem<T>[];
  activeKey: T;
  onNavigate: (key: T) => void;
  gpus: GPUInfo[];
  children: React.ReactNode;
}

export default function Layout<T extends string>({ navItems, activeKey, onNavigate, gpus, children }: LayoutProps<T>) {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">GT</div>
          <div>
            <h1>GPU Tuner</h1>
            <p className="brand-author">by oscar666123</p>
            <p>{gpus.length > 0 ? `${gpus.length} NVIDIA GPU` : "NVIDIA Control"}</p>
          </div>
        </div>
        <nav className="nav">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                className={item.key === activeKey ? "nav-item active" : "nav-item"}
                key={item.key}
                type="button"
                onClick={() => onNavigate(item.key)}
              >
                <Icon size={18} />
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
        <div className="sidebar-note">
          <span>Admin mode required for writes</span>
        </div>
      </aside>
      <main className="content">{children}</main>
    </div>
  );
}
