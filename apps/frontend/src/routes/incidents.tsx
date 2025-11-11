import { createRoute } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { AlertCircle } from "lucide-react";

function IncidentsPage() {
	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-7xl mx-auto">
				<div className="border-4 border-black dark:border-white p-6 mb-8 bg-[#ff6633]">
					<h1 className="text-4xl mb-2">INCIDENTS</h1>
					<p className="font-mono text-sm normal-case">
						Monitor downtime and incident history
					</p>
				</div>

				<div className="border-4 border-black dark:border-white p-8 bg-white dark:bg-black">
					<div className="text-center py-12">
						<AlertCircle
							size={64}
							strokeWidth={2}
							className="mx-auto mb-4 text-muted-foreground"
						/>
						<h3 className="text-2xl mb-2">NO INCIDENTS</h3>
						<p className="font-mono text-sm normal-case text-muted-foreground">
							All systems operational
						</p>
					</div>
				</div>

				<div className="mt-8 border-4 border-black dark:border-white p-6 bg-black text-white">
					<h2 className="mb-4 text-[#ff6633]">// INCIDENT METRICS</h2>
					<pre className="font-mono text-xs">
						{`MTTR (Mean Time To Recovery): --
Total Incidents (30d):         0
Average Downtime:               0m
Longest Incident:               --`}
					</pre>
				</div>
			</div>
		</div>
	);
}

export default (
	parentRoute: RootRoute<Register, undefined, RouterContext>,
) =>
	createRoute({
		path: "/incidents",
		component: IncidentsPage,
		getParentRoute: () => parentRoute,
	});
