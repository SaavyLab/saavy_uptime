import type { ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { bootstrapStatusQueryOptions } from "@/lib/bootstrap";
import BootstrapWizard from "./BootstrapWizard";

interface BootstrapGateProps {
	children: ReactNode;
}

function FullScreenMessage({
	title,
	body,
	action,
}: {
	title: string;
	body?: string;
	action?: ReactNode;
}) {
	return (
		<div className="flex min-h-screen flex-col items-center justify-center gap-4 bg-background text-foreground p-4">
			<div className="text-center">
				<h1 className="text-2xl font-semibold">{title}</h1>
				{body ? <p className="mt-2 text-muted-foreground">{body}</p> : null}
			</div>
			{action}
		</div>
	);
}

export default function BootstrapGate({ children }: BootstrapGateProps) {
	const statusQuery = useQuery(bootstrapStatusQueryOptions);

	if (statusQuery.isLoading) {
		return (
			<FullScreenMessage
				title="Preparing first-run setup"
				body="Hang tight while we check the environment."
			/>
		);
	}

	if (statusQuery.isError) {
		return (
			<FullScreenMessage
				title="Unable to contact backend"
				body={
					statusQuery.error instanceof Error
						? statusQuery.error.message
						: "Unknown error"
				}
				action={
					<button
						type="button"
						className="rounded-md bg-white/10 px-4 py-2 text-sm font-medium text-white transition hover:bg-white/20"
						onClick={() => statusQuery.refetch()}
					>
						Retry
					</button>
				}
			/>
		);
	}

	const status = statusQuery.data;
	if (!status) {
		return (
			<FullScreenMessage
				title="Unknown status"
				body="Bootstrap status payload missing."
			/>
		);
	}

	if (!status.isBootstrapped) {
		return (
			<BootstrapWizard
				suggestedSlug={status.suggestedSlug}
				email={status.email}
			/>
		);
	}

	return <>{children}</>;
}
