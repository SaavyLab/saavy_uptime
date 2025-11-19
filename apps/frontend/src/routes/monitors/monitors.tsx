import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState, useEffect } from "react";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import {
	RefreshCcw,
	Plus,
	Search,
	Wrench,
	Database,
	ChevronLeft,
	ChevronRight,
	Activity,
	CheckCircle,
	XCircle,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getMonitors, seedMonitors, type Monitor } from "@/lib/monitors";
import { reconcileTickers } from "@/lib/ticker";
import type { RouterContext } from "@/router-context";
import { cn } from "@/lib/utils";

function StatusBadge({ status }: { status?: string }) {
	const colors = {
		up: "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400",
		down: "bg-red-500/15 text-red-600 dark:text-red-400",
		maintenance: "bg-amber-500/15 text-amber-600 dark:text-amber-400",
		unknown: "bg-slate-500/15 text-slate-600 dark:text-slate-400",
	};

	const color = colors[status as keyof typeof colors] || colors.unknown;

	return (
		<span
			className={cn(
				"inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ring-1 ring-inset ring-black/5 dark:ring-white/10",
				color,
			)}
		>
			{status?.toUpperCase() || "UNKNOWN"}
		</span>
	);
}

type SortColumn = "name" | "status" | "interval" | "lastCheckedAt";

function MonitorsPage() {
	const {
		data: monitors = [],
		isLoading,
		refetch,
	} = useQuery<Monitor[]>({
		queryKey: ["monitors"],
		staleTime: 1000 * 60 * 5,
		queryFn: () => getMonitors(),
	});

	const [searchTerm, setSearchTerm] = useState("");
	const [kindFilter, setKindFilter] = useState("all");
	const [statusFilter, setStatusFilter] = useState("all");
	const [pageIndex, setPageIndex] = useState(0);
	const [pageSize, setPageSize] = useState(20);
	const [sortColumn, setSortColumn] = useState<SortColumn>("name");
	const [sortDirection, setSortDirection] = useState<"asc" | "desc">("asc");

	const queryClient = useQueryClient();
	const navigate = useNavigate();

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
			toast.error(
				err instanceof Error ? err.message : "Unable to reconcile tickers",
			);
		},
	});

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
			toast.error(
				err instanceof Error ? err.message : "Unable to seed monitors",
			);
		},
	});

	const monitorKinds = useMemo(() => {
		const kinds = Array.from(new Set(monitors.map((monitor) => monitor.kind)));
		return kinds.sort((a, b) => a.localeCompare(b));
	}, [monitors]);

	const monitorStatuses = useMemo(() => {
		const statuses = Array.from(new Set(monitors.map((monitor) => monitor.status)));
		return statuses.sort((a, b) => a.localeCompare(b));
	}, [monitors]);

	const filteredMonitors = useMemo(() => {
		const normalizedSearch = searchTerm.trim().toLowerCase();
		return monitors.filter((monitor) => {
			const matchesKind =
				kindFilter === "all" ? true : monitor.kind === kindFilter;
			const matchesStatus =
				statusFilter === "all" ? true : monitor.status === statusFilter;
			const matchesSearch = normalizedSearch
				? [monitor.name, monitor.config.url].some((v) =>
						v.toLowerCase().includes(normalizedSearch),
					)
				: true;
			return matchesKind && matchesStatus && matchesSearch;
		});
	}, [monitors, kindFilter, statusFilter, searchTerm]);

	const sortedMonitors = useMemo(() => {
		const data = [...filteredMonitors];
		const direction = sortDirection === "asc" ? 1 : -1;
		data.sort((a, b) => {
			const compare = (valueA: string | number | null, valueB: string | number | null) => {
				if (valueA == null && valueB == null) return 0;
				if (valueA == null) return -1;
				if (valueB == null) return 1;
				if (typeof valueA === "number" && typeof valueB === "number") {
					return valueA - valueB;
				}
				return valueA.toString().localeCompare(valueB.toString());
			};

			switch (sortColumn) {
				case "status":
					return direction * compare(a.status, b.status);
				case "interval":
					return direction * compare(a.config.interval, b.config.interval);
				case "lastCheckedAt":
					return direction * compare(a.lastCheckedAt ?? null, b.lastCheckedAt ?? null);
				default:
					return direction * compare(a.name, b.name);
			}
		});
		return data;
	}, [filteredMonitors, sortColumn, sortDirection]);

	// Pagination logic
	const totalPages = Math.ceil(sortedMonitors.length / pageSize) || 1;
	const safePageIndex = Math.min(pageIndex, totalPages - 1);
	const pageStart = safePageIndex * pageSize;
	const pageEnd = Math.min(pageStart + pageSize, sortedMonitors.length);
	const currentData = sortedMonitors.slice(pageStart, pageEnd);

	// Reset page when filters change
	useEffect(() => {
		// Explicitly reference dependencies so Biome understands the effect is tied to them.
		void searchTerm;
		void kindFilter;
		void statusFilter;
		void pageSize;
		setPageIndex(0);
	}, [searchTerm, kindFilter, statusFilter, pageSize]);

	const toggleSort = (column: SortColumn) => {
		setPageIndex(0);
		if (sortColumn === column) {
			setSortDirection((prev) => (prev === "asc" ? "desc" : "asc"));
			return;
		}
		setSortColumn(column);
		setSortDirection("asc");
	};

	const sortIndicator = (column: SortColumn) => {
		if (sortColumn !== column) return null;
		return sortDirection === "asc" ? "↑" : "↓";
	};

	// Fleet Stats
	const totalCount = monitors.length;
	const upCount = monitors.filter((m) => m.status === "up").length;
	const downCount = monitors.filter((m) => m.status === "down").length;

	return (
		<div className="space-y-8">
			<div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
				<div className="space-y-1">
					<h1 className="text-2xl font-bold tracking-tight">
						Monitor Inventory
					</h1>
					<p className="text-muted-foreground">
						Manage configuration and view health status for all deployed checks.
					</p>
				</div>
				<div className="flex items-center gap-2">
					{tickerAdminEnabled && (
						<>
							<Button
								variant="outline"
								onClick={() => reconcileMutation.mutate()}
								disabled={reconcileMutation.isPending}
								className="gap-2"
							>
								<Wrench
									className={cn(
										"h-4 w-4",
										reconcileMutation.isPending && "animate-spin",
									)}
								/>
								Warm Ticker
							</Button>
							<Button
								variant="outline"
								onClick={() => seedMutation.mutate()}
								disabled={seedMutation.isPending}
								className="gap-2"
							>
								<Database
									className={cn(
										"h-4 w-4",
										seedMutation.isPending && "animate-spin",
									)}
								/>
								Seed Data
							</Button>
						</>
					)}
					<Button
						variant="secondary"
						onClick={() => refetch()}
						className="gap-2"
					>
						<RefreshCcw
							className={cn("h-4 w-4", isLoading && "animate-spin")}
						/>
						Refresh
					</Button>
					<Link to="/monitors/new">
						<Button className="gap-2">
							<Plus className="h-4 w-4" />
							New Monitor
						</Button>
					</Link>
				</div>
			</div>

			<div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">
							Total Monitors
						</CardTitle>
						<Activity className="h-4 w-4 text-muted-foreground" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">{totalCount}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Healthy</CardTitle>
						<CheckCircle className="h-4 w-4 text-emerald-500" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">{upCount}</div>
					</CardContent>
				</Card>
				<Card>
					<CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
						<CardTitle className="text-sm font-medium">Down</CardTitle>
						<XCircle className="h-4 w-4 text-red-500" />
					</CardHeader>
					<CardContent>
						<div className="text-2xl font-bold">{downCount}</div>
					</CardContent>
				</Card>
			</div>

			<div className="flex flex-col gap-4">
		<div className="flex items-center gap-4">
					<div className="relative flex-1 max-w-sm">
						<Search className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
						<Input
							placeholder="Search monitors..."
							className="pl-9 bg-background"
							value={searchTerm}
							onChange={(e) => setSearchTerm(e.target.value)}
						/>
					</div>
					<Select value={kindFilter} onValueChange={setKindFilter}>
						<SelectTrigger className="w-[180px] bg-background">
							<SelectValue placeholder="Filter by type" />
						</SelectTrigger>
						<SelectContent>
							<SelectItem value="all">All Types</SelectItem>
							{monitorKinds.map((k) => (
								<SelectItem key={k} value={k}>
									{k.toUpperCase()}
								</SelectItem>
							))}
						</SelectContent>
					</Select>
			<Select value={statusFilter} onValueChange={setStatusFilter}>
				<SelectTrigger className="w-[180px] bg-background">
					<SelectValue placeholder="Filter by status" />
				</SelectTrigger>
				<SelectContent>
					<SelectItem value="all">All Statuses</SelectItem>
					{monitorStatuses.map((status) => (
						<SelectItem key={status} value={status}>
							{status.toUpperCase()}
						</SelectItem>
					))}
				</SelectContent>
			</Select>
				</div>

				<div className="rounded-md border bg-card">
					<Table>
				<TableHeader>
					<TableRow>
						<TableHead className="w-[300px]">
							<button
								type="button"
								className="flex items-center gap-1"
								onClick={() => toggleSort("name")}
							>
								Name {sortIndicator("name")}
							</button>
						</TableHead>
						<TableHead>
							<button
								type="button"
								className="flex items-center gap-1"
								onClick={() => toggleSort("status")}
							>
								Status {sortIndicator("status")}
							</button>
						</TableHead>
						<TableHead>
							<button
								type="button"
								className="flex items-center gap-1"
								onClick={() => toggleSort("interval")}
							>
								Interval {sortIndicator("interval")}
							</button>
						</TableHead>
						<TableHead>
							<button
								type="button"
								className="flex items-center gap-1"
								onClick={() => toggleSort("lastCheckedAt")}
							>
								Last Check {sortIndicator("lastCheckedAt")}
							</button>
						</TableHead>
						<TableHead className="text-right">Actions</TableHead>
					</TableRow>
				</TableHeader>
						<TableBody>
							{isLoading ? (
								<TableRow>
									<TableCell colSpan={5} className="h-24 text-center">
										Loading monitors...
									</TableCell>
								</TableRow>
							) : currentData.length === 0 ? (
								<TableRow>
									<TableCell
										colSpan={5}
										className="h-24 text-center text-muted-foreground"
									>
										No monitors found.
									</TableCell>
								</TableRow>
							) : (
								currentData.map((monitor) => (
									<TableRow
										key={monitor.id}
										className="cursor-pointer"
										onClick={() =>
											navigate({
												to: "/monitors/$monitorId",
												params: { monitorId: monitor.id },
											})
										}
									>
										<TableCell>
											<div className="font-medium">{monitor.name}</div>
											<div
												className="text-xs text-muted-foreground truncate max-w-[280px]"
												title={monitor.config.url}
											>
												{monitor.config.url}
											</div>
										</TableCell>
										<TableCell>
											<StatusBadge status={monitor.status} />
										</TableCell>
										<TableCell>{monitor.config.interval}s</TableCell>
										<TableCell>
											{monitor.lastCheckedAt
												? new Date(
														monitor.lastCheckedAt * 1000,
													).toLocaleString()
												: "Never"}
										</TableCell>
										<TableCell className="text-right">
											<Button variant="ghost" size="sm">
												Details
											</Button>
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</div>

				<div className="flex items-center justify-between px-2">
					<div className="text-xs text-muted-foreground">
						Showing {currentData.length > 0 ? pageStart + 1 : 0}-{pageEnd} of{" "}
						{filteredMonitors.length} monitors
					</div>

					<div className="flex items-center gap-2">
						<div className="flex items-center gap-2 mr-4">
							<span className="text-xs text-muted-foreground">
								Rows per page
							</span>
							<Select
								value={pageSize.toString()}
								onValueChange={(val) => setPageSize(Number(val))}
							>
								<SelectTrigger className="h-8 w-[70px]">
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="10">10</SelectItem>
									<SelectItem value="20">20</SelectItem>
									<SelectItem value="50">50</SelectItem>
									<SelectItem value="100">100</SelectItem>
								</SelectContent>
							</Select>
						</div>

						<Button
							variant="outline"
							size="icon"
							className="h-8 w-8"
							onClick={() => setPageIndex((p) => Math.max(0, p - 1))}
							disabled={safePageIndex === 0}
						>
							<ChevronLeft className="h-4 w-4" />
						</Button>
						<div className="text-xs font-medium">
							Page {safePageIndex + 1} of {totalPages}
						</div>
						<Button
							variant="outline"
							size="icon"
							className="h-8 w-8"
							onClick={() =>
								setPageIndex((p) => Math.min(totalPages - 1, p + 1))
							}
							disabled={safePageIndex >= totalPages - 1}
						>
							<ChevronRight className="h-4 w-4" />
						</Button>
					</div>
				</div>
			</div>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/monitors",
		component: MonitorsPage,
		getParentRoute: () => parentRoute,
	});
