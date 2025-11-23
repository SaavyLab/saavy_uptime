import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { Trash } from "lucide-react";
import { toast } from "sonner";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/Skeleton";
import { StatusPill } from "@/components/ui/StatusPill";
import {
	deleteMonitor,
	getMonitor,
	getMonitorHeartbeats,
	type HeartbeatSample,
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
			queryFn: () =>
				getMonitorHeartbeats(monitorId, {
					limit: 100,
					windowHours: 24,
				}),
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
		const heartbeatResponse = heartbeatQuery.data;
		const heartbeats = heartbeatResponse?.items ?? [];
		const heartbeatWindow = heartbeatResponse?.window;

		return (
			<div className="space-y-8">
				<div className="flex items-center justify-between">
					<div className="flex items-center gap-2 text-sm text-muted-foreground">
						<Link
							to="/monitors"
							className="hover:text-foreground transition-colors"
						>
							Monitors
						</Link>
						<span>/</span>
						<span className="text-foreground font-medium">
							{monitor?.name ?? "Loading..."}
						</span>
					</div>
				</div>

				{monitorQuery.isLoading ? (
					<div className="space-y-6">
						<Skeleton className="h-24" />
						<Skeleton className="h-64" />
					</div>
				) : monitorQuery.error instanceof Error ? (
					<div className="space-y-4">
						<p className="font-mono text-sm text-destructive">
							{monitorQuery.error.message}
						</p>
						<Button type="button" onClick={() => monitorQuery.refetch()}>
							Try again
						</Button>
					</div>
				) : monitor ? (
					<>
						<div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
							<div className="space-y-1">
								<div className="flex items-center gap-3">
									<h1 className="text-2xl font-bold tracking-tight">
										{monitor.name}
									</h1>
									<StatusPill status={monitor.status} />
								</div>
								<p className="text-muted-foreground font-mono text-sm">
									{monitor.config.url}
								</p>
							</div>
							<div className="flex items-center gap-2">
								<Link to="/monitors/$monitorId/edit" params={{ monitorId }}>
									<Button variant="outline">Edit</Button>
								</Link>
								<Button
									type="button"
									variant="destructive"
									onClick={handleDelete}
									disabled={deleteMutation.isPending}
									size="icon"
								>
									<Trash size={16} />
								</Button>
							</div>
						</div>

						<div className="grid gap-4 md:grid-cols-4">
							<div className="rounded-xl border border-border bg-muted/20 p-4">
								<div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
									Last Check
								</div>
								<div className="mt-1 text-lg font-mono font-medium text-foreground">
									{formatTimestamp(monitor.lastCheckedAt)}
								</div>
							</div>
							<div className="rounded-xl border border-border bg-muted/20 p-4">
								<div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
									Interval
								</div>
								<div className="mt-1 text-lg font-mono font-medium text-foreground">
									{monitor.config.interval}s
								</div>
							</div>
							<div className="rounded-xl border border-border bg-muted/20 p-4">
								<div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
									Timeout
								</div>
								<div className="mt-1 text-lg font-mono font-medium text-foreground">
									{monitor.config.timeout}ms
								</div>
							</div>
							{/* <div className="rounded-xl border border-border bg-muted/20 p-4">
								<div className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
									Status Code
								</div>
								<div className="mt-1 text-lg font-mono font-medium text-foreground">
									{monitor. && monitor.expectStatusHigh
										? `${monitor.expectStatusLow}-${monitor.expectStatusHigh}`
										: "Any 2xx"}
								</div>
							</div> */}
						</div>

						<section className="grid gap-6 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
							<SectionCard title="Configuration">
								<dl className="grid grid-cols-1 gap-4 text-sm text-muted-foreground">
									<div className="flex justify-between border-b border-border pb-2">
										<dt>Follow Redirects</dt>
										<dd className="text-foreground font-medium">
											{monitor.config.followRedirects ? "Yes" : "No"}
										</dd>
									</div>
									<div className="flex justify-between border-b border-border pb-2">
										<dt>Verify TLS</dt>
										<dd className="text-foreground font-medium">
											{monitor.config.verifyTls ? "Yes" : "No"}
										</dd>
									</div>
									<div className="flex justify-between border-b border-border pb-2">
										<dt>Created At</dt>
										<dd className="text-foreground font-medium">
											{formatTimestamp(monitor.createdAt)}
										</dd>
									</div>
									<div className="flex justify-between border-b border-border pb-2">
										<dt>Enabled</dt>
										<dd
											className={
												monitor.enabled
													? "text-emerald-500 font-medium"
													: "text-muted-foreground font-medium"
											}
										>
											{monitor.enabled ? "Yes" : "No"}
										</dd>
									</div>
								</dl>
							</SectionCard>
							<SectionCard
								title="Recent heartbeats"
								description="Latest executions streamed from the worker ticker"
							>
								{heartbeatWindow && (
									<p className="text-xs text-muted-foreground font-mono mb-2">
										Observing last {heartbeatWindow.hours}h ·{" "}
										{new Date(heartbeatWindow.sinceMs).toLocaleString()} →{" "}
										{new Date(heartbeatWindow.untilMs).toLocaleString()}
									</p>
								)}
								{heartbeatQuery.isLoading ? (
									<div className="space-y-3">
										<Skeleton className="h-12" />
										<Skeleton className="h-12" />
										<Skeleton className="h-12" />
									</div>
								) : heartbeatQuery.error instanceof Error ? (
									<div className="space-y-4">
										<p className="font-mono text-sm text-destructive">
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
									<p className="text-sm text-muted-foreground">
										No heartbeats yet. Checks will appear here after the first
										run.
									</p>
								) : (
									<div className="space-y-3">
										{heartbeats.map((heartbeat) => (
											<HeartbeatRow
												key={`${heartbeat.dispatchId ?? "unknown"}-${heartbeat.timestampMs}`}
												heartbeat={heartbeat}
											/>
										))}
									</div>
								)}
							</SectionCard>
						</section>
					</>
				) : null}
			</div>
		);
	}

	return route;
};

function HeartbeatRow({ heartbeat }: { heartbeat: HeartbeatSample }) {
	const ok = heartbeat.status === "up";
	const region = heartbeat.region ?? heartbeat.colo ?? "Unknown POP";
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
					<p className="font-mono text-xs text-[var(--text-muted)]">{region}</p>
				</div>
				<p className="font-mono text-xs text-[var(--text-muted)]">
					{formatHeartbeatTimestamp(heartbeat.timestampMs)}
				</p>
			</div>
			<div className="mt-2 grid grid-cols-2 gap-3 text-xs font-mono text-[var(--text-muted)] md:grid-cols-4">
				<div>
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						Status
					</p>
					<p className="text-[var(--text-primary)]">{heartbeat.code ?? "—"}</p>
				</div>
				<div>
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						RTT
					</p>
					<p className="text-[var(--text-primary)]">
						{heartbeat.latencyMs ? `${heartbeat.latencyMs}ms` : "—"}
					</p>
				</div>
				<div className="md:col-span-2">
					<p className="uppercase tracking-[0.3em] text-[var(--text-soft)]">
						Error
					</p>
					<p className="text-[var(--text-primary)] break-all">
						{heartbeat.error ?? "—"}
					</p>
				</div>
			</div>
		</div>
	);
}
