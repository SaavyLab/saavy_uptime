import { Link } from "@tanstack/react-router";
import {
	Activity,
	AlertCircle,
	Building2,
	GitBranch,
	Home,
	Menu,
	Monitor,
	X,
} from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";

const NAV_ITEMS = [
	{ to: "/", label: "Dashboard", icon: Home },
	{ to: "/monitors", label: "Monitors", icon: Monitor },
	{ to: "/dag", label: "DAG", icon: GitBranch },
	{ to: "/organization", label: "Organization", icon: Building2 },
	{ to: "/incidents", label: "Incidents", icon: AlertCircle },
	{ to: "/status", label: "Status", icon: Activity },
];

export default function Header() {
	const [isOpen, setIsOpen] = useState(false);

	const navLinkClass =
		"flex items-center gap-2 rounded-full border border-transparent px-4 py-2 text-xs font-semibold uppercase tracking-[0.3em] text-[var(--text-soft)] transition hover:border-white/20 hover:text-[var(--text-primary)]";
	const navLinkActiveClass =
		"flex items-center gap-2 rounded-full border border-white/40 bg-white/5 px-4 py-2 text-xs font-semibold uppercase tracking-[0.3em] text-[var(--text-primary)]";

	return (
		<>
			<header className="sticky top-0 z-40 border-b border-white/10 bg-[rgba(4,4,5,0.85)] px-4 py-4 backdrop-blur-2xl">
				<div className="mx-auto flex max-w-6xl items-center justify-between gap-4">
					<Link
						to="/"
						className="flex items-center gap-3 rounded-full border border-white/10 bg-white/5 px-4 py-2 text-[0.75rem] font-semibold uppercase tracking-[0.4em] text-[var(--text-primary)]"
					>
						<span className="h-2 w-2 rounded-full bg-[var(--accent)] shadow-[0_0_25px_var(--accent)]" />
						Saavy Uptime
					</Link>

					<nav className="hidden items-center gap-2 md:flex">
						{NAV_ITEMS.map(({ to, label, icon: Icon }) => (
							<Link
								key={to}
								to={to}
								className={navLinkClass}
								activeProps={{ className: navLinkActiveClass }}
							>
								<Icon size={16} />
								<span>{label}</span>
							</Link>
						))}
					</nav>

					<div className="flex items-center gap-3">
						<Link to="/monitors/new" className="hidden md:block">
							<Button size="sm">New monitor</Button>
						</Link>
						<button
							type="button"
							className="flex items-center gap-2 rounded-full border border-white/15 bg-white/5 px-3 py-2 text-[var(--text-primary)] transition hover:border-white/40 md:hidden"
							onClick={() => setIsOpen(true)}
							aria-label="Open navigation"
						>
							<Menu size={18} />
							Menu
						</button>
					</div>
				</div>
			</header>

			<button
				type="button"
				aria-label="Close navigation"
				className={`fixed inset-0 z-40 bg-black/60 backdrop-blur-sm transition-opacity duration-200 md:hidden ${
					isOpen
						? "opacity-100 pointer-events-auto"
						: "pointer-events-none opacity-0"
				}`}
				onClick={() => setIsOpen(false)}
			/>

			<aside
				className={`fixed bottom-4 right-4 z-50 w-[calc(100%-2rem)] max-w-sm rounded-[28px] border border-white/10 bg-[rgba(5,5,7,0.95)] p-6 text-[var(--text-primary)] shadow-[0_40px_80px_rgba(0,0,0,0.65)] transition-transform duration-300 md:hidden ${
					isOpen ? "translate-y-0" : "translate-y-[120%]"
				}`}
			>
				<div className="flex items-center justify-between">
					<div>
						<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
							Navigate
						</p>
						<p className="text-lg font-semibold">Control Surface</p>
					</div>
					<button
						type="button"
						className="rounded-full border border-white/15 bg-white/10 p-2"
						onClick={() => setIsOpen(false)}
						aria-label="Close navigation"
					>
						<X size={18} />
					</button>
				</div>

				<nav className="mt-6 flex flex-col gap-3">
					{NAV_ITEMS.map(({ to, label, icon: Icon }) => (
						<Link
							key={`mobile-${to}`}
							to={to}
							onClick={() => setIsOpen(false)}
							className="flex items-center gap-3 rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm font-medium"
							activeProps={{
								className:
									"flex items-center gap-3 rounded-2xl border-white/40 bg-white/10 px-4 py-3 text-sm font-medium",
							}}
						>
							<span className="rounded-xl bg-white/10 p-2">
								<Icon size={18} />
							</span>
							<span className="flex-1">{label}</span>
						</Link>
					))}
				</nav>

				<Link to="/monitors/new" onClick={() => setIsOpen(false)}>
					<Button className="mt-6 w-full">Launch new monitor</Button>
				</Link>
			</aside>
		</>
	);
}
