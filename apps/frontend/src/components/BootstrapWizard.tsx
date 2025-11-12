import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { useAppForm } from "./form/useAppForm";
import {
	invalidateBootstrapStatus,
	provisionOrganization,
} from "@/lib/bootstrap";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { toast } from "sonner";

const bootstrapSchema = z.object({
	name: z.string().min(1, "Name is required"),
	slug: z.string().min(1, "Slug is required"),
});

export type BootstrapWizardFormValues = z.infer<typeof bootstrapSchema>;

interface BootstrapWizardProps {
	suggestedSlug: string;
	email: string;
}

export default function BootstrapWizard({
	suggestedSlug,
	email,
}: BootstrapWizardProps) {
	const navigate = useNavigate();
	const queryClient = useQueryClient();

	const mutation = useMutation({
		mutationFn: ({ name, slug }: BootstrapWizardFormValues) =>
			provisionOrganization(name.trim(), slug.trim()),
		onSuccess: async () => {
			toast.success("Organization created", {
				description: "Redirecting to dashboard...",
			});
			await invalidateBootstrapStatus(queryClient);
			navigate({ to: "/" });
		},
		onError: (error: Error) => {
			toast.error("Unable to create organization", {
				description: error.message,
			});
		},
	});

	const form = useAppForm({
		defaultValues: {
			name: "",
			slug: suggestedSlug,
		},
		onSubmit: async ({ value }) => {
			await mutation.mutateAsync(value);
		},
	});

	return (
		<main className="min-h-screen bg-[var(--surface)] px-4 py-10 text-[var(--text-primary)] lg:px-8">
			<div className="mx-auto flex max-w-5xl flex-col gap-6">
				<div className="space-y-3">
					<p className="text-sm font-medium uppercase tracking-wide text-[var(--text-muted)]">
						Onboarding
					</p>
					<h1 className="text-3xl font-semibold tracking-tight">
						Welcome to Saavy Uptime
					</h1>
					<p className="text-base text-[var(--text-muted)]">
						You’re signed in as{" "}
						<span className="font-semibold text-[var(--text-primary)]">
							{email}
						</span>
						. Create your first organization to unlock monitors, incidents, and
						dashboards.
					</p>
				</div>

				<Card className="border-[var(--border-subtle)] bg-[var(--surface-strong)]">
					<CardHeader>
						<CardTitle className="text-2xl">Create your organization</CardTitle>
						<CardDescription>
							We suggest a slug based on your Access team. You can update the
							name, slug, and other settings later.
						</CardDescription>
					</CardHeader>
					<CardContent className="space-y-6">
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
										onBlur: ({ value }) => {
											if (!value?.trim()) {
												return "Name is required";
											}
											return undefined;
										},
									}}
								>
									{(field) => <field.TextField label="Organization name" />}
								</form.AppField>
								<form.AppField name="slug">
									{(field) => (
										<field.TextField
											label="Organization slug"
											description="Used in URLs and status pages"
										/>
									)}
								</form.AppField>
								<div className="flex flex-col gap-3 sm:flex-row sm:items-center">
									<Button
										type="submit"
										className="w-full sm:w-auto"
										disabled={mutation.isPending}
									>
										{mutation.isPending ? "Creating..." : "Create organization"}
									</Button>
									<p className="text-sm text-[var(--text-muted)]">
										We’ll automatically redirect you to the dashboard
										afterwards.
									</p>
								</div>
							</form>
						</form.AppForm>
					</CardContent>
				</Card>
			</div>
		</main>
	);
}
