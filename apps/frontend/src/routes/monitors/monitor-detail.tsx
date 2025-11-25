import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { Trash2, ChevronLeft, Clock, Timer, Settings2 } from "lucide-react";
import { toast } from "sonner";
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
import { cn } from "@/lib/utils";

const formatTimestamp = (value?: number | null) => {
	if (!value) {
		return "—";
	}
	return new Date(value * 1000).toLocaleString();
};

const formatHeartbeatTimestamp = (value: number) => {
	return new Date(value).toLocaleTimeString();
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
			<div className="space-y-6">
				{/* Breadcrumb */}
				<Link
					to="/monitors"
					className="inline-flex items-center gap-1.5 text-xs text-zinc-500 hover:text-zinc-300 transition-colors"
				>
					<ChevronLeft size={14} />
					Back to Monitors
				</Link>

				{monitorQuery.isLoading ? (
					<div className="space-y-6">
						<Skeleton className="h-20" />
						<Skeleton className="h-48" />
					</div>
				) : monitorQuery.error instanceof Error ? (
					<div className="rounded-lg border border-red-500/20 bg-red-500/10 p-6">
						<p className="font-mono text-sm text-red-400">
							{monitorQuery.error.message}
						</p>
						<Button type="button" size="sm" className="mt-4" onClick={() => monitorQuery.refetch()}>
							Try again
						</Button>
					</div>
				) : monitor ? (
					<>
						{/* Header */}
						<div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
							<div className="space-y-2">
								<div className="flex items-center gap-3">
									<h1 className="text-xl font-semibold tracking-tight text-zinc-100">
										{monitor.name}
									</h1>
									<StatusPill status={monitor.status} />
								</div>
								<p className="text-zinc-500 font-mono text-xs">
									{monitor.config.url}
								</p>
							</div>
							<div className="flex items-center gap-2">
								<Link to="/monitors/$monitorId/edit" params={{ monitorId }}>
									<Button variant="outline" size="sm">Edit</Button>
								</Link>
								<Button
									type="button"
									variant="ghost"
									onClick={handleDelete}
									disabled={deleteMutation.isPending}
									size="sm"
									className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
								>
									<Trash2 size={14} />
								</Button>
							</div>
						</div>

						{/* Stats Row */}
						<div className="grid gap-3 grid-cols-2 lg:grid-cols-4">
							<div className="rounded-lg border border-white/[0.06] bg-zinc-900/50 p-4">
								<div className="flex items-center gap-2 text-zinc-500 mb-2">
									<Clock size={12} />
									<span className="text-[10px] font-medium uppercase tracking-wider">Last Check</span>
								</div>
								<div className="text-sm font-mono font-medium text-zinc-200">
									{formatTimestamp(monitor.lastCheckedAt)}
								</div>
							</div>
							<div className="rounded-lg border border-white/[0.06] bg-zinc-900/50 p-4">
								<div className="flex items-center gap-2 text-zinc-500 mb-2">
									<Timer size={12} />
									<span className="text-[10px] font-medium uppercase tracking-wider">Interval</span>
								</div>
								<div className="text-sm font-mono font-medium text-zinc-200">
									{monitor.config.interval}s
								</div>
							</div>
							<div className="rounded-lg border border-white/[0.06] bg-zinc-900/50 p-4">
								<div className="flex items-center gap-2 text-zinc-500 mb-2">
									<Settings2 size={12} />
									<span className="text-[10px] font-medium uppercase tracking-wider">Timeout</span>
								</div>
								<div className="text-sm font-mono font-medium text-zinc-200">
									{monitor.config.timeout}ms
								</div>
							</div>
							<div className="rounded-lg border border-white/[0.06] bg-zinc-900/50 p-4">
								<div className="flex items-center gap-2 text-zinc-500 mb-2">
									<span className="text-[10px] font-medium uppercase tracking-wider">Response Time</span>
								</div>
								<div className="text-sm font-mono font-medium text-zinc-200">
									{monitor.rtMs ? `${monitor.rtMs}ms` : "—"}
								</div>
							</div>
						</div>

						{/* Main Content */}
						<div className="grid gap-6 lg:grid-cols-5">
							{/* Configuration */}
							<div className="lg:col-span-2 rounded-lg border border-white/[0.06] bg-zinc-900/50">
								<div className="border-b border-white/[0.06] px-5 py-4">
									<h2 className="text-sm font-medium text-zinc-200">Configuration</h2>
								</div>
								<div className="p-5 space-y-3">
									<div className="flex justify-between items-center py-2 border-b border-white/[0.04]">
										<span className="text-xs text-zinc-500">Follow Redirects</span>
										<span className="text-xs font-medium text-zinc-300">
											{monitor.config.followRedirects ? "Yes" : "No"}
										</span>
									</div>
									<div className="flex justify-between items-center py-2 border-b border-white/[0.04]">
										<span className="text-xs text-zinc-500">Verify TLS</span>
										<span className="text-xs font-medium text-zinc-300">
											{monitor.config.verifyTls ? "Yes" : "No"}
										</span>
									</div>
									<div className="flex justify-between items-center py-2 border-b border-white/[0.04]">
										<span className="text-xs text-zinc-500">Created</span>
										<span className="text-xs font-medium text-zinc-300">
											{formatTimestamp(monitor.createdAt)}
										</span>
									</div>
									<div className="flex justify-between items-center py-2">
										<span className="text-xs text-zinc-500">Enabled</span>
										<span className={cn(
											"text-xs font-medium",
											monitor.enabled ? "text-emerald-400" : "text-zinc-500"
										)}>
											{monitor.enabled ? "Yes" : "No"}
										</span>
									</div>
								</div>
							</div>

							{/* Heartbeats */}
							<div className="lg:col-span-3 rounded-lg border border-white/[0.06] bg-zinc-900/50">
								<div className="border-b border-white/[0.06] px-5 py-4">
									<h2 className="text-sm font-medium text-zinc-200">Recent Heartbeats</h2>
									{heartbeatWindow && (
										<p className="text-[10px] text-zinc-600 font-mono mt-1">
											Last {heartbeatWindow.hours}h · {heartbeats.length} checks
										</p>
									)}
								</div>
								<div className="max-h-[400px] overflow-y-auto">
									{heartbeatQuery.isLoading ? (
										<div className="p-5 space-y-3">
											<Skeleton className="h-10" />
											<Skeleton className="h-10" />
											<Skeleton className="h-10" />
										</div>
									) : heartbeatQuery.error instanceof Error ? (
										<div className="p-5">
											<p className="font-mono text-xs text-red-400">
												{heartbeatQuery.error.message}
											</p>
											<Button
												type="button"
												variant="outline"
												size="sm"
												className="mt-3"
												onClick={() => heartbeatQuery.refetch()}
											>
												Retry
											</Button>
										</div>
									) : heartbeats.length === 0 ? (
										<div className="p-8 text-center">
											<p className="text-xs text-zinc-500">
												No heartbeats yet. Checks will appear here after the first run.
											</p>
										</div>
									) : (
										<div className="divide-y divide-white/[0.04]">
											{heartbeats.slice(0, 20).map((heartbeat) => (
												<HeartbeatRow
													key={`${heartbeat.dispatchId ?? "unknown"}-${heartbeat.timestampMs}`}
													heartbeat={heartbeat}
												/>
											))}
										</div>
									)}
								</div>
							</div>
						</div>
					</>
				) : null}
			</div>
		);
	}

	return route;
};

function HeartbeatRow({ heartbeat }: { heartbeat: HeartbeatSample }) {
	const ok = heartbeat.status === "up";
	const region = heartbeat.region ?? heartbeat.colo ?? "—";
	
	return (
		<div className="flex items-center justify-between px-5 py-3 hover:bg-white/[0.02] transition-colors">
			<div className="flex items-center gap-3">
				<div className={cn(
					"w-1.5 h-1.5 rounded-full",
					ok ? "bg-emerald-400" : "bg-red-400"
				)} />
				<span className={cn(
					"text-xs font-mono font-medium w-8",
					ok ? "text-emerald-400" : "text-red-400"
				)}>
					{ok ? "OK" : "FAIL"}
				</span>
				<span className="text-[10px] font-mono text-zinc-500 bg-zinc-800 px-1.5 py-0.5 rounded">
					{region}
				</span>
			</div>
			<div className="flex items-center gap-4">
				{heartbeat.latencyMs && (
					<span className="text-xs font-mono text-zinc-400">
						{heartbeat.latencyMs}ms
					</span>
				)}
				{heartbeat.code && (
					<span className="text-[10px] font-mono text-zinc-500">
						{heartbeat.code}
					</span>
				)}
				<span className="text-[10px] font-mono text-zinc-600">
					{formatHeartbeatTimestamp(heartbeat.timestampMs)}
				</span>
			</div>
		</div>
	);
}
