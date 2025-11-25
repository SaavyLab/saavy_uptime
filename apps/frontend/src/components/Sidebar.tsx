import { Link } from "@tanstack/react-router";
import {
	Activity,
	AlertCircle,
	Building2,
	GitBranch,
	Home,
	Monitor,
	Settings,
	Radio,
} from "lucide-react";

const NAV_ITEMS = [
	{ to: "/", label: "Overview", icon: Home },
	{ to: "/monitors", label: "Monitors", icon: Monitor },
	{ to: "/incidents", label: "Incidents", icon: AlertCircle },
	{ to: "/dag", label: "Execution DAG", icon: GitBranch },
	{ to: "/status", label: "Status Pages", icon: Radio },
	{ to: "/organization", label: "Organization", icon: Building2 },
];

export function Sidebar() {
	return (
		<aside className="fixed inset-y-0 left-0 z-50 hidden w-56 flex-col border-r border-white/[0.04] bg-[#0a0a0c]/90 backdrop-blur-xl md:flex">
			<div className="flex h-14 items-center border-b border-white/[0.04] px-5">
				<Link
					to="/"
					className="flex items-center gap-2.5 text-foreground"
				>
					<div className="h-7 w-7 rounded bg-gradient-to-br from-cyan-400 to-cyan-600 flex items-center justify-center shadow-[0_0_12px_rgba(6,182,212,0.4)]">
						<Activity size={14} strokeWidth={2.5} className="text-black" />
					</div>
					<div className="flex flex-col">
						<span className="text-sm font-semibold tracking-tight">Saavy</span>
						<span className="text-[10px] font-medium text-muted-foreground -mt-0.5 tracking-wide">UPTIME</span>
					</div>
				</Link>
			</div>

			<div className="flex-1 overflow-y-auto py-5">
				<nav className="px-3 space-y-0.5">
					{NAV_ITEMS.map((item) => (
						<Link
							key={item.to}
							to={item.to}
							className="group flex items-center gap-2.5 rounded px-2.5 py-2 text-[13px] font-medium text-zinc-500 transition-all hover:bg-white/[0.03] hover:text-zinc-300"
							activeProps={{
								className:
									"bg-cyan-500/[0.08] text-cyan-400 hover:bg-cyan-500/[0.08] hover:text-cyan-400",
							}}
							activeOptions={item.to === "/" ? { exact: true } : undefined}
						>
							<item.icon size={15} strokeWidth={1.75} />
							{item.label}
						</Link>
					))}
				</nav>
			</div>

			<div className="border-t border-white/[0.04] p-3">
				<div className="flex items-center gap-2.5 rounded px-2 py-2 hover:bg-white/[0.03] cursor-pointer transition-colors group">
					<div className="h-7 w-7 rounded bg-zinc-800 flex items-center justify-center text-[10px] font-semibold text-zinc-400 ring-1 ring-white/[0.06]">
						SU
					</div>
					<div className="flex-1 overflow-hidden">
						<p className="truncate text-xs font-medium text-zinc-300">
							Saavy User
						</p>
						<p className="truncate text-[10px] text-zinc-600">
							user@saavy.dev
						</p>
					</div>
					<Settings size={14} className="text-zinc-600 group-hover:text-zinc-400 transition-colors" />
				</div>
			</div>
		</aside>
	);
}
