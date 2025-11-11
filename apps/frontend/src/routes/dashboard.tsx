import { useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import { RefreshCcw } from "lucide-react";
import { Hero } from "@/components/layout/Hero";
import { Button } from "@/components/ui/button";
import { getMonitors, type Monitor } from "@/lib/monitors";
import type { RouterContext } from "@/router-context";
import { StatsGrid } from "@/components/ui/StatsCard";
import { SectionCard } from "@/components/layout/SectionCard";
import { Skeleton } from "@/components/ui/Skeleton";
import { StatusPill } from "@/components/ui/StatusPill";

const ORG_ID = "zimr7nsz8gj0nxsgqktogm4v";

const formatTimestamp = (value?: number | null) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

function DashboardPage() {
	const { data, isLoading, isFetching, error } = useQuery<Monitor[]>({
		queryKey: ["monitors", ORG_ID],
		queryFn: () => getMonitors(ORG_ID),
	});

	const monitors = data ?? [];
	const total = monitors.length;
	const upCount = monitors.filter(
		(monitor) => monitor.current_status === "up",
	).length;
	const downCount = monitors.filter(
		(monitor) => monitor.current_status === "down",
	).length;
	const maintenanceCount = monitors.filter(
		(monitor) => monitor.current_status === "maintenance",
	).length;
	const unknownCount = Math.max(
		total - (upCount + downCount + maintenanceCount),
		0,
	);

	const quickStats = [
		{ label: "Up", value: upCount, tone: "text-emerald-300" },
		{ label: "Down", value: downCount, tone: "text-[var(--accent-red)]" },
		{ label: "Maintenance", value: maintenanceCount, tone: "text-amber-200" },
		{ label: "Unknown", value: unknownCount, tone: "text-[var(--text-soft)]" },
	];

	const recentMonitors = [...monitors]
		.sort((a, b) => (b.last_checked_at_ts ?? 0) - (a.last_checked_at_ts ?? 0))
		.slice(0, 8);

	const logEntries = recentMonitors.length
		? recentMonitors.map((monitor) => {
				const status = monitor.current_status ?? "unknown";
				const timestamp = formatTimestamp(monitor.last_checked_at_ts);
				return `[${status.toUpperCase()}] ${monitor.name} @ ${timestamp}`;
			})
		: [
				"[scheduler] worker idle — waiting for monitors",
				"[storage] D1 heartbeat OK",
				"[auth] CF Access enforcing session",
			];

	return (
		<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto max-w-6xl space-y-8">
				<Hero
					eyebrow="Saavy Uptime"
					title="Cloudflare-native uptime cockpit"
					description="Authenticated through Cloudflare Access, scheduled inside Workers. Use this view to confirm the monitors you deployed are still healthy."
					actions={
						<>
							<Link to="/monitors/new">
								<Button>Add monitor</Button>
							</Link>
							<Link to="/monitors">
								<Button variant="secondary">Inventory</Button>
							</Link>
						</>
					}
					sideContent={
						<StatsGrid
							items={quickStats.map((stat) => ({
								...stat,
								value: isLoading ? "…" : stat.value,
							}))}
							cardClassName="bg-black/30 border-white/10"
						/>
					}
				/>

				<section className="grid gap-6 lg:grid-cols-[minmax(0,1.5fr)_minmax(0,0.8fr)]">
					<SectionCard
						title="Monitor inventory"
						description={`Showing ${recentMonitors.length} of ${total || 0} monitors`}
						actions={
							<Link to="/monitors">
								<Button variant="ghost">Open monitors view</Button>
							</Link>
						}
					>
						{isLoading ? (
							<div className="space-y-3">
								{["alpha", "beta", "gamma", "delta"].map((token) => (
									<Skeleton
										key={`dashboard-skeleton-${token}`}
										className="h-14"
									/>
								))}
							</div>
						) : error instanceof Error ? (
							<p className="font-mono text-sm text-[var(--accent-red)]">
								{error.message}
							</p>
						) : recentMonitors.length === 0 ? (
							<p className="text-sm text-[var(--text-muted)]">
								No monitors yet. Add one to start collecting uptime signals.
							</p>
						) : (
							<div className="divide-y divide-white/5">
								<div className="grid grid-cols-[minmax(0,1.4fr)_minmax(0,0.6fr)_minmax(0,0.6fr)_minmax(0,0.8fr)] gap-4 px-3 py-2 text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
									<span>Name</span>
									<span>Status</span>
									<span>Interval · Timeout</span>
									<span>Last check</span>
								</div>
								{recentMonitors.map((monitor) => (
									<div
										key={monitor.id}
										className="grid grid-cols-[minmax(0,1.4fr)_minmax(0,0.6fr)_minmax(0,0.6fr)_minmax(0,0.8fr)] items-center gap-4 px-3 py-3 text-sm"
									>
										<div className="truncate font-medium">{monitor.name}</div>
										<StatusPill status={monitor.current_status} />
										<p className="font-mono text-xs text-[var(--text-muted)]">
											{monitor.interval_s}s · {monitor.timeout_ms}ms
										</p>
										<p className="font-mono text-xs text-[var(--text-muted)]">
											{formatTimestamp(monitor.last_checked_at_ts)}
										</p>
									</div>
								))}
							</div>
						)}
					</SectionCard>

					<SectionCard
						title="System log"
						description="Latest events from this deployment"
						actions={
							isFetching ? (
								<RefreshCcw
									size={16}
									className="animate-spin text-[var(--accent)]"
								/>
							) : null
						}
					>
						<div className="space-y-3 font-mono text-sm text-[var(--text-muted)]">
							{logEntries.map((entry) => (
								<p
									key={entry}
									className="rounded-2xl border border-white/5 bg-black/30 px-4 py-2"
								>
									{entry}
								</p>
							))}
						</div>
					</SectionCard>
				</section>
			</div>
		</main>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/",
		component: DashboardPage,
		getParentRoute: () => parentRoute,
	});
