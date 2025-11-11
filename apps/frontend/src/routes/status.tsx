import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { Activity } from "lucide-react";
import { Hero } from "@/components/layout/Hero";
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
		<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto max-w-6xl space-y-10">
				<Hero
					eyebrow="Status"
					title="All systems operational"
					description="Last updated: just now"
					actions={
						<>
							<Button variant="secondary">Subscribe</Button>
							<Button variant="ghost">View history</Button>
						</>
					}
				/>

				<SectionCard
					title={
						<span className="flex items-center gap-3">
							<Activity size={20} />
							<span>Services</span>
						</span>
					}
				>
					<div className="rounded-3xl border border-white/10 bg-black/30 p-6 text-center text-sm text-[var(--text-muted)]">
						No services configured yet. As monitors come online they will be
						grouped here with simple status chips and availability notes.
					</div>
				</SectionCard>

				<SectionCard title="Uptime history">
					<StatsGrid
						items={uptimeWindows.map((window) => ({
							label: window.label,
							value: window.value,
							tone: "text-[var(--accent-green)]",
						}))}
						className="sm:grid-cols-3"
						cardClassName="bg-white/[0.02]"
					/>
				</SectionCard>

				<p className="text-center text-xs text-[var(--text-muted)]">
					Powered by Saavy Uptime — a Cloudflare-native status surface.
				</p>
			</div>
		</main>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/status",
		component: StatusPage,
		getParentRoute: () => parentRoute,
	});
