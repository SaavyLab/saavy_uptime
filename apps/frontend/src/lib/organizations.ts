import { z } from "zod";
import { withAccessHeader } from "./api";

const apiBase = import.meta.env.VITE_API_URL;

const createOrganizationSchema = z.object({
	slug: z.string(),
	name: z.string(),
});

export type CreateOrganizationInput = z.infer<typeof createOrganizationSchema>;

const organizationSchema = z.object({
	id: z.string(),
	slug: z.string(),
	name: z.string(),
	createdAt: z.number(),
	updatedAt: z.number(),
});

export type Organization = z.infer<typeof organizationSchema>;

export const getOrganization = async (
	organizationId: string,
): Promise<Organization> => {
	const response = await fetch(
		`${apiBase}/api/organizations/${organizationId}`,
		{
			headers: withAccessHeader(),
		},
	);

	if (!response.ok) {
		throw new Error(`Unable to load organization (${response.status})`);
	}

	return organizationSchema.parse(await response.json());
};

export const createOrganization = async (
	payload: CreateOrganizationInput,
): Promise<Organization> => {
	const response = await fetch(`${apiBase}/api/organizations`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify(payload),
	});

	if (!response.ok) {
		throw new Error(`Unable to create organization (${response.status})`);
	}

	return organizationSchema.parse(await response.json());
};
