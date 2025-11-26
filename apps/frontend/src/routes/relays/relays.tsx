import { useMemo } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import { Globe2, MapPin, PlusCircle } from "lucide-react";
import { toast } from "sonner";
import { useAppForm } from "@/components/form/useAppForm";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { createRelay, getRelays, type Relay } from "@/lib/relays";
import type { RouterContext } from "@/router-context";

interface RelayFormValues {
	slug: string;
	name: string;
	locationHint: string;
}

const LOCATION_HINTS = [
	{ value: "wnam", label: "West North America" },
	{ value: "enam", label: "East North America" },
	{ value: "weur", label: "West Europe" },
	{ value: "eeur", label: "East Europe" },
	{ value: "apac", label: "Asia Pacific" },
	{ value: "oc", label: "Oceania" },
];

const defaultValues: RelayFormValues = {
	slug: "",
	name: "",
	locationHint: LOCATION_HINTS[0]?.value ?? "wnam",
};

function RelaysPage() {
	const queryClient = useQueryClient();
	const relaysQuery = useQuery<Relay[]>({
		queryKey: ["relays"],
		queryFn: () => getRelays(),
	});
	const relayError =
		relaysQuery.error instanceof Error ? relaysQuery.error.message : null;

	const createRelayMutation = useMutation({
		mutationFn: (values: RelayFormValues) =>
			createRelay({
				slug: values.slug.trim(),
				name: values.name.trim(),
				locationHint: values.locationHint,
			}),
		onSuccess: async (relay) => {
			toast.success("Relay created", {
				description: `${relay.name} (${relay.locationHint}) is ready for assignments`,
			});
			await queryClient.invalidateQueries({ queryKey: ["relays"] });
		},
		onError: (error: unknown) => {
			const message =
				error instanceof Error ? error.message : "Unable to create relay";
			toast.error(message);
		},
	});

	const form = useAppForm({
		defaultValues,
		onSubmit: async ({ value, formApi }) => {
			await createRelayMutation.mutateAsync(value);
			formApi.reset();
		},
	});

	const relays = relaysQuery.data ?? [];
	const hasRelays = relays.length > 0;

	const sortedRelays = useMemo(
		() => [...relays].sort((a, b) => b.createdAt - a.createdAt),
		[relays],
	);

	return (
		<div className="space-y-8">
			<div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
				<div>
					<h1 className="text-2xl font-bold tracking-tight">Relays</h1>
					<p className="text-muted-foreground">
						Pin Durable Objects to specific Cloudflare regions and route
						monitors through them.
					</p>
				</div>
				<Button asChild variant="secondary">
					<Link to="/monitors/new">Create monitor</Link>
				</Button>
			</div>

			<div className="grid gap-8 lg:grid-cols-[minmax(0,1.2fr)_minmax(320px,0.7fr)]">
				<Card className="overflow-hidden">
					<CardHeader>
						<CardTitle className="flex items-center gap-2 text-base font-semibold">
							<Globe2 className="h-4 w-4 text-primary" />
							Configured Relays
						</CardTitle>
					</CardHeader>
					<CardContent>
						{relaysQuery.isLoading ? (
							<p className="text-sm text-muted-foreground">Loading relays…</p>
						) : relayError ? (
							<div className="rounded-lg border border-destructive/40 bg-destructive/5 p-4 text-sm text-destructive">
								<p className="font-medium">Unable to load relays</p>
								<p>{relayError}</p>
							</div>
						) : hasRelays ? (
							<div className="space-y-4">
								{sortedRelays.map((relay) => (
									<div
										key={relay.id}
										className="flex flex-col gap-3 rounded-xl border border-border/70 bg-muted/10 p-4 sm:flex-row sm:items-center sm:justify-between"
									>
										<div>
											<p className="text-sm font-semibold text-foreground">
												{relay.name}
											</p>
											<p className="text-xs text-muted-foreground">
												Slug: {relay.slug}
											</p>
										</div>
										<div className="flex flex-wrap gap-2 text-xs text-muted-foreground">
											<span className="inline-flex items-center gap-1 rounded-full border border-border px-2 py-1">
												<MapPin className="h-3 w-3" />
												{relay.locationHint}
											</span>
											<span className="inline-flex items-center gap-1 rounded-full border border-border px-2 py-1">
												Jurisdiction: {relay.jurisdiction}
											</span>
										</div>
										<div className="text-xs text-muted-foreground">
											<p>DO ID: {relay.durableObjectId.slice(0, 12)}…</p>
											<p>
												Bootstrapped:{" "}
												{formatTimestamp(relay.lastBootstrappedAt)}
											</p>
										</div>
									</div>
								))}
							</div>
						) : (
							<div className="rounded-xl border border-dashed border-border/70 p-8 text-center">
								<p className="font-medium text-foreground">No relays yet</p>
								<p className="text-sm text-muted-foreground">
									Create a relay on the right to start assigning monitors to
									specific regions.
								</p>
							</div>
						)}
					</CardContent>
				</Card>

				<Card>
					<CardHeader>
						<CardTitle className="flex items-center gap-2 text-base font-semibold">
							<PlusCircle className="h-4 w-4 text-primary" />
							New Relay
						</CardTitle>
					</CardHeader>
					<CardContent>
						<form.AppForm>
							<form
								className="space-y-6"
								onSubmit={(event) => {
									event.preventDefault();
									void form.handleSubmit();
								}}
							>
								<form.AppField
									name="name"
									validators={{
										onBlur: ({ value }) =>
											value?.trim().length ? undefined : "Name is required",
									}}
								>
									{(field) => (
										<field.TextField
											label="Display name"
											placeholder="WNAM Relay"
										/>
									)}
								</form.AppField>

								<form.AppField
									name="slug"
									validators={{
										onBlur: ({ value }) =>
											value?.trim().length ? undefined : "Slug is required",
									}}
								>
									{(field) => (
										<field.TextField label="Slug" placeholder="wnam" />
									)}
								</form.AppField>

								<form.AppField name="locationHint">
									{(field) => (
										<div className="space-y-2">
											<Label className="text-sm font-medium">
												Location hint
											</Label>
											<Select
												value={field.state.value ?? ""}
												onValueChange={field.handleChange}
											>
												<SelectTrigger className="w-full">
													<SelectValue placeholder="Select a region" />
												</SelectTrigger>
												<SelectContent>
													{LOCATION_HINTS.map((hint) => (
														<SelectItem key={hint.value} value={hint.value}>
															{hint.label} ({hint.value})
														</SelectItem>
													))}
												</SelectContent>
											</Select>
											{field.state.meta.errors[0] ? (
												<p className="text-sm text-destructive">
													{field.state.meta.errors[0]}
												</p>
											) : null}
										</div>
									)}
								</form.AppField>

								<div className="rounded-lg border border-dashed border-border/70 bg-muted/10 p-3 text-xs text-muted-foreground">
									<p className="font-semibold text-foreground text-sm mb-1">
										Jurisdiction policy
									</p>
									<p>
										European hints (WEUR/EEUR) pin the Durable Object to the EU
										to match data residency; other regions remain global.
									</p>
								</div>

								<form.SubmitButton
									label={
										createRelayMutation.isPending ? "Creating…" : "Create relay"
									}
									className="w-full"
								/>
							</form>
						</form.AppForm>
					</CardContent>
				</Card>
			</div>
		</div>
	);
}

const formatTimestamp = (value: number | null | undefined): string => {
	if (!value) {
		return "—";
	}
	return new Date(value).toLocaleString();
};

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/relays",
		component: RelaysPage,
		getParentRoute: () => parentRoute,
	});
