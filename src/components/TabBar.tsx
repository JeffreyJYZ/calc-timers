import { Calculator, Snowflake, Timer } from "lucide-react";

export type Tab = "calc" | "timers";

interface Props {
  active: Tab;
  onChange: (tab: Tab) => void;
  timerCount: number;
}

export function TabBar({ active, onChange, timerCount }: Props) {
  const items: { id: Tab; label: string; icon: typeof Calculator; badge?: number }[] = [
    { id: "calc", label: "Calculator", icon: Calculator },
    {
      id: "timers",
      label: "Timers",
      icon: Timer,
      badge: timerCount > 0 ? timerCount : undefined,
    },
  ];
  return (
    <nav
      aria-label="Main"
      className="glass sticky top-0 z-20 flex w-full items-center border-b border-[var(--color-border)]/40 px-2 py-2 backdrop-blur-xl"
    >
      <div className="flex items-center gap-2 pr-3 pl-1 text-[var(--color-frost)]">
        <Snowflake size={16} className="opacity-80" />
      </div>
      <div className="flex flex-1 gap-1">
        {items.map(({ id, label, icon: Icon, badge }) => {
          const isActive = id === active;
          return (
            <button
              key={id}
              onClick={() => onChange(id)}
              className={`btn-press relative flex flex-1 items-center justify-center gap-2 rounded-xl px-4 py-2.5 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-[var(--color-accent)]/15 text-[var(--color-frost)]"
                  : "text-white/50 hover:bg-white/5 hover:text-white/80"
              }`}
              aria-pressed={isActive}
            >
              <Icon size={18} aria-hidden />
              <span>{label}</span>
              {badge !== undefined && (
                <span
                  className={`ml-1 inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] font-bold ${
                    isActive
                      ? "bg-[var(--color-accent)] text-[#02101e]"
                      : "bg-white/10 text-white/70"
                  }`}
                >
                  {badge}
                </span>
              )}
              {isActive && (
                <span
                  className="absolute inset-x-6 -bottom-[5px] h-[3px] rounded-full bg-gradient-to-r from-[var(--color-accent)] to-[var(--color-accent-2)] shadow-[0_0_12px_var(--color-accent)]"
                  aria-hidden
                />
              )}
            </button>
          );
        })}
      </div>
    </nav>
  );
}
