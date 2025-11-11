import { createRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

function MonitorNewPage() {
	return (
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-4xl mx-auto">
				<Link
					to="/monitors"
					className="inline-flex items-center gap-2 mb-6 font-bold uppercase hover:text-[#ff6633] transition-colors"
				>
					<ArrowLeft size={20} strokeWidth={3} />
					Back to Monitors
				</Link>

				<div className="border-4 border-black dark:border-white p-6 mb-8 bg-[#ff6633]">
					<h1 className="text-4xl mb-2">NEW MONITOR</h1>
					<p className="font-mono text-sm normal-case">
						Configure a new HTTP/HTTPS monitor
					</p>
				</div>

				<form className="border-4 border-black dark:border-white p-8 bg-white dark:bg-black space-y-6">
					<div>
						<Label htmlFor="name">Monitor Name</Label>
						<Input
							id="name"
							type="text"
							placeholder="My API Endpoint"
							className="mt-2"
						/>
					</div>

					<div>
						<Label htmlFor="url">URL</Label>
						<Input
							id="url"
							type="url"
							placeholder="https://api.example.com/health"
							className="mt-2"
						/>
					</div>

					<div className="grid grid-cols-2 gap-4">
						<div>
							<Label htmlFor="interval">Check Interval (seconds)</Label>
							<Input
								id="interval"
								type="number"
								placeholder="60"
								defaultValue="60"
								className="mt-2"
							/>
						</div>
						<div>
							<Label htmlFor="timeout">Timeout (milliseconds)</Label>
							<Input
								id="timeout"
								type="number"
								placeholder="5000"
								defaultValue="5000"
								className="mt-2"
							/>
						</div>
					</div>

					<div className="border-4 border-black dark:border-white p-6 bg-black text-white">
						<h3 className="mb-4 text-[#ff6633]">// ADVANCED OPTIONS</h3>
						<div className="space-y-4">
							<div className="flex items-center gap-4 font-mono text-sm">
								<input type="checkbox" id="followRedirects" />
								<label htmlFor="followRedirects">Follow Redirects</label>
							</div>
							<div className="flex items-center gap-4 font-mono text-sm">
								<input type="checkbox" id="verifyTls" defaultChecked />
								<label htmlFor="verifyTls">Verify TLS Certificate</label>
							</div>
						</div>
					</div>

					<div className="flex gap-4">
						<Button type="submit" className="flex-1">
							Create Monitor
						</Button>
						<Link to="/monitors">
							<Button type="button" variant="outline">
								Cancel
							</Button>
						</Link>
					</div>
				</form>
			</div>
		</div>
	);
}

export default (
	parentRoute: RootRoute<Register, undefined, RouterContext>,
) =>
	createRoute({
		path: "/monitors/new",
		component: MonitorNewPage,
		getParentRoute: () => parentRoute,
	});
