import { cn } from "@/lib/utils";

const STATUS_MAP: Record<string, string> = {
	up: "bg-emerald-500/15 text-emerald-400 ring-1 ring-emerald-500/30",
	down: "bg-red-500/15 text-red-400 ring-1 ring-red-500/30",
	degraded: "bg-amber-500/15 text-amber-400 ring-1 ring-amber-500/30",
	pending: "bg-cyan-500/15 text-cyan-400 ring-1 ring-cyan-500/30",
	maintenance: "bg-zinc-500/15 text-zinc-400 ring-1 ring-zinc-500/30",
	unknown: "bg-zinc-500/15 text-zinc-400 ring-1 ring-zinc-500/30",
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
				"inline-flex items-center rounded px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wider",
				STATUS_MAP[normalized] ?? STATUS_MAP.unknown,
				className,
			)}
		>
			{status ?? "unknown"}
		</span>
	);
}
