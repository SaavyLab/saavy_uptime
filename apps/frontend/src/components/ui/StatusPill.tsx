import { cn } from "@/lib/utils";

const STATUS_MAP: Record<string, string> = {
	up: "border border-emerald-400/30 bg-emerald-400/10 text-emerald-200",
	down: "border border-[var(--accent-red)]/40 bg-[var(--accent-red)]/15 text-[var(--accent-red)]",
	degraded: "border border-amber-300/40 bg-amber-300/15 text-amber-200",
	maintenance:
		"border border-[var(--accent)]/40 bg-[var(--accent)]/15 text-[var(--accent-strong)]",
	unknown: "border border-white/20 bg-white/10 text-[var(--text-primary)]",
};

type StatusPillProps = {
	status?: string | null;
	className?: string;
};

export function StatusPill({ status, className }: StatusPillProps) {
	const normalized = (status ?? "unknown").toLowerCase();
	return (
		<span
			className={cn(
				"w-fit rounded-full px-3 py-1 text-xs font-semibold uppercase",
				STATUS_MAP[normalized] ?? STATUS_MAP.unknown,
				className,
			)}
		>
			{status ?? "unknown"}
		</span>
	);
}
