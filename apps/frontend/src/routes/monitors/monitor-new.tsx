import { createRoute, Link, useNavigate } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { createMonitor } from "@/lib/monitors";
import { useAppForm } from "@/components/form/useAppForm";

type MonitorFormValues = {
	orgId: string;
	name: string;
	url: string;
	interval: number;
	timeout: number;
	followRedirects: boolean;
	verifyTls: boolean;
};

function MonitorNewPage() {
	const navigate = useNavigate({ from: "/monitors/new" });
	const defaultValues: MonitorFormValues = {
		orgId: "",
		name: "",
		url: "",
		interval: 60,
		timeout: 5000,
		followRedirects: true,
		verifyTls: true,
	};

	const form = useAppForm({
		defaultValues,
		onSubmit: async ({ value, formApi }) => {
			await createMonitor({
				orgId: value.orgId.trim(),
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
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-4xl mx-auto space-y-8">
				<Link
					to="/monitors"
					className="inline-flex items-center gap-2 font-bold uppercase hover:text-[#ff6633] transition-colors"
				>
					<ArrowLeft size={20} strokeWidth={3} />
					Back to Monitors
				</Link>

				<div className="border-4 border-black dark:border-white p-6 bg-[#ff6633]">
					<h1 className="text-4xl mb-2">NEW MONITOR</h1>
					<p className="font-mono text-sm normal-case">
						Configure a new HTTP/HTTPS monitor
					</p>
				</div>

				<form.AppForm>
					<form
						className="border-4 border-black dark:border-white p-8 bg-white dark:bg-black space-y-8"
						onSubmit={(event) => {
							event.preventDefault();
							void form.handleSubmit();
						}}
					>
						<div className="space-y-6">
							<form.AppField
								name="orgId"
								validators={{
									onBlur: ({ value }) => {
										if (!value?.trim()) {
											return "Organization ID is required";
										}
										return undefined;
									},
								}}
							>
								{(field) => (
									<field.TextField
										label="Organization ID"
										placeholder="cz3exampleorgid"
										description="Paste the ID from the Organization page"
									/>
								)}
							</form.AppField>

							<form.AppField
								name="name"
								validators={{
					onBlur: ({ value }) =>
						value?.trim().length ? undefined : "Name is required",
								}}
							>
								{(field) => (
									<field.TextField
										label="Monitor Name"
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

							<div className="grid grid-cols-1 md:grid-cols-2 gap-6">
								<form.AppField
									name="interval"
									validators={{
						onBlur: ({ value }) =>
							value >= 15 ? undefined : "Min interval is 15 seconds",
								}}
								>
									{(field) => (
										<field.NumberField
											label="Check Interval (seconds)"
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

							<div className="border-4 border-black dark:border-white p-6 bg-black text-white space-y-6">
								<h3 className="text-[#ff6633] font-mono text-sm tracking-widest">
									// ADVANCED OPTIONS
								</h3>
								<form.AppField name="followRedirects">
									{(field) => (
										<field.BooleanSwitchField label="Follow Redirects" />
									)}
								</form.AppField>
								<form.AppField name="verifyTls">
									{(field) => (
										<field.BooleanSwitchField label="Verify TLS certificate" />
									)}
								</form.AppField>
							</div>
						</div>

						<div className="flex gap-4">
							<form.SubmitButton label="Create Monitor" className="flex-1" />
							<Link to="/monitors" className="flex-1">
								<Button type="button" variant="outline" className="w-full">
									Cancel
								</Button>
							</Link>
						</div>
					</form>
				</form.AppForm>
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
