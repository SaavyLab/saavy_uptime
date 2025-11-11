import { createRoute, Link } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { Plus } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { getMonitors, type Monitor } from "@/lib/monitors";

const ORG_ID = "zimr7nsz8gj0nxsgqktogm4v";

const statusColorMap: Record<string, string> = {
	up: "bg-[#00ff00] text-black",
	down: "bg-[#ff0000] text-white",
	degraded: "bg-[#ffff00] text-black",
	maintenance: "bg-[#ff6633] text-black",
	unknown: "bg-gray-400 text-black",
};

const formatTimestamp = (value?: number | null) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

function MonitorsPage() {
	const {
		data,
		isLoading,
		error,
		refetch,
	} = useQuery<Monitor[]>({
		queryKey: ["monitors", ORG_ID],
		queryFn: () => getMonitors(ORG_ID),
	});

	const monitors = data ?? [];
	const total = monitors.length;
	const enabled = monitors.filter((monitor) => Boolean(monitor.enabled)).length;
	const failing = monitors.filter((monitor) => monitor.current_status === "down").length;

	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-7xl mx-auto space-y-8">
				<div className="flex items-center justify-between">
					<div className="border-4 border-black dark:border-white p-6 flex-1 mr-4 bg-[#ff6633]">
						<h1 className="text-4xl mb-2">MONITORS</h1>
						<p className="font-mono text-sm normal-case">
							Configure and manage HTTP/HTTPS monitors
						</p>
					</div>
					<Link
						to="/monitors/new"
						className="border-4 border-black dark:border-white bg-[#00ff00] text-black p-6 hover:bg-[#00cc00] transition-colors flex items-center gap-3 font-bold uppercase"
					>
						<Plus size={24} strokeWidth={3} />
						New Monitor
					</Link>
				</div>

				<div className="grid grid-cols-1 md:grid-cols-3 gap-4">
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<p className="text-sm font-mono text-muted-foreground uppercase mb-2">
							Total
						</p>
						<p className="text-4xl font-bold">{isLoading ? "…" : total}</p>
					</div>
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<p className="text-sm font-mono text-muted-foreground uppercase mb-2">
							Enabled
						</p>
						<p className="text-4xl font-bold text-[#00ff00]">{isLoading ? "…" : enabled}</p>
					</div>
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<p className="text-sm font-mono text-muted-foreground uppercase mb-2">Failing</p>
						<p className="text-4xl font-bold text-[#ff0000]">{isLoading ? "…" : failing}</p>
					</div>
				</div>

				<div className="border-4 border-black dark:border-white bg-white dark:bg-black">
					<div className="flex items-center justify-between border-b-4 border-black dark:border-white p-6">
						<div>
							<h2 className="text-2xl uppercase">All monitors</h2>
							<p className="text-sm font-mono text-muted-foreground">
								Showing {total} monitors for org {ORG_ID}
							</p>
						</div>
						<button
							type="button"
							onClick={() => refetch()}
							className="border-2 border-black dark:border-white px-4 py-2 font-bold uppercase hover:bg-black hover:text-white transition-colors"
						>
							Refresh
						</button>
					</div>

					{isLoading ? (
						<p className="p-6 font-mono text-sm text-muted-foreground">
							Loading monitors…
						</p>
					) : error instanceof Error ? (
						<div className="p-6 space-y-2">
							<p className="text-red-500 font-mono text-sm">{error.message}</p>
							<button
								type="button"
								onClick={() => refetch()}
								className="border-2 border-black dark:border-white px-4 py-2 font-bold uppercase hover:bg-black hover:text-white transition-colors"
							>
								Try again
							</button>
						</div>
					) : monitors.length === 0 ? (
						<div className="p-6 font-mono text-sm text-muted-foreground">
							No monitors yet. Create one to start tracking uptime.
						</div>
					) : (
						<div className="divide-y-4 divide-black dark:divide-white">
							{monitors.map((monitor) => {
								const statusClass =
									statusColorMap[monitor.current_status] ?? statusColorMap.unknown;
								return (
									<div
										key={monitor.id}
										className="p-6 flex flex-col gap-3 md:flex-row md:items-center md:justify-between"
									>
										<div>
											<div className="flex items-center gap-3">
												<h3 className="text-2xl font-bold uppercase">{monitor.name}</h3>
												<span className={`px-3 py-1 text-xs font-bold uppercase ${statusClass}`}>
													{monitor.current_status}
												</span>
											</div>
											<p className="font-mono text-sm text-muted-foreground break-all">
												{monitor.url}
											</p>
											<p className="font-mono text-xs text-muted-foreground mt-1">
												Created {formatTimestamp(monitor.created_at)}
											</p>
										</div>
										<div className="grid grid-cols-2 gap-4 text-sm font-mono">
											<div>
												<p className="uppercase text-muted-foreground">Interval</p>
												<p>{monitor.interval_s}s</p>
											</div>
											<div>
												<p className="uppercase text-muted-foreground">Timeout</p>
												<p>{monitor.timeout_ms}ms</p>
											</div>
											<div>
												<p className="uppercase text-muted-foreground">Last Check</p>
												<p>{formatTimestamp(monitor.last_checked_at_ts)}</p>
											</div>
											<div>
												<p className="uppercase text-muted-foreground">Enabled</p>
												<p>{monitor.enabled ? "Yes" : "No"}</p>
											</div>
										</div>
									</div>
								);
							})}
						</div>
					)}
				</div>
			</div>
		</div>
	);
}

export default (
	parentRoute: RootRoute<Register, undefined, RouterContext>,
) =>
	createRoute({
		path: "/monitors",
		component: MonitorsPage,
		getParentRoute: () => parentRoute,
	});
