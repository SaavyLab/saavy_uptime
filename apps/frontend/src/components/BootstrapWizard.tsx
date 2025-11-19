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
		<main className="min-h-screen bg-background px-4 py-10 text-foreground flex items-center justify-center lg:px-8 relative">
			{/* Background Flavor */}
			<div className="fixed inset-0 z-0 pointer-events-none">
				<div className="absolute top-0 right-0 -mt-20 -mr-20 w-96 h-96 rounded-full bg-primary/5 blur-3xl" />
				<div className="absolute bottom-0 left-0 -mb-20 -ml-20 w-96 h-96 rounded-full bg-primary/5 blur-3xl" />
			</div>

			<div className="w-full max-w-lg space-y-8 relative z-10">
				<div className="text-center space-y-2">
					<p className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
						Onboarding
					</p>
					<h1 className="text-3xl font-bold tracking-tight">
						Welcome to Saavy Uptime
					</h1>
					<p className="text-muted-foreground">
						You’re signed in as{" "}
						<span className="font-medium text-foreground">
							{email}
						</span>
						. Create your first organization to unlock monitors, incidents, and
						dashboards.
					</p>
				</div>

				<Card className="border-border bg-card">
					<CardHeader>
						<CardTitle className="text-xl">Create your organization</CardTitle>
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
									{(field) => (
										<field.TextField
											label="Organization Name"
											placeholder="My Organization"
										/>
									)}
								</form.AppField>
								<form.AppField name="slug">
									{(field) => (
										<field.TextField
											label="Slug"
											placeholder="my-org"
											description="Used in URLs and status pages"
										/>
									)}
								</form.AppField>
								<div className="flex flex-col gap-4 pt-2">
									<Button
										type="submit"
										className="w-full"
										disabled={mutation.isPending}
									>
										{mutation.isPending ? "Creating..." : "Create organization"}
									</Button>
									<p className="text-xs text-center text-muted-foreground">
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
