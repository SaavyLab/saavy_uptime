import { useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { Building2, RefreshCcw, Sparkles } from "lucide-react";
import { useId, useState } from "react";
import { useAppForm } from "@/components/form/useAppForm";
import { Hero } from "@/components/layout/Hero";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	createOrganization,
	getOrganization,
} from "@/lib/organizations";
import type { RouterContext } from "@/router-context";

const formatTimestamp = (value?: number) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

function OrganizationPage() {
	const lookupFieldId = useId();
	const [lookupId, setLookupId] = useState("");
	const [activeOrgId, setActiveOrgId] = useState("");
	const [creationError, setCreationError] = useState<string | null>(null);
	const [creationSuccess, setCreationSuccess] = useState<string | null>(null);

	const organizationQuery = useQuery({
		queryKey: ["organization", activeOrgId],
		queryFn: () => getOrganization(activeOrgId),
		enabled: Boolean(activeOrgId),
	});

	const orgForm = useAppForm({
		defaultValues: {
			name: "",
			slug: "",
		},
		onSubmit: async ({ value, formApi }) => {
			setCreationError(null);
			setCreationSuccess(null);
			try {
				const organization = await createOrganization({
					name: value.name.trim(),
					slug: value.slug.trim(),
				});
				setLookupId(organization.id);
				setActiveOrgId(organization.id);
				setCreationSuccess(
					`Created ${organization.name} (${organization.slug})`,
				);
				formApi.reset();
			} catch (error) {
				const message =
					error instanceof Error
						? error.message
						: "Failed to create organization";
				setCreationError(message);
			}
		},
	});

	const handleLookup = (event: React.FormEvent<HTMLFormElement>) => {
		event.preventDefault();
		if (!lookupId.trim()) {
			return;
		}
		setActiveOrgId(lookupId.trim());
	};

	const organization = organizationQuery.data;
	const isLoadingOrganization = organizationQuery.isFetching;
	const organizationError =
		organizationQuery.error instanceof Error ? organizationQuery.error : null;

	return (
		<main className="min-h-screen bg-[var(--surface)] px-6 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto max-w-6xl space-y-8">
				<Hero
					eyebrow="Organization"
					title="Tenant controls for Saavy Uptime"
					description="Look up an org or mint a new one for testing without leaving this pane."
					sideContent={
						<div className="rounded-2xl border border-white/10 bg-black/30 p-4 w-fit">
							<Building2 size={32} />
						</div>
					}
				/>

				<div className="grid gap-8 lg:grid-cols-2">
					<div className="rounded-[32px] border border-white/10 bg-white/[0.02]">
						<div className="border-b border-white/10 px-6 py-5">
							<div className="flex items-center justify-between">
								<div>
									<p className="text-xs uppercase tracking-[0.4em] text-[var(--text-soft)]">
										Lookup
									</p>
									<p className="text-sm text-[var(--text-muted)]">
										Inspect current organization data.
									</p>
								</div>
								{isLoadingOrganization ? (
									<RefreshCcw
										className="animate-spin text-[var(--accent-green)]"
										size={18}
									/>
								) : null}
							</div>
							<form onSubmit={handleLookup} className="mt-4 space-y-4">
								<div className="space-y-2">
									<Label htmlFor={lookupFieldId} className="tracking-[0.3em]">
										Organization ID
									</Label>
									<div className="flex flex-col gap-3 sm:flex-row">
										<Input
											id={lookupFieldId}
											placeholder="cjwtc8example"
											value={lookupId}
											onChange={(event) => setLookupId(event.target.value)}
											autoComplete="off"
										/>
										<Button
											type="submit"
											variant="secondary"
											disabled={!lookupId.trim()}
											className="sm:w-32"
										>
											Load
										</Button>
									</div>
									<p className="text-xs text-[var(--text-muted)]">
										Tip: after creating an org the ID appears here
										automatically.
									</p>
								</div>
							</form>
						</div>
						<div className="px-6 py-6">
							{!activeOrgId ? (
								<p className="text-sm text-[var(--text-muted)]">
									No organization selected yet. Enter an ID or use the form on
									the right to create one.
								</p>
							) : isLoadingOrganization ? (
								<p className="text-sm text-[var(--accent-green)]">
									Loading organization…
								</p>
							) : organizationError ? (
								<p className="text-sm text-[var(--accent-red)]">
									{organizationError.message}
								</p>
							) : organization ? (
								<div className="space-y-4">
									<div>
										<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
											Name
										</p>
										<p className="text-xl font-semibold">{organization.name}</p>
									</div>
									<div>
										<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
											Slug
										</p>
										<p className="font-mono text-sm text-[var(--text-muted)]">
											{organization.slug}
										</p>
									</div>
									<div>
										<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
											ID
										</p>
										<p className="font-mono text-xs text-[var(--text-muted)] break-all">
											{organization.id}
										</p>
									</div>
									<div>
										<p className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
											Created
										</p>
										<p className="font-mono text-sm text-[var(--text-muted)]">
											{formatTimestamp(organization.created_at)}
										</p>
									</div>
								</div>
							) : null}
						</div>
					</div>

					<div className="rounded-[32px] border border-white/10 bg-white/[0.02] p-6 shadow-[var(--shadow-soft)]">
						<div className="flex items-center gap-3">
							<Sparkles size={20} className="text-[var(--accent)]" />
							<div>
								<p className="text-xs uppercase tracking-[0.4em] text-[var(--text-soft)]">
									Create organization
								</p>
								<p className="text-sm text-[var(--text-muted)]">
									Slug must be unique per environment.
								</p>
							</div>
						</div>

						<orgForm.AppForm>
							<form
								className="mt-6 space-y-6"
								onSubmit={(event) => {
									event.preventDefault();
									void orgForm.handleSubmit();
								}}
							>
								<orgForm.AppField
									name="name"
									validators={{
										onBlur: ({ value }) => {
											const trimmed = value?.trim() ?? "";
											if (!trimmed) {
												return "Name is required";
											}
											return undefined;
										},
									}}
								>
									{(field) => (
										<field.TextField
											label="Display name"
											placeholder="Saavy Internal"
										/>
									)}
								</orgForm.AppField>

								<orgForm.AppField
									name="slug"
									validators={{
										onBlur: ({ value }) => {
											const trimmed = value?.trim() ?? "";
											if (!trimmed) {
												return "Slug is required";
											}
											if (!/^[a-z0-9-]+$/.test(trimmed)) {
												return "Use lowercase letters, numbers, and dashes only";
											}
											return undefined;
										},
									}}
								>
									{(field) => (
										<field.TextField
											label="Slug"
											placeholder="saavy"
											description="Used in URLs and APIs"
										/>
									)}
								</orgForm.AppField>

								{creationError ? (
									<p className="text-sm text-[var(--accent-red)]" role="alert">
										{creationError}
									</p>
								) : null}
								{creationSuccess ? (
									<p
										className="text-sm text-[var(--accent-green)]"
										aria-live="polite"
									>
										{creationSuccess}
									</p>
								) : null}

								<orgForm.SubmitButton
									className="w-full"
									label="Create organization"
								/>
							</form>
						</orgForm.AppForm>
					</div>
				</div>
			</div>
		</main>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/organization",
		component: OrganizationPage,
		getParentRoute: () => parentRoute,
	});
