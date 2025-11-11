import { createRoute } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { Activity, CheckCircle } from "lucide-react";

function StatusPage() {
	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-5xl mx-auto">
				<div className="border-4 border-black dark:border-white p-8 mb-8 bg-[#00ff00] text-black">
					<div className="flex items-center gap-4 mb-4">
						<CheckCircle size={48} strokeWidth={3} />
						<div>
							<h1 className="text-5xl mb-1">ALL SYSTEMS OPERATIONAL</h1>
							<p className="font-mono text-sm">Last updated: Just now</p>
						</div>
					</div>
				</div>

				<div className="border-4 border-black dark:border-white p-8 bg-white dark:bg-black mb-8">
					<h2 className="mb-6 flex items-center gap-3">
						<Activity size={24} strokeWidth={3} />
						SERVICES
					</h2>

					<div className="space-y-4">
						{/* Placeholder for when monitors exist */}
						<div className="text-center py-8 text-muted-foreground">
							<p className="font-mono text-sm normal-case">
								No services configured yet
							</p>
						</div>
					</div>
				</div>

				<div className="border-4 border-black dark:border-white p-6 bg-black text-white">
					<h2 className="mb-4 text-[#ff6633]">// UPTIME STATISTICS</h2>
					<div className="grid grid-cols-3 gap-6 font-mono text-sm">
						<div>
							<div className="text-muted-foreground mb-1">24 HOURS</div>
							<div className="text-2xl font-bold text-[#00ff00]">100.00%</div>
						</div>
						<div>
							<div className="text-muted-foreground mb-1">7 DAYS</div>
							<div className="text-2xl font-bold text-[#00ff00]">100.00%</div>
						</div>
						<div>
							<div className="text-muted-foreground mb-1">30 DAYS</div>
							<div className="text-2xl font-bold text-[#00ff00]">100.00%</div>
						</div>
					</div>
				</div>

				<div className="mt-8 text-center">
					<p className="font-mono text-xs text-muted-foreground">
						Powered by CF-UPTIME • Cloudflare-Native Monitoring
					</p>
				</div>
			</div>
		</div>
	);
}

export default (
	parentRoute: RootRoute<Register, undefined, RouterContext>,
) =>
	createRoute({
		path: "/status",
		component: StatusPage,
		getParentRoute: () => parentRoute,
	});
