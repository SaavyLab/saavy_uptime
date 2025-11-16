import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import { ArrowLeft, Pencil } from "lucide-react";
import { toast } from "sonner";
import { useAppForm } from "@/components/form/useAppForm";
import { Hero } from "@/components/layout/Hero";
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
			<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
				<div className="mx-auto max-w-5xl space-y-10">
					<Link
						to="/monitors"
						className="inline-flex items-center gap-2 text-sm text-[var(--text-muted)] transition hover:text-[var(--text-primary)]"
					>
						<ArrowLeft size={16} />
						Back to monitors
					</Link>

					<Hero
						eyebrow="Edit monitor"
						title="Refine monitor behavior"
						description="Adjust cadence, timeouts, and URL targets without redeploying Workers. Changes apply instantly to the Durable Object scheduler."
						actions={
							<Button
								type="button"
								variant="secondary"
								onClick={() => navigate({ to: "/monitors" })}
							>
								Cancel
							</Button>
						}
						sideContent={
							<div className="rounded-2xl border border-white/10 bg-black/30 p-4">
								<div className="flex items-center gap-3">
									<Pencil size={18} className="text-[var(--accent)]" />
									<div>
										<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
											Live updates
										</p>
										<p className="text-sm text-[var(--text-muted)]">
											No worker restarts required—updates propagate instantly.
										</p>
									</div>
								</div>
							</div>
						}
					/>

					<section className="rounded-[32px] border border-white/10 bg-white/[0.02] p-6 shadow-[var(--shadow-soft)] sm:p-8">
						{isLoading ? (
							<div className="space-y-4">
								<Skeleton className="h-16" />
								<Skeleton className="h-16" />
								<Skeleton className="h-32" />
							</div>
						) : loadError ? (
							<div className="space-y-4">
								<p className="font-mono text-sm text-[var(--accent-red)]">
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
													value?.trim().length
														? undefined
														: "Name is required",
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

										<div className="rounded-3xl border border-white/15 bg-black/30 p-6">
											<p className="text-xs font-mono uppercase tracking-[0.4em] text-[var(--text-soft)]">
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
					</section>
				</div>
			</main>
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
