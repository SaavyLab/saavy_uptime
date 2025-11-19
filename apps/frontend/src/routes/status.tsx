import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { Activity } from "lucide-react";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { StatsGrid } from "@/components/ui/StatsCard";
import type { RouterContext } from "@/router-context";

function StatusPage() {
	const uptimeWindows = [
		{ label: "24 HOURS", value: "100.00%" },
		{ label: "7 DAYS", value: "100.00%" },
		{ label: "30 DAYS", value: "100.00%" },
	];

	return (
		<div className="space-y-8">
			<div className="flex items-center justify-between">
				<div className="space-y-1">
					<h1 className="text-2xl font-bold tracking-tight">Status Pages</h1>
					<p className="text-muted-foreground">
						Public status page configuration and history.
					</p>
				</div>
				<div className="flex items-center gap-2">
					<Button variant="secondary">Subscribe</Button>
					<Button variant="ghost">View History</Button>
				</div>
			</div>

			<SectionCard
				title={
					<span className="flex items-center gap-3">
						<Activity size={20} />
						<span>Services</span>
					</span>
				}
			>
				<div className="rounded-3xl border border-border bg-muted/20 p-6 text-center text-sm text-muted-foreground">
					No services configured yet. As monitors come online they will be
					grouped here with simple status chips and availability notes.
				</div>
			</SectionCard>

			<SectionCard title="Uptime history">
				<StatsGrid
					items={uptimeWindows.map((window) => ({
						label: window.label,
						value: window.value,
						tone: "text-emerald-500",
					}))}
					className="sm:grid-cols-3"
					cardClassName="bg-muted/20"
				/>
			</SectionCard>

			<p className="text-center text-xs text-muted-foreground">
				Powered by Saavy Uptime — a Cloudflare-native status surface.
			</p>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/status",
		component: StatusPage,
		getParentRoute: () => parentRoute,
	});
