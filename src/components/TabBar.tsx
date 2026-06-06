import { Calculator, Timer } from "lucide-react";

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
			className="flex w-full items-end border-b border-[var(--color-border)] bg-[var(--color-bg)] px-2 pt-2 sm:px-4"
		>
			<div className="flex flex-1 gap-1">
				{items.map(({ id, label, icon: Icon, badge }) => {
					const isActive = id === active;
					return (
						<button
							key={id}
							onClick={() => onChange(id)}
							className={`btn-press relative flex items-center gap-2 rounded-t-xl px-4 py-2.5 text-sm font-medium transition-colors ${
								isActive
									? "-mb-px border border-[var(--color-border)] border-b-[var(--color-surface)] bg-[var(--color-surface)] text-[var(--color-text)]"
									: "text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
							}`}
							aria-pressed={isActive}
						>
							<Icon size={16} aria-hidden />
							<span>{label}</span>
							{badge !== undefined && (
								<span
									className={`inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] font-semibold ${
										isActive
											? "bg-[var(--color-accent)] text-[#fbf9f3]"
											: "bg-[var(--color-bg-soft)] text-[var(--color-text-muted)]"
									}`}
								>
									{badge}
								</span>
							)}
						</button>
					);
				})}
			</div>
		</nav>
	);
}
