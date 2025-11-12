import type { QueryClient } from "@tanstack/react-query";
import { withAccessHeader } from "./api";

export type BootstrapStatus = {
	isBootstrapped: boolean;
	suggestedSlug: string;
	email: string;
};

const apiBase = import.meta.env.VITE_API_URL;

export const fetchBootstrapStatus = async (): Promise<BootstrapStatus> => {
	const response = await fetch(`${apiBase}/api/bootstrap/status`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load bootstrap status (${response.status})`);
	}

	return response.json();
};

export const bootstrapStatusQueryOptions = {
	queryKey: ["bootstrapStatus"],
	queryFn: fetchBootstrapStatus,
	staleTime: 30_000,
};

export const invalidateBootstrapStatus = async (queryClient: QueryClient) => {
	await queryClient.invalidateQueries({ queryKey: ["bootstrapStatus"] });
};

export const provisionOrganization = async (name: string, slug: string) => {
	const response = await fetch(`${apiBase}/api/bootstrap/initialize`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify({ name, slug }),
	});

	if (!response.ok) {
		throw new Error(`Unable to provision organization (${response.status})`);
	}

	return response.json() as Promise<BootstrapStatus>;
};
