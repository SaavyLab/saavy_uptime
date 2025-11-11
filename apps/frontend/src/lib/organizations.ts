export type CreateOrganizationInput = {
	slug: string;
	name: string;
};

export type Organization = {
	id: string;
	slug: string;
	name: string;
	created_at: number;
};

const apiBase = import.meta.env.VITE_API_URL;

export const getOrganization = async (organizationId: string): Promise<Organization> => {
	const response = await fetch(`${apiBase}/api/organizations/${organizationId}`);

	if (!response.ok) {
		throw new Error(`Unable to load organization (${response.status})`);
	}

	return response.json();
};

export const createOrganization = async (
	payload: CreateOrganizationInput,
): Promise<Organization> => {
	const response = await fetch(`${apiBase}/api/organizations`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(payload),
	});

	if (!response.ok) {
		throw new Error(`Unable to create organization (${response.status})`);
	}

	return response.json();
};
