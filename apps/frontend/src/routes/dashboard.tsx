import { useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import {
	Activity,
	CheckCircle,
	XCircle,
	AlertTriangle,
	Plus,
	ArrowUpRight,
	Zap,
	Database,
	Radio,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/Skeleton";
import { getMonitors, type Monitor } from "@/lib/monitors";
import type { RouterContext } from "@/router-context";
import { cn } from "@/lib/utils";

const formatTimestamp = (value?: number | null) => {
	if (!value) return "—";
	return new Date(value * 1000).toLocaleString();
};

function StatCard({
	label,
	value,
	icon: Icon,
	trend,
	color = "cyan",
}: {
	label: string;
	value: string | number;
	icon: React.ElementType;
	trend?: string;
	color?: "cyan" | "green" | "red" | "amber";
}) {
	const colors = {
		cyan: "from-cyan-500/20 to-transparent border-cyan-500/20 text-cyan-400",
		green: "from-emerald-500/20 to-transparent border-emerald-500/20 text-emerald-400",
		red: "from-red-500/20 to-transparent border-red-500/20 text-red-400",
		amber: "from-amber-500/20 to-transparent border-amber-500/20 text-amber-400",
	};
	const iconColors = {
		cyan: "text-cyan-400",
		green: "text-emerald-400",
		red: "text-red-400",
		amber: "text-amber-400",
	};

	return (
		<div className={cn(
			"relative overflow-hidden rounded-lg border bg-gradient-to-b p-4",
			colors[color]
		)}>
			<div className="flex items-start justify-between">
				<div>
					<p className="text-xs font-medium text-zinc-500 uppercase tracking-wider">{label}</p>
					<p className="mt-2 text-3xl font-semibold tracking-tight text-zinc-100">{value}</p>
					{trend && (
						<p className="mt-1 text-xs text-zinc-500">{trend}</p>
					)}
				</div>
				<div className={cn("p-2 rounded-md bg-white/5", iconColors[color])}>
					<Icon size={18} strokeWidth={1.5} />
				</div>
			</div>
		</div>
	);
}

function DashboardPage() {
	const { data: monitors = [], isLoading } = useQuery<Monitor[]>({
		queryKey: ["monitors"],
		queryFn: () => getMonitors(),
	});

	const upCount = monitors.filter((m) => m.status === "up").length;
	const downCount = monitors.filter((m) => m.status === "down").length;
	const degradedCount = monitors.filter((m) => m.status === "degraded").length;
	const uptimePercent = monitors.length > 0 
		? ((upCount / monitors.length) * 100).toFixed(1) 
		: "—";

	const recentMonitors = [...monitors]
		.sort((a, b) => (b.lastCheckedAt ?? 0) - (a.lastCheckedAt ?? 0))
		.slice(0, 5);

	return (
		<div className="space-y-8">
			{/* Header */}
			<div className="flex items-start justify-between">
				<div>
					<h1 className="text-2xl font-semibold tracking-tight text-zinc-100">Overview</h1>
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
			<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
				<StatCard
					label="Total Monitors"
					value={isLoading ? "—" : monitors.length}
					icon={Activity}
					color="cyan"
				/>
				<StatCard
					label="Operational"
					value={isLoading ? "—" : upCount}
					icon={CheckCircle}
					trend={!isLoading && monitors.length > 0 ? `${uptimePercent}% uptime` : undefined}
					color="green"
				/>
				<StatCard
					label="Down"
					value={isLoading ? "—" : downCount}
					icon={XCircle}
					color="red"
				/>
				<StatCard
					label="Degraded"
					value={isLoading ? "—" : degradedCount}
					icon={AlertTriangle}
					color="amber"
				/>
			</div>

			{/* Main Content Grid */}
			<div className="grid gap-6 lg:grid-cols-5">
				{/* Recent Activity */}
				<div className="lg:col-span-3 rounded-lg border border-white/[0.06] bg-zinc-900/50">
					<div className="flex items-center justify-between border-b border-white/[0.06] px-5 py-4">
						<div>
							<h2 className="text-sm font-medium text-zinc-200">Recent Activity</h2>
							<p className="text-xs text-zinc-600 mt-0.5">Latest checks from your monitors</p>
						</div>
						<Link to="/monitors" className="text-xs text-cyan-400 hover:text-cyan-300 flex items-center gap-1 transition-colors">
							View all <ArrowUpRight size={12} />
						</Link>
					</div>
					<div className="divide-y divide-white/[0.04]">
						{isLoading ? (
							<div className="p-5 space-y-3">
								<Skeleton className="h-12 w-full" />
								<Skeleton className="h-12 w-full" />
								<Skeleton className="h-12 w-full" />
							</div>
						) : recentMonitors.length > 0 ? (
							recentMonitors.map((monitor) => (
								<Link
									key={monitor.id}
									to="/monitors/$monitorId"
									params={{ monitorId: monitor.id }}
									className="flex items-center justify-between px-5 py-3.5 hover:bg-white/[0.02] transition-colors"
								>
									<div className="flex items-center gap-3 min-w-0">
										<div
											className={cn(
												"w-1.5 h-1.5 rounded-full shrink-0",
												monitor.status === "up"
													? "bg-emerald-400"
													: monitor.status === "down"
														? "bg-red-400"
														: "bg-zinc-500",
											)}
										/>
										<div className="min-w-0">
											<p className="text-sm font-medium text-zinc-300 truncate">
												{monitor.name}
											</p>
											<p className="text-xs text-zinc-600 truncate font-mono">
												{monitor.config.url}
											</p>
										</div>
									</div>
									<div className="text-right shrink-0 ml-4">
										<p className={cn(
											"text-xs font-medium",
											monitor.status === "up" ? "text-emerald-400" : 
											monitor.status === "down" ? "text-red-400" : "text-zinc-500"
										)}>
											{monitor.status?.toUpperCase()}
										</p>
										<p className="text-[10px] text-zinc-600 font-mono mt-0.5">
											{formatTimestamp(monitor.lastCheckedAt)}
										</p>
									</div>
								</Link>
							))
						) : (
							<div className="p-8 text-center">
								<p className="text-sm text-zinc-500">No monitors configured yet</p>
								<Link to="/monitors/new">
									<Button variant="outline" size="sm" className="mt-3">
										Create your first monitor
									</Button>
								</Link>
							</div>
						)}
					</div>
				</div>

				{/* System Status */}
				<div className="lg:col-span-2 rounded-lg border border-white/[0.06] bg-zinc-900/50">
					<div className="border-b border-white/[0.06] px-5 py-4">
						<h2 className="text-sm font-medium text-zinc-200">System Status</h2>
						<p className="text-xs text-zinc-600 mt-0.5">Platform health indicators</p>
					</div>
					<div className="p-5 space-y-4">
						<div className="flex items-center gap-3">
							<div className="h-8 w-8 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Zap size={14} className="text-emerald-400" />
							</div>
							<div className="flex-1">
								<p className="text-sm text-zinc-300">Durable Object Ticker</p>
								<p className="text-xs text-zinc-600">Scheduling checks</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
						</div>
						<div className="flex items-center gap-3">
							<div className="h-8 w-8 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Database size={14} className="text-emerald-400" />
							</div>
							<div className="flex-1">
								<p className="text-sm text-zinc-300">D1 Database</p>
								<p className="text-xs text-zinc-600">Hot state storage</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400" />
						</div>
						<div className="flex items-center gap-3">
							<div className="h-8 w-8 rounded-md bg-emerald-500/10 flex items-center justify-center">
								<Radio size={14} className="text-emerald-400" />
							</div>
							<div className="flex-1">
								<p className="text-sm text-zinc-300">Analytics Engine</p>
								<p className="text-xs text-zinc-600">Heartbeat metrics</p>
							</div>
							<div className="h-2 w-2 rounded-full bg-emerald-400" />
						</div>
					</div>
					
					{/* Quick Stats */}
					<div className="border-t border-white/[0.06] px-5 py-4">
						<div className="grid grid-cols-2 gap-4">
							<div>
								<p className="text-[10px] font-medium text-zinc-600 uppercase tracking-wider">Avg Response</p>
								<p className="text-lg font-mono font-medium text-zinc-300 mt-1">—</p>
							</div>
							<div>
								<p className="text-[10px] font-medium text-zinc-600 uppercase tracking-wider">Checks/min</p>
								<p className="text-lg font-mono font-medium text-zinc-300 mt-1">—</p>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/",
		component: DashboardPage,
		getParentRoute: () => parentRoute,
	});
