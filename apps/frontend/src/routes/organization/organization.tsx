import { useState } from "react";
import { createRoute } from "@tanstack/react-router";
import type { Register, RootRoute } from "@tanstack/react-router";
import type { RouterContext } from "@/router-context";
import { Building2, RefreshCcw, Sparkles } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import { useAppForm } from "@/components/form/useAppForm";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	createOrganization,
	getOrganization,
	type CreateOrganizationInput,
} from "@/lib/organizations";

const formatTimestamp = (value?: number) => {
	if (!value) {
		return "—";
	}

	return new Date(value * 1000).toLocaleString();
};

function OrganizationPage() {
	const [lookupId, setLookupId] = useState("");
	const [activeOrgId, setActiveOrgId] = useState("");
	const [creationError, setCreationError] = useState<string | null>(null);
	const [creationSuccess, setCreationSuccess] = useState<string | null>(null);

	const organizationQuery = useQuery({
		queryKey: ["organization", activeOrgId],
		queryFn: () => getOrganization(activeOrgId),
		enabled: Boolean(activeOrgId),
	});

	const orgForm = useAppForm<CreateOrganizationInput>({
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
				setCreationSuccess(`Created ${organization.name} (${organization.slug})`);
				formApi.reset();
			} catch (error) {
				const message =
					error instanceof Error ? error.message : "Failed to create organization";
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
		<div className="min-h-screen bg-white dark:bg-black text-black dark:text-white p-8">
			<div className="max-w-6xl mx-auto space-y-8">
				<div className="border-4 border-black dark:border-white p-6 bg-[#ff6633] flex items-center gap-4">
					<div className="p-4 bg-black text-white border-4 border-black">
						<Building2 size={32} strokeWidth={3} />
					</div>
					<div>
						<h1 className="text-4xl mb-1 uppercase">Organization</h1>
						<p className="font-mono text-sm normal-case">
							Look up an org or mint a new one for testing
						</p>
					</div>
				</div>

				<div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
					<div className="border-4 border-black dark:border-white bg-white dark:bg-black">
						<div className="border-b-4 border-black dark:border-white p-6 bg-black text-white flex items-center justify-between">
							<div>
								<h2 className="text-2xl uppercase">Lookup</h2>
								<p className="text-sm text-muted-foreground">
									Paste an organization ID to inspect current data
								</p>
							</div>
							<RefreshCcw
								size={24}
								strokeWidth={2.5}
								className={isLoadingOrganization ? "animate-spin text-[#00ff00]" : ""}
							/>
						</div>

						<form onSubmit={handleLookup} className="p-6 space-y-4">
							<div className="space-y-2">
								<Label htmlFor="organization-id" className="font-semibold tracking-wide">
									Organization ID
								</Label>
								<div className="flex gap-3">
									<Input
										id="organization-id"
										placeholder="cjwtc8example"
										value={lookupId}
										onChange={(event) => setLookupId(event.target.value)}
										autoComplete="off"
									/>
									<Button
										type="submit"
										className="uppercase font-bold"
										disabled={!lookupId.trim()}
									>
										Load
									</Button>
								</div>
								<p className="text-xs text-muted-foreground">
									Tip: after creating an org the ID appears here automatically.
								</p>
							</div>
						</form>

						<div className="border-t-4 border-black dark:border-white p-6 space-y-4">
							{!activeOrgId ? (
								<p className="font-mono text-sm text-muted-foreground">
									No organization selected yet. Enter an ID or use the form on the right to
									create one.
								</p>
							) : isLoadingOrganization ? (
								<p className="font-mono text-sm text-[#00ff00]">Loading organization…</p>
							) : organizationError ? (
								<p className="font-mono text-sm text-red-500">
									{organizationError.message}
								</p>
							) : organization ? (
								<div className="space-y-2">
									<div>
										<p className="text-sm text-muted-foreground uppercase">Name</p>
										<p className="font-bold text-2xl">{organization.name}</p>
									</div>
									<div>
										<p className="text-sm text-muted-foreground uppercase">Slug</p>
										<p className="font-mono text-lg">{organization.slug}</p>
									</div>
									<div>
										<p className="text-sm text-muted-foreground uppercase">ID</p>
										<p className="font-mono text-lg break-all">{organization.id}</p>
									</div>
									<div>
										<p className="text-sm text-muted-foreground uppercase">Created</p>
										<p className="font-mono text-lg">
											{formatTimestamp(organization.created_at)}
										</p>
									</div>
								</div>
							) : null}
						</div>
					</div>

					<div className="border-4 border-black dark:border-white bg-white dark:bg-black">
						<div className="border-b-4 border-black dark:border-white p-6 bg-[#00ff00] text-black flex items-center gap-2">
							<Sparkles size={24} strokeWidth={3} />
							<div>
								<h2 className="text-2xl uppercase">Create Organization</h2>
								<p className="text-sm font-mono normal-case">
									Slug must be unique per environment
								</p>
							</div>
						</div>

						<orgForm.AppForm>
							<form
								className="p-6 space-y-6"
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
											label="Display Name"
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
									<p className="text-sm text-red-500" role="alert">
										{creationError}
									</p>
								) : null}
								{creationSuccess ? (
									<p className="text-sm text-green-600" role="status">
										{creationSuccess}
									</p>
								) : null}

								<orgForm.SubmitButton
									className="w-full uppercase font-bold"
									label="Create Organization"
								/>
							</form>
						</orgForm.AppForm>
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
		path: "/organization",
		component: OrganizationPage,
		getParentRoute: () => parentRoute,
	});
