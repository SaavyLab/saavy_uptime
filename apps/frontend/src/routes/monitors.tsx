import { createRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { Plus } from "lucide-react";

function MonitorsPage() {
	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-7xl mx-auto">
				<div className="flex items-center justify-between mb-8">
					<div className="border-4 border-black dark:border-white p-6 flex-1 mr-4 bg-[#ff6633]">
						<h1 className="text-4xl mb-2">MONITORS</h1>
						<p className="font-mono text-sm normal-case">
							Configure and manage HTTP/HTTPS monitors
						</p>
					</div>
					<Link
						to="/monitors/new"
						className="border-4 border-black dark:border-white bg-[#00ff00] text-black p-6 hover:bg-[#00cc00] transition-colors flex items-center gap-3 font-bold uppercase"
					>
						<Plus size={24} strokeWidth={3} />
						New Monitor
					</Link>
				</div>

				<div className="border-4 border-black dark:border-white p-8 bg-white dark:bg-black">
					<div className="text-center py-12">
						<div className="text-6xl mb-4 text-muted-foreground">[ ]</div>
						<h3 className="text-2xl mb-2">NO MONITORS YET</h3>
						<p className="font-mono text-sm normal-case mb-6 text-muted-foreground">
							Create your first monitor to start tracking uptime
						</p>
						<Link
							to="/monitors/new"
							className="inline-block border-2 border-black dark:border-white bg-[#ff6633] text-black px-6 py-3 hover:bg-[#ff5500] transition-colors font-bold uppercase"
						>
							Create Monitor
						</Link>
					</div>
				</div>
			</div>
		</div>
	);
}

export default (
	parentRoute: RootRoute<Register, undefined, RouterContext>,
) =>
	createRoute({
		path: "/monitors",
		component: MonitorsPage,
		getParentRoute: () => parentRoute,
	});
