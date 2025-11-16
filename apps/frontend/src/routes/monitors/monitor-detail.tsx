import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Trash } from "lucide-react";
import { toast } from "sonner";
import { Hero } from "@/components/layout/Hero";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/Skeleton";
import { StatusPill } from "@/components/ui/StatusPill";
import {
	deleteMonitor,
	getMonitor,
	getMonitorHeartbeats,
	type Heartbeat,
	type Monitor,
} from "@/lib/monitors";
import type { RouterContext } from "@/router-context";

const formatTimestamp = (value?: number | null) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

const formatHeartbeatTimestamp = (value: number) => {
	return new Date(value).toLocaleString();
};

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) => {
	const route = createRoute({
		path: "/monitors/$monitorId",
		getParentRoute: () => parentRoute,
		component: MonitorDetailPage,
	});

	function MonitorDetailPage() {
		const { monitorId } = route.useParams();
		const navigate = useNavigate({ from: "/monitors/$monitorId" });
		const queryClient = useQueryClient();

		const monitorQuery = useQuery({
			queryKey: ["monitor", monitorId],
			queryFn: () => getMonitor(monitorId),
		});

		const heartbeatQuery = useQuery({
			enabled: Boolean(monitorQuery.data),
			queryKey: ["monitor", monitorId, "heartbeats"],
			queryFn: () => getMonitorHeartbeats(monitorId, 50),
		});

		const deleteMutation = useMutation({
			mutationFn: () => deleteMonitor(monitorId),
			onSuccess: async () => {
				toast.success("Monitor deleted");
				await Promise.all([
					queryClient.invalidateQueries({ queryKey: ["monitors"] }),
					queryClient.removeQueries({ queryKey: ["monitor", monitorId] }),
				]);
				navigate({ to: "/monitors" });
			},
			onError: (error: unknown) => {
				const message =
					error instanceof Error ? error.message : "Unable to delete monitor";
				toast.error(message);
			},
		});

		const handleDelete = () => {
			const confirmed = window.confirm(
				"Delete this monitor? Checks and history will stop immediately.",
			);
			if (!confirmed) {
				return;
			}
			void deleteMutation.mutateAsync();
		};

		const monitor = monitorQuery.data;
		const heartbeats = heartbeatQuery.data ?? [];

		const configSummary = useMemo(() => {
			if (!monitor) {
				return [];
			}
			return [
				{ label: "Interval", value: `${monitor.intervalS}s` },
				{ label: "Timeout", value: `${monitor.timeoutMs}ms` },
				{
					label: "Redirects",
					value: monitor.followRedirects ? "Follow" : "Do not follow",
				},
				{
					label: "TLS",
					value: monitor.verifyTls ? "Verify" : "Allow self-signed",
				},
				{
					label: "Status expectations",
					value:
						monitor.expectStatusLow && monitor.expectStatusHigh
							? `${monitor.expectStatusLow}–${monitor.expectStatusHigh}`
							: "Any 2xx",
				},
			];
		}, [monitor]);

		return (
			<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
				<div className="mx-auto max-w-6xl space-y-10">
					<Link
						to="/monitors"
						className="inline-flex items-center gap-2 text-sm text-[var(--text-muted)] transition hover:text-[var(--text-primary)]"
					>
						<ArrowLeft size={16} />
						Back to monitors
					</Link>

					{monitorQuery.isLoading ? (
						<div className="space-y-6">
							<Skeleton className="h-24" />
							<Skeleton className="h-64" />
						</div>
					) : monitorQuery.error instanceof Error ? (
						<div className="space-y-4">
							<p className="font-mono text-sm text-[var(--accent-red)]">
								{monitorQuery.error.message}
							</p>
							<Button type="button" onClick={() => monitorQuery.refetch()}>
								Try again
							</Button>
						</div>
					) : monitor ? (
						<>
							<Hero
								eyebrow="Monitor detail"
								title={monitor.name}
								description={monitor.url}
								actions={
									<div className="flex flex-wrap gap-3">
										<Link
											to="/monitors/$monitorId/edit"
											params={{ monitorId }}
										>
											<Button>Edit monitor</Button>
										</Link>
										<Button
											type="button"
											variant="destructive"
											onClick={handleDelete}
											disabled={deleteMutation.isPending}
											className="flex items-center gap-2"
										>
											<Trash size={16} />
											{deleteMutation.isPending ? "Deleting…" : "Delete"}
										</Button>
									</div>
								}
								sideContent={
									<div className="rounded-2xl border border-white/10 bg-black/30 p-4">
                                        <div className="space-y-2">
											<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Status
											</p>
											<StatusPill status={monitor.currentStatus} />
											<p className="text-xs text-[var(--text-muted)]">
												Last check: {formatTimestamp(monitor.lastCheckedAtTs)}
											</p>
										</div>
									</div>
								}
							/>

							<section className="grid gap-6 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
								<SectionCard title="Configuration">
									<dl className="grid grid-cols-1 gap-4 text-sm font-mono text-[var(--text-muted)]">
										{configSummary.map((item) => (
											<div key={item.label}>
												<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
													{item.label}
												</dt>
												<dd className="text-base text-[var(--text-primary)]">
													{item.value}
												</dd>
											</div>
										))}
										<div>
											<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
												Created
											</dt>
											<dd className="text-base text-[var(--text-primary)]">
												{formatTimestamp(monitor.createdAt)}
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
												{monitor.enabled ? "Active" : "Disabled"}
											</dd>
										</div>
									</dl>
								</SectionCard>
								<SectionCard
									title="Recent heartbeats"
									description="Latest executions streamed from the worker ticker"
								>
									{heartbeatQuery.isLoading ? (
										<div className="space-y-3">
											<Skeleton className="h-12" />
											<Skeleton className="h-12" />
											<Skeleton className="h-12" />
										</div>
									) : heartbeatQuery.error instanceof Error ? (
										<div className="space-y-4">
											<p className="font-mono text-sm text-[var(--accent-red)]">
												{heartbeatQuery.error.message}
											</p>
											<Button
												type="button"
												variant="secondary"
												onClick={() => heartbeatQuery.refetch()}
											>
												Retry
											</Button>
										</div>
									) : heartbeats.length === 0 ? (
										<p className="text-sm text-[var(--text-muted)]">
											No heartbeats yet. Checks will appear here after the first
											run.
										</p>
									) : (
										<div className="space-y-3">
											{heartbeats.map((heartbeat) => (
												<HeartbeatRow key={`${heartbeat.monitorId}-${heartbeat.ts}`} heartbeat={heartbeat} />
											))}
										</div>
									)}
								</SectionCard>
							</section>
						</>
					) : null}
				</div>
			</main>
		);
	}

	return route;
};

function HeartbeatRow({ heartbeat }: { heartbeat: Heartbeat }) {
	const ok = heartbeat.ok === 1;
	return (
		<div className="rounded-2xl border border-white/10 bg-black/30 px-4 py-3 text-sm">
			<div className="flex flex-wrap items-center justify-between gap-3">
				<div className="flex items-center gap-3">
					<span
						className={
							ok
								? "h-2 w-2 rounded-full bg-[var(--accent-green)]"
								: "h-2 w-2 rounded-full bg-[var(--accent-red)]"
						}
					/>
					<p className="font-mono text-[var(--text-primary)]">
						{ok ? "OK" : "FAIL"}
					</p>
					<p className="font-mono text-xs text-[var(--text-muted)]">
						{heartbeat.region ?? "Unknown POP"}
					</p>
				</div>
				<p className="font-mono text-xs text-[var(--text-muted)]">
					{formatHeartbeatTimestamp(heartbeat.ts)}
				</p>
			</div>
			<div className="mt-2 grid grid-cols-2 gap-3 text-xs font-mono text-[var(--text-muted)] md:grid-cols-4">
				<div>
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						Status
					</p>
					<p className="text-[var(--text-primary)]">
						{heartbeat.code ?? "—"}
					</p>
				</div>
				<div>
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						RTT
					</p>
					<p className="text-[var(--text-primary)]">
						{heartbeat.rttMs ? `${heartbeat.rttMs}ms` : "—"}
					</p>
				</div>
				<div className="md:col-span-2">
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						Error
					</p>
					<p className="text-[var(--text-primary)] break-all">
						{heartbeat.err ?? "—"}
					</p>
				</div>
			</div>
		</div>
	);
}
