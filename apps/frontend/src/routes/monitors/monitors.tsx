import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState, useId } from "react";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import { Database, RefreshCcw, Sparkles, Wrench, ArrowUpRight } from "lucide-react";
import { toast } from "sonner";
import { Hero } from "@/components/layout/Hero";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/Skeleton";
import { StatsGrid } from "@/components/ui/StatsCard";
import { StatusPill } from "@/components/ui/StatusPill";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { getMonitors, seedMonitors, type Monitor } from "@/lib/monitors";
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

	const [searchTerm, setSearchTerm] = useState("");
	const [kindFilter, setKindFilter] = useState("all");
	const [pageSize, setPageSize] = useState(20);
	const [currentPage, setCurrentPage] = useState(0);
	const searchFieldId = useId();
	const kindFieldId = useId();

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

	const monitorKinds = useMemo(() => {
		const kinds = Array.from(new Set(monitors.map((monitor) => monitor.kind)));
		return kinds.sort((a, b) => a.localeCompare(b));
	}, [monitors]);

	const filteredMonitors = useMemo(() => {
		const normalizedSearch = searchTerm.trim().toLowerCase();
		return monitors.filter((monitor) => {
			const matchesKind =
				kindFilter === "all"
					? true
					: monitor.kind.toLowerCase() === kindFilter.toLowerCase();
			const matchesSearch = normalizedSearch
				? [monitor.name, monitor.url].some((value) =>
						value.toLowerCase().includes(normalizedSearch),
				  )
				: true;
			return matchesKind && matchesSearch;
		});
	}, [monitors, kindFilter, searchTerm]);

	const sortedFilteredMonitors = useMemo(
		() =>
			[...filteredMonitors].sort(
				(a, b) => (b.lastCheckedAtTs ?? 0) - (a.lastCheckedAtTs ?? 0),
			),
		[filteredMonitors],
	);

	const totalPages = Math.max(
		1,
		Math.ceil(sortedFilteredMonitors.length / pageSize) || 1,
	);
	const safePage = Math.min(currentPage, totalPages - 1);
	const pageStart = safePage * pageSize;
	const pageEnd = Math.min(pageStart + pageSize, sortedFilteredMonitors.length);
	const paginatedMonitors = sortedFilteredMonitors.slice(pageStart, pageEnd);

	useEffect(() => {
		setCurrentPage(0);
	}, [searchTerm, kindFilter, pageSize]);

	useEffect(() => {
		if (currentPage !== safePage) {
			setCurrentPage(safePage);
		}
	}, [currentPage, safePage]);

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

	const queryClient = useQueryClient();
	const seedMutation = useMutation({
		mutationFn: () => seedMonitors(),
		onSuccess: ({ created, failed }) => {
			const description =
				failed > 0
					? `Created ${created} monitors (${failed} failed)`
					: `Created ${created} monitors`;
			toast.success("Seed complete", { description });
			queryClient.invalidateQueries({ queryKey: ["monitors"] });
		},
		onError: (err: unknown) => {
			const message =
				err instanceof Error ? err.message : "Unable to seed monitors";
			toast.error(message);
		},
	});

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
								<>
									<Button
										type="button"
										variant="outline"
										onClick={() => reconcileMutation.mutate()}
										disabled={reconcileMutation.isPending}
										className="flex items-center gap-2"
									>
										<Wrench size={16} />
										{reconcileMutation.isPending
											? "Bootstrapping…"
											: "Warm ticker"}
									</Button>
									<Button
										type="button"
										variant="outline"
										onClick={() => seedMutation.mutate()}
										disabled={seedMutation.isPending}
										className="flex items-center gap-2"
									>
										<Database size={16} />
										{seedMutation.isPending ? "Seeding…" : "Seed monitors"}
									</Button>
								</>
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
					) : monitors.length === 0 ? (
						<div className="space-y-4 py-12 text-center">
							<p className="text-base text-[var(--text-muted)]">
								No monitors yet. Create one to start tracking uptime.
							</p>
							<Link to="/monitors/new">
								<Button>Launch first monitor</Button>
							</Link>
						</div>
					) : (
						<div className="space-y-6">
							<div className="flex flex-col gap-4 border-b border-white/5 pb-6 lg:flex-row lg:items-end lg:justify-between">
								<div className="flex-1 space-y-2">
									<Label
										htmlFor={searchFieldId}
										className="text-xs font-semibold uppercase tracking-[0.3em] text-[var(--text-soft)]"
									>
										Search
									</Label>
									<Input
										id={searchFieldId}
										placeholder="Search by name or URL"
										value={searchTerm}
										onChange={(event) => setSearchTerm(event.target.value)}
									/>
								</div>
								<div className="space-y-2">
									<Label
										htmlFor={kindFieldId}
										className="text-xs font-semibold uppercase tracking-[0.3em] text-[var(--text-soft)]"
									>
										Monitor type
									</Label>
									<Select
										value={kindFilter}
										onValueChange={(value) => setKindFilter(value)}
									>
										<SelectTrigger id={kindFieldId} className="min-w-[200px]">
											<SelectValue placeholder="All monitors" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="all">All monitors</SelectItem>
											{monitorKinds.map((kind) => (
												<SelectItem key={kind} value={kind}>
													{kind.toUpperCase()}
												</SelectItem>
											))}
										</SelectContent>
									</Select>
								</div>
							</div>

							{filteredMonitors.length === 0 ? (
								<div className="space-y-4 py-12 text-center">
									<p className="text-base text-[var(--text-muted)]">
										No monitors match your filters. Try adjusting the search or
										type filter.
									</p>
									<Button
										type="button"
										variant="secondary"
										onClick={() => {
											setSearchTerm("");
											setKindFilter("all");
										}}
									>
										Clear filters
									</Button>
								</div>
							) : (
								<>
									<div className="space-y-4">
										{paginatedMonitors.map((monitor) => (
											<Link
												key={monitor.id}
												to="/monitors/$monitorId"
												params={{ monitorId: monitor.id }}
												className="block rounded-[28px] border border-white/10 bg-white/[0.01] p-4 transition hover:border-white/30 hover:bg-white/[0.03]"
											>
												<div className="grid gap-6 md:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]">
													<div className="space-y-3">
														<div className="flex flex-wrap items-center gap-3">
															<h3 className="text-xl font-semibold">
																{monitor.name}
															</h3>
															<StatusPill status={monitor.currentStatus} />
															<span className="flex items-center gap-1 text-xs font-semibold uppercase tracking-[0.3em] text-[var(--text-soft)]">
																View
																<ArrowUpRight size={12} />
															</span>
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
											</Link>
										))}
									</div>
									<div className="flex flex-col gap-4 border-t border-white/5 pt-4 text-sm text-[var(--text-muted)] md:flex-row md:items-center md:justify-between">
										<p>
											Showing{" "}
											<strong className="text-[var(--text-primary)]">
												{filteredMonitors.length === 0
													? 0
													: `${pageStart + 1}-${pageEnd}`}
											</strong>{" "}
											of{" "}
											<strong className="text-[var(--text-primary)]">
												{filteredMonitors.length}
											</strong>{" "}
											monitors
										</p>
										<div className="flex flex-col gap-3 md:flex-row md:items-center md:gap-6">
											<div className="flex items-center gap-2">
												<span>Rows per page</span>
												<Select
													value={pageSize.toString()}
													onValueChange={(value) => {
														setPageSize(Number(value));
													}}
												>
													<SelectTrigger size="sm">
														<SelectValue />
													</SelectTrigger>
													<SelectContent>
														<SelectItem value="10">10</SelectItem>
														<SelectItem value="20">20</SelectItem>
														<SelectItem value="50">50</SelectItem>
													</SelectContent>
												</Select>
											</div>
											<div className="flex items-center gap-2">
												<Button
													type="button"
													variant="ghost"
													size="sm"
													onClick={() =>
														setCurrentPage((prev) => Math.max(prev - 1, 0))
													}
													disabled={safePage === 0}
												>
													Prev
												</Button>
												<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
													Page {safePage + 1} / {totalPages}
												</p>
												<Button
													type="button"
													variant="ghost"
													size="sm"
													onClick={() =>
														setCurrentPage((prev) =>
															Math.min(prev + 1, totalPages - 1),
														)
													}
													disabled={safePage >= totalPages - 1}
												>
													Next
												</Button>
											</div>
										</div>
									</div>
								</>
							)}
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
