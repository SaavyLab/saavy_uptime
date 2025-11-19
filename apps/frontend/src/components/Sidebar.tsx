import { Link } from "@tanstack/react-router";
import {
	Activity,
	AlertCircle,
	Building2,
	GitBranch,
	Home,
	Monitor,
	Settings,
} from "lucide-react";

const NAV_ITEMS = [
	{ to: "/", label: "Overview", icon: Home },
	{ to: "/monitors", label: "Monitors", icon: Monitor },
	{ to: "/incidents", label: "Incidents", icon: AlertCircle },
	{ to: "/dag", label: "DAG Execution", icon: GitBranch },
	{ to: "/status", label: "Status Pages", icon: Activity },
	{ to: "/organization", label: "Organization", icon: Building2 },
];

export function Sidebar() {
	return (
		<aside className="fixed inset-y-0 left-0 z-50 hidden w-64 flex-col border-r border-border bg-muted/5 md:flex">
			<div className="flex h-14 items-center border-b border-border px-6">
				<Link
					to="/"
					className="flex items-center gap-2 font-bold text-foreground"
				>
					<div className="h-6 w-6 rounded-md bg-primary text-primary-foreground flex items-center justify-center">
						<Activity size={16} strokeWidth={3} />
					</div>
					<span>Saavy Uptime</span>
				</Link>
			</div>

			<div className="flex-1 overflow-y-auto py-4">
				<nav className="px-4 space-y-1">
					<div className="px-2 pb-2 text-xs font-medium text-muted-foreground uppercase tracking-wider">
						Platform
					</div>
					{NAV_ITEMS.map((item) => (
						<Link
							key={item.to}
							to={item.to}
							className="group flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
							activeProps={{
								className:
									"bg-sidebar-accent text-sidebar-foreground font-semibold",
							}}
						>
							<item.icon size={16} />
							{item.label}
						</Link>
					))}
				</nav>
			</div>

			<div className="border-t border-border p-4">
				<div className="flex items-center gap-3 rounded-md p-2 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground cursor-pointer transition-colors">
					<div className="h-8 w-8 rounded-full bg-muted flex items-center justify-center text-xs font-medium">
						SU
					</div>
					<div className="flex-1 overflow-hidden">
						<p className="truncate text-sm font-medium text-foreground">
							Saavy User
						</p>
						<p className="truncate text-xs text-muted-foreground">
							user@saavy.dev
						</p>
					</div>
					<Settings size={16} className="text-muted-foreground" />
				</div>
			</div>
		</aside>
	);
}
