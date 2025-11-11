import { createRoute } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";

function DashboardPage() {
	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-7xl mx-auto">
				<div className="border-4 border-black dark:border-white p-6 mb-8 bg-[#ff6633]">
					<h1 className="text-4xl mb-2">DASHBOARD</h1>
					<p className="font-mono text-sm normal-case">
						Real-time monitoring overview
					</p>
				</div>

				<div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<div className="text-5xl font-bold mb-2 text-[#ff6633]">0</div>
						<div className="text-sm font-bold uppercase">Total Monitors</div>
					</div>
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<div className="text-5xl font-bold mb-2 text-[#00ff00]">0</div>
						<div className="text-sm font-bold uppercase">Online</div>
					</div>
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<div className="text-5xl font-bold mb-2 text-[#ff0000]">0</div>
						<div className="text-sm font-bold uppercase">Down</div>
					</div>
					<div className="border-4 border-black dark:border-white p-6 bg-white dark:bg-black">
						<div className="text-5xl font-bold mb-2 text-[#ffff00]">0</div>
						<div className="text-sm font-bold uppercase">Incidents</div>
					</div>
				</div>

				<div className="border-4 border-black dark:border-white p-6 bg-black text-white">
					<h2 className="mb-4 text-[#ff6633]">RECENT ACTIVITY</h2>
					<pre className="font-mono text-xs">
						{`[INFO] No monitors configured yet
[INFO] Create your first monitor to start tracking uptime`}
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
		path: "/",
		component: DashboardPage,
		getParentRoute: () => parentRoute,
	});
