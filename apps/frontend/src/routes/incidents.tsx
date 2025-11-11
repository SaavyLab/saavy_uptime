import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { AlertOctagon, ShieldOff } from "lucide-react";

import { Hero } from "@/components/layout/Hero";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import { StatsGrid } from "@/components/ui/StatsCard";
import type { RouterContext } from "@/router-context";

const metrics = [
	{ label: "MTTR", value: "--", hint: "mean time to recovery" },
	{ label: "Incidents (30d)", value: "0", hint: "last month" },
	{ label: "Avg downtime", value: "0m", hint: "rolling window" },
	{ label: "Longest incident", value: "—", hint: "awaiting data" },
];

function IncidentsPage() {
	return (
		<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto max-w-6xl space-y-10">
				<Hero
					eyebrow="Incidents"
					title="Incident room for Saavy Uptime"
					description="When the worker detects downtime, this board becomes the logbook for responders—statuses, timestamps, and notes in one place."
					actions={<Button variant="secondary">Export history</Button>}
				/>

				<SectionCard title="Incident metrics">
					<StatsGrid
						items={metrics}
						className="sm:grid-cols-2"
						cardClassName="bg-white/[0.02] border-white/10"
					/>
				</SectionCard>

				<SectionCard contentClassName="space-y-4 text-center">
					<AlertOctagon size={48} className="mx-auto text-[var(--text-soft)]" />
					<h2 className="text-2xl font-medium">All systems operational</h2>
					<p className="text-sm text-[var(--text-muted)]">
						Incidents appear here instantly with live badges and postmortems
						once they close.
					</p>
					<div className="mt-4 flex flex-wrap justify-center gap-3">
						<Button variant="secondary">Create test incident</Button>
						<Button variant="ghost">View status page</Button>
					</div>
				</SectionCard>

				<SectionCard
					title="Incident playbook"
					description="Lightweight guardrails for on-call responders"
					contentClassName="space-y-4"
				>
					<div className="flex items-center gap-3 text-[var(--text-muted)]">
						<ShieldOff className="text-[var(--accent)]" />
						<p className="text-sm">
							Use this playbook as a baseline while the product is still in MVP
							mode.
						</p>
					</div>
					<ul className="list-inside list-disc space-y-2 text-sm text-[var(--text-muted)]">
						<li>Live ticker surfaces failing monitors immediately.</li>
						<li>
							Timeline records action items, responders, and follow-ups for each
							event.
						</li>
						<li>
							Export JSON or Markdown postmortems directly from this surface.
						</li>
					</ul>
				</SectionCard>
			</div>
		</main>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/incidents",
		component: IncidentsPage,
		getParentRoute: () => parentRoute,
	});
