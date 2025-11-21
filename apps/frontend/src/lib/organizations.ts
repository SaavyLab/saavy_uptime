import { z } from "zod";
import { apiBase, withAccessHeader } from "./api";

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
	ownerId: z.string(),
	updatedAt: z.number().optional().nullable(),
});

export type Organization = z.infer<typeof organizationSchema>;

const memberSchema = z.object({
	email: z.string(),
	role: z.string(),
});

export type Member = z.infer<typeof memberSchema>;

export const getOrganization = async (): Promise<Organization> => {
	const response = await fetch(`${apiBase}/api/organizations`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load organization (${response.status})`);
	}

	return organizationSchema.parse(await response.json());
};

export const getOrganizationMembers = async (): Promise<Member[]> => {
	const response = await fetch(`${apiBase}/api/organizations/members`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load members (${response.status})`);
	}

	return memberSchema.array().parse(await response.json());
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
