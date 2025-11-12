import { useMutation, useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import { RefreshCcw, Sparkles, Wrench } from "lucide-react";
import { toast } from "sonner";
import { Hero } from "@/components/layout/Hero";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/Skeleton";
import { StatsGrid } from "@/components/ui/StatsCard";
import { StatusPill } from "@/components/ui/StatusPill";
import { getMonitors, type Monitor } from "@/lib/monitors";
import { reconcileTickers } from "@/lib/ticker";
import type { RouterContext } from "@/router-context";

const formatTimestamp = (value?: number | null) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

function MonitorsPage() {
	const { data, isLoading, error, refetch } = useQuery<Monitor[]>({
		queryKey: ["monitors"],
		queryFn: () => getMonitors(),
	});

	const tickerAdminEnabled =
		import.meta.env.DEV ||
		["1", "true"].includes(
			(import.meta.env.VITE_ENABLE_TICKER_ADMIN ?? "").toLowerCase(),
		);

	const reconcileMutation = useMutation({
		mutationFn: () => reconcileTickers(),
		onSuccess: (summary) => {
			toast.success("Ticker bootstrap complete", {
				description: `Bootstrapped ${summary.bootstrapped}/${summary.organizations} orgs`,
			});
			void refetch();
		},
		onError: (err: unknown) => {
			const message =
				err instanceof Error ? err.message : "Unable to reconcile tickers";
			toast.error(message);
		},
	});

	const monitors = data ?? [];
	const total = monitors.length;
	const enabled = monitors.filter((monitor) => Boolean(monitor.enabled)).length;
	const failing = monitors.filter(
		(monitor) => monitor.currentStatus === "down",
	).length;
	const averageInterval = monitors.length
		? Math.round(
				monitors.reduce(
					(sum, monitor) => sum + Number(monitor.intervalS ?? 0),
					0,
				) / monitors.length,
			)
		: null;
	const mostRecentCheck = monitors.reduce(
		(latest, monitor) => Math.max(latest, monitor.lastCheckedAtTs ?? 0),
		0,
	);

	const sortedMonitors = [...monitors].sort(
		(a, b) => (b.lastCheckedAtTs ?? 0) - (a.lastCheckedAtTs ?? 0),
	);

	const overviewCards = [
		{
			label: "Total monitors",
			value: isLoading ? "…" : total,
			meta: `${enabled} enabled`,
		},
		{
			label: "Down",
			value: isLoading ? "…" : failing,
			meta: "requires attention",
		},
		{
			label: "Average interval",
			value: averageInterval ? `${averageInterval}s` : "—",
			meta: "fleet cadence",
		},
		{
			label: "Last check",
			value: mostRecentCheck ? formatTimestamp(mostRecentCheck) : "—",
			meta: "latest sample",
		},
	];

	return (
		<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto max-w-6xl space-y-10">
				<Hero
					eyebrow="Monitor Fleet"
					title="Live monitor inventory for Saavy Uptime."
					description="Review the checks you’ve already deployed, refresh status from the worker, and stay focused on a lean uptime MVP."
					actions={
						<>
							<Link to="/monitors/new">
								<Button>New monitor</Button>
							</Link>
							<Button
								type="button"
								variant="secondary"
								onClick={() => refetch()}
								disabled={isLoading}
								className="flex items-center gap-2"
							>
								<RefreshCcw size={16} />
								Refresh
							</Button>
							{tickerAdminEnabled ? (
								<Button
									type="button"
									variant="outline"
									onClick={() => reconcileMutation.mutate()}
									disabled={reconcileMutation.isPending}
									className="flex items-center gap-2"
								>
									<Wrench size={16} />
									{reconcileMutation.isPending ? "Bootstrapping…" : "Warm ticker"}
								</Button>
							) : null}
						</>
					}
					sideContent={
						<div className="space-y-4">
							<StatsGrid
								items={overviewCards}
								cardClassName="bg-white/5 border-white/10"
							/>
							<div className="flex items-center gap-3 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm">
								<Sparkles size={18} className="text-[var(--accent)]" />
								<span className="text-[var(--text-muted)]">
									{enabled
										? `${enabled} monitors currently online`
										: "Waiting for your first monitor"}
								</span>
							</div>
						</div>
					}
				/>

				<SectionCard
					title="Monitor inventory"
					actions={
						<>
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onClick={() => refetch()}
								className="flex items-center gap-2"
							>
								<RefreshCcw size={14} />
								Sync
							</Button>
							<Link to="/monitors/new">
								<Button size="sm">Add</Button>
							</Link>
						</>
					}
				>
					{isLoading ? (
						<div className="space-y-4">
							{["alpha", "beta", "gamma"].map((token) => (
								<Skeleton key={`monitors-skeleton-${token}`} className="h-24" />
							))}
						</div>
					) : error instanceof Error ? (
						<div className="space-y-4">
							<p className="font-mono text-sm text-[var(--accent-red)]">
								{error.message}
							</p>
							<Button type="button" onClick={() => refetch()}>
								Try again
							</Button>
						</div>
					) : sortedMonitors.length === 0 ? (
						<div className="space-y-4 py-12 text-center">
							<p className="text-base text-[var(--text-muted)]">
								No monitors yet. Create one to start tracking uptime.
							</p>
							<Link to="/monitors/new">
								<Button>Launch first monitor</Button>
							</Link>
						</div>
					) : (
						<div className="divide-y divide-white/5">
							{sortedMonitors.map((monitor) => (
								<div
									key={monitor.id}
									className="grid gap-6 py-6 md:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)] px-3 md:px-6"
								>
									<div className="space-y-3">
										<div className="flex flex-wrap items-center gap-3">
											<h3 className="text-xl font-semibold">{monitor.name}</h3>
											<StatusPill status={monitor.currentStatus} />
										</div>
										<p className="font-mono text-sm text-[var(--text-muted)] break-all">
											{monitor.url}
										</p>
										<p className="font-mono text-xs text-[var(--text-soft)]">
											Created {formatTimestamp(monitor.createdAt)}
										</p>
									</div>
									<dl className="grid grid-cols-2 gap-4 text-sm font-mono text-[var(--text-muted)]">
										<div>
											<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Interval
											</dt>
											<dd className="text-base text-[var(--text-primary)]">
												{monitor.intervalS}s
											</dd>
										</div>
										<div>
											<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Timeout
											</dt>
											<dd className="text-base text-[var(--text-primary)]">
												{monitor.timeoutMs}ms
											</dd>
										</div>
										<div>
											<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Last check
											</dt>
											<dd className="text-base text-[var(--text-primary)]">
												{formatTimestamp(monitor.lastCheckedAtTs)}
											</dd>
										</div>
										<div>
											<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Enabled
											</dt>
											<dd
												className={
													monitor.enabled
														? "text-[var(--accent-green)]"
														: "text-[var(--accent-red)]"
												}
											>
												{monitor.enabled ? "Yes" : "No"}
											</dd>
										</div>
									</dl>
								</div>
							))}
						</div>
					)}
				</SectionCard>
			</div>
		</main>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/monitors",
		component: MonitorsPage,
		getParentRoute: () => parentRoute,
	});
