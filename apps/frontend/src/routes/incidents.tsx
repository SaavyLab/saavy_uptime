import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { AlertOctagon, ShieldOff } from "lucide-react";

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
		<div className="space-y-8">
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <h1 className="text-2xl font-bold tracking-tight">Incidents</h1>
          <p className="text-muted-foreground">
            Incident log and response coordination for Saavy Uptime.
          </p>
        </div>
        <Button variant="secondary">Export History</Button>
      </div>

			<SectionCard title="Incident metrics">
				<StatsGrid
					items={metrics}
					className="sm:grid-cols-2"
					cardClassName="bg-muted/20 border-border"
				/>
			</SectionCard>

			<SectionCard contentClassName="space-y-4 text-center">
				<AlertOctagon size={48} className="mx-auto text-muted-foreground" />
				<h2 className="text-2xl font-medium">All systems operational</h2>
				<p className="text-sm text-muted-foreground">
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
				<div className="flex items-center gap-3 text-muted-foreground">
					<ShieldOff className="text-primary" />
					<p className="text-sm">
						Use this playbook as a baseline while the product is still in MVP
						mode.
					</p>
				</div>
				<ul className="list-inside list-disc space-y-2 text-sm text-muted-foreground">
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
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/incidents",
		component: IncidentsPage,
		getParentRoute: () => parentRoute,
	});
