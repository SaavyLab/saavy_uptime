import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { Pencil } from "lucide-react";
import { toast } from "sonner";
import { useAppForm } from "@/components/form/useAppForm";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/Skeleton";
import { getMonitor, updateMonitor, type Monitor } from "@/lib/monitors";
import type { RouterContext } from "@/router-context";
import {
	defaultMonitorFormValues,
	type MonitorFormValues,
} from "./monitor-form";

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) => {
	const route = createRoute({
		path: "/monitors/$monitorId/edit",
		getParentRoute: () => parentRoute,
		component: MonitorEditPage,
	});

	function MonitorEditPage() {
		const { monitorId } = route.useParams();
		const navigate = useNavigate({ from: "/monitors/$monitorId/edit" });
		const queryClient = useQueryClient();

		const monitorQuery = useQuery({
			queryKey: ["monitor", monitorId],
			queryFn: () => getMonitor(monitorId),
		});

		const updateMutation = useMutation({
			mutationFn: (values: MonitorFormValues) =>
				updateMonitor(monitorId, {
					name: values.name,
					url: values.url,
					interval: values.interval,
					timeout: values.timeout,
					followRedirects: values.followRedirects,
					verifyTls: values.verifyTls,
				}),
			onSuccess: async (monitor) => {
				toast.success("Monitor updated", {
					description: `${monitor.name} saved successfully`,
				});
				await Promise.all([
					queryClient.invalidateQueries({ queryKey: ["monitors"] }),
					queryClient.invalidateQueries({ queryKey: ["monitor", monitorId] }),
				]);
				navigate({ to: "/monitors" });
			},
			onError: (error: unknown) => {
				const message =
					error instanceof Error ? error.message : "Unable to update monitor";
				toast.error(message);
			},
		});

		const form = useAppForm({
			defaultValues: defaultMonitorFormValues,
			onSubmit: async ({ value }) => {
				await updateMutation.mutateAsync(value);
			},
		});

		useEffect(() => {
			if (monitorQuery.data) {
				form.reset(mapMonitorToFormValues(monitorQuery.data), {
					keepDefaultValues: true,
				});
			}
		}, [monitorQuery.data, form]);

		const isLoading = monitorQuery.isLoading;
		const loadError =
			monitorQuery.error instanceof Error ? monitorQuery.error : null;

		return (
			<div className="space-y-8">
				<div className="flex items-center justify-between">
					<div className="space-y-1">
						<h1 className="text-2xl font-bold tracking-tight">Edit Monitor</h1>
						<p className="text-muted-foreground">
							Refine monitor behavior without redeploying Workers.
						</p>
					</div>
					<Button
						type="button"
						variant="secondary"
						onClick={() => navigate({ to: "/monitors" })}
					>
						Cancel
					</Button>
				</div>

				<div className="grid gap-8 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.7fr)]">
					<div className="rounded-xl border border-border bg-card p-6 shadow-sm sm:p-8">
						{isLoading ? (
							<div className="space-y-4">
								<Skeleton className="h-16" />
								<Skeleton className="h-16" />
								<Skeleton className="h-32" />
							</div>
						) : loadError ? (
							<div className="space-y-4">
								<p className="font-mono text-sm text-destructive">
									{loadError.message}
								</p>
								<Button type="button" onClick={() => monitorQuery.refetch()}>
									Try again
								</Button>
							</div>
						) : (
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
														value >= 15
															? undefined
															: "Min interval is 15 seconds",
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
														value >= 1000
															? undefined
															: "Min timeout is 1000 ms",
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
										<form.SubmitButton
											className="flex-1"
											label={
												updateMutation.isPending
													? "Saving changes..."
													: "Save changes"
											}
										/>
										<Link to="/monitors" className="flex-1">
											<Button variant="secondary" className="w-full">
												Cancel
											</Button>
										</Link>
									</div>
								</form>
							</form.AppForm>
						)}
					</div>

					<aside className="rounded-xl border border-border bg-muted/10 p-6 shadow-sm">
						<div className="rounded-lg border border-border bg-muted/20 p-6">
							<div className="flex items-center gap-2 mb-2">
								<Pencil size={18} className="text-primary" />
								<p className="text-sm font-medium text-foreground">
									Live updates
								</p>
							</div>
							<p className="text-sm text-muted-foreground">
								Changes apply instantly to the Durable Object scheduler. No
								worker restarts required.
							</p>
						</div>
					</aside>
				</div>
			</div>
		);
	}

	return route;
};

const mapMonitorToFormValues = (monitor: Monitor): MonitorFormValues => ({
	name: monitor.name,
	url: monitor.url,
	interval: monitor.intervalS,
	timeout: monitor.timeoutMs,
	followRedirects: Boolean(monitor.followRedirects),
	verifyTls: Boolean(monitor.verifyTls),
});
