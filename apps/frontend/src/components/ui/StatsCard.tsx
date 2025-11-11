import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type StatsItem = {
	label: ReactNode;
	value: ReactNode;
	hint?: ReactNode;
	tone?: string;
};

type StatsCardProps = StatsItem & {
	className?: string;
};

export function StatsCard({
	label,
	value,
	hint,
	tone,
	className,
}: StatsCardProps) {
	return (
		<div
			className={cn(
				"rounded-3xl border border-white/10 bg-black/30 px-5 py-4",
				className,
			)}
		>
			<p className="text-xs uppercase tracking-[0.4em] text-[var(--text-soft)]">
				{label}
			</p>
			<p className={cn("mt-2 text-2xl font-semibold", tone)}>{value}</p>
			{hint ? <p className="text-xs text-[var(--text-muted)]">{hint}</p> : null}
		</div>
	);
}

type StatsGridProps = {
	items: StatsItem[];
	className?: string;
	cardClassName?: string;
};

export function StatsGrid({ items, className, cardClassName }: StatsGridProps) {
	return (
		<div className={cn("grid gap-4 sm:grid-cols-2", className)}>
			{items.map((item) => (
				<StatsCard
					key={String(item.label)}
					label={item.label}
					value={item.value}
					hint={item.hint}
					tone={item.tone}
					className={cardClassName}
				/>
			))}
		</div>
	);
}
