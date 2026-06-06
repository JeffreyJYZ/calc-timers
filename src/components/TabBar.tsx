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
			className="border-border bg-bg flex w-full items-end border-b px-2 pt-2 sm:px-4"
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
									? "border-border border-b-surface bg-surface text-text -mb-px border"
									: "text-text-muted hover:text-text"
							}`}
							aria-pressed={isActive}
						>
							<Icon size={16} aria-hidden />
							<span>{label}</span>
							{badge !== undefined && (
								<span
									className={`inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] font-semibold ${
										isActive
											? "bg-accent text-surface"
											: "bg-bg-soft text-text-muted"
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
