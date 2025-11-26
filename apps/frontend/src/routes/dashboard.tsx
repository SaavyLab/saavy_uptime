import { useQuery, useQueries } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import {
	Activity,
	CheckCircle,
	XCircle,
	AlertTriangle,
	Plus,
	Zap,
	Database,
	Radio,
	Globe,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { WorldMap, aggregateHeartbeatsByLocation } from "@/components/WorldMap";
import {
	getMonitors,
	getMonitorHeartbeats,
	type Monitor,
} from "@/lib/monitors";
import type { RouterContext } from "@/router-context";

function DashboardPage() {
	const { data: monitors = [], isLoading } = useQuery<Monitor[]>({
		queryKey: ["monitors"],
		queryFn: () => getMonitors(),
	});

	// Fetch heartbeats for up to 10 monitors to show on the map
	const monitorIds = monitors.slice(0, 10).map((m) => m.id);
	const heartbeatQueries = useQueries({
		queries: monitorIds.map((id) => ({
			queryKey: ["monitor", id, "heartbeats", "dashboard"],
			queryFn: () => getMonitorHeartbeats(id, { limit: 20, windowHours: 1 }),
			enabled: monitors.length > 0,
			staleTime: 30000,
		})),
	});

	const allHeartbeats = heartbeatQueries.flatMap((q) => q.data?.items ?? []);
	const mapMarkers = aggregateHeartbeatsByLocation(allHeartbeats);

	const upCount = monitors.filter((m) => m.status === "up").length;
	const downCount = monitors.filter((m) => m.status === "down").length;
	const degradedCount = monitors.filter((m) => m.status === "degraded").length;

	return (
		<div className="space-y-6">
			{/* Header */}
			<div className="flex items-start justify-between">
				<div>
					<h1 className="text-2xl font-semibold tracking-tight text-zinc-100">
						Overview
					</h1>
					<p className="mt-1 text-sm text-zinc-500">
						Real-time infrastructure health across all monitors
					</p>
				</div>
				<Link to="/monitors/new">
					<Button size="sm" className="gap-1.5">
						<Plus className="h-3.5 w-3.5" />
						New Monitor
					</Button>
				</Link>
			</div>

			{/* Stats Grid */}
			<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Total Monitors
						</CardTitle>
						<Activity className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">
							{isLoading ? "—" : monitors.length}
						</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Healthy</CardTitle>
						<CheckCircle className="h-4 w-4 text-emerald-500" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">
							{isLoading ? "—" : upCount}
						</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Down</CardTitle>
						<XCircle className="h-4 w-4 text-red-500" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">
							{isLoading ? "—" : downCount}
						</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Degraded</CardTitle>
						<AlertTriangle className="h-4 w-4 text-amber-500" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">
							{isLoading ? "—" : degradedCount}
						</div>
					</CardContent>
				</Card>
			</div>

			{/* World Map */}
			<Card className="overflow-hidden">
				<CardHeader className="flex flex-row items-center justify-between space-y-0 border-b border-white/[0.06]">
					<div className="flex items-center gap-2">
						<Globe size={16} className="text-cyan-400" />
						<div>
							<CardTitle>Global Check Distribution</CardTitle>
							<p className="text-xs text-zinc-500 mt-0.5">
								Where your health checks are running
							</p>
						</div>
					</div>
					<div className="text-xs text-zinc-500">
						{mapMarkers.length} locations
					</div>
				</CardHeader>
				<div className="h-[300px] bg-zinc-950">
					{monitors.length === 0 ? (
						<div className="flex items-center justify-center h-full text-sm text-zinc-600">
							No monitors configured yet
						</div>
					) : (
						<WorldMap markers={mapMarkers} className="h-full" />
					)}
				</div>
			</Card>

			{/* System Status - Horizontal */}
			<Card>
				<CardHeader className="pb-3">
					<CardTitle>System Status</CardTitle>
					<p className="text-xs text-zinc-500">Platform health indicators</p>
				</CardHeader>
				<CardContent>
					<div className="grid gap-4 md:grid-cols-3">
						<div className="flex items-center gap-3 p-3 rounded-lg bg-white/[0.02]">
							<div className="h-9 w-9 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Zap size={16} className="text-emerald-400" />
							</div>
							<div className="flex-1 min-w-0">
								<p className="text-sm font-medium text-zinc-200">Ticker DO</p>
								<p className="text-xs text-zinc-500 truncate">
									Scheduling checks
								</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
						</div>
						<div className="flex items-center gap-3 p-3 rounded-lg bg-white/[0.02]">
							<div className="h-9 w-9 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Database size={16} className="text-emerald-400" />
							</div>
							<div className="flex-1 min-w-0">
								<p className="text-sm font-medium text-zinc-200">D1 Database</p>
								<p className="text-xs text-zinc-500 truncate">
									Hot state storage
								</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400" />
						</div>
						<div className="flex items-center gap-3 p-3 rounded-lg bg-white/[0.02]">
							<div className="h-9 w-9 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Radio size={16} className="text-emerald-400" />
							</div>
							<div className="flex-1 min-w-0">
								<p className="text-sm font-medium text-zinc-200">
									Analytics Engine
								</p>
								<p className="text-xs text-zinc-500 truncate">
									Heartbeat metrics
								</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400" />
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/",
		component: DashboardPage,
		getParentRoute: () => parentRoute,
	});
