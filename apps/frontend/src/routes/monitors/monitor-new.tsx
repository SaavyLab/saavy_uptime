import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { ShieldCheck } from "lucide-react";
import { useAppForm } from "@/components/form/useAppForm";
import { Button } from "@/components/ui/button";
import { createMonitor } from "@/lib/monitors";
import type { RouterContext } from "@/router-context";
import {
	defaultMonitorFormValues,
	type MonitorFormValues,
} from "./monitor-form";

function MonitorNewPage() {
	const navigate = useNavigate({ from: "/monitors/new" });
	const defaultValues: MonitorFormValues = {
		...defaultMonitorFormValues,
	};

	const form = useAppForm({
		defaultValues,
		onSubmit: async ({ value, formApi }) => {
			await createMonitor({
				name: value.name,
				url: value.url,
				interval: value.interval,
				timeout: value.timeout,
				followRedirects: value.followRedirects,
				verifyTls: value.verifyTls,
			});
			formApi.reset();
			navigate({ to: "/monitors" });
		},
	});

	return (
		<div className="space-y-8">
			<div className="flex items-center justify-between">
				<div className="space-y-1">
					<h1 className="text-2xl font-bold tracking-tight">New Monitor</h1>
					<p className="text-muted-foreground">
						Provision HTTP/HTTPS coverage with a URL, interval, and timeout.
					</p>
				</div>
			</div>

			<div className="grid gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.7fr)]">
				<div className="rounded-xl border border-border bg-card p-6 shadow-sm sm:p-8">
					<form.AppForm>
						<form
							className="space-y-8"
							onSubmit={(event) => {
								event.preventDefault();
								void form.handleSubmit();
							}}
						>
							<div className="grid gap-6">
								<form.AppField
									name="name"
									validators={{
										onBlur: ({ value }) =>
											value?.trim().length ? undefined : "Name is required",
									}}
								>
									{(field) => (
										<field.TextField
											label="Monitor name"
											placeholder="My API Endpoint"
										/>
									)}
								</form.AppField>

								<form.AppField
									name="url"
									validators={{
										onBlur: ({ value }) => {
											if (!value?.trim()) {
												return "URL is required";
											}
											try {
												new URL(value);
												return undefined;
											} catch {
												return "Enter a valid URL";
											}
										},
									}}
								>
									{(field) => (
										<field.TextField
											label="URL"
											placeholder="https://api.example.com/health"
										/>
									)}
								</form.AppField>

								<div className="grid gap-6 md:grid-cols-2">
									<form.AppField
										name="interval"
										validators={{
											onBlur: ({ value }) =>
												value >= 15 ? undefined : "Min interval is 15 seconds",
										}}
									>
										{(field) => (
											<field.NumberField
												label="Check interval (seconds)"
												placeholder="60"
												min={15}
											/>
										)}
									</form.AppField>
									<form.AppField
										name="timeout"
										validators={{
											onBlur: ({ value }) =>
												value >= 1000 ? undefined : "Min timeout is 1000 ms",
										}}
									>
										{(field) => (
											<field.NumberField
												label="Timeout (milliseconds)"
												placeholder="5000"
												min={1000}
											/>
										)}
									</form.AppField>
								</div>

								<div className="rounded-xl border border-border bg-muted/20 p-6">
									<p className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
										Advanced options
									</p>
									<div className="mt-4 space-y-4">
										<form.AppField name="followRedirects">
											{(field) => (
												<field.BooleanSwitchField label="Follow redirects" />
											)}
										</form.AppField>
										<form.AppField name="verifyTls">
											{(field) => (
												<field.BooleanSwitchField label="Verify TLS certificate" />
											)}
										</form.AppField>
									</div>
								</div>
							</div>

							<div className="flex flex-col gap-3 sm:flex-row">
								<form.SubmitButton className="flex-1" label="Create monitor" />
								<Link to="/monitors" className="flex-1">
									<Button variant="secondary" className="w-full">
										Cancel
									</Button>
								</Link>
							</div>
						</form>
					</form.AppForm>
				</div>

				<aside className="rounded-xl border border-border bg-muted/10 p-6 shadow-sm space-y-6">
					<div className="rounded-lg border border-border bg-muted/20 p-6">
						<p className="text-xs uppercase tracking-wider font-medium text-muted-foreground">
							Deployment checklist
						</p>
						<ul className="mt-4 space-y-4 text-sm text-muted-foreground">
							<li>1. Reference org ID from the Organization panel.</li>
							<li>2. Use fully-qualified URLs—edge workers enforce TLS.</li>
							<li>
								3. Keep intervals ≥ 15s to stay within Durable Object budgets.
							</li>
							<li>
								4. Advanced options keep redirect and TLS toggles in reach so
								you can double-check behavior before saving.
							</li>
						</ul>
					</div>

					<div className="rounded-lg border border-border bg-muted/20 p-6">
						<div className="flex items-center gap-2 mb-2">
							<ShieldCheck size={18} className="text-emerald-500" />
							<p className="text-sm font-medium text-foreground">
								Saavy control plane
							</p>
						</div>
						<p className="text-sm text-muted-foreground">
							Each monitor streams status back into the Monitors, Incidents, and
							Status pages automatically—no extra wiring required.
						</p>
					</div>
				</aside>
			</div>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/monitors/new",
		component: MonitorNewPage,
		getParentRoute: () => parentRoute,
	});
