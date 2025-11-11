export type CreateMonitorInput = {
	orgId: string;
	name: string;
	url: string;
	interval: number;
	timeout: number;
	followRedirects: boolean;
	verifyTls: boolean;
};

export type Monitor = {
	id: string;
	org_id: string;
	name: string;
	url: string;
	interval_s: number;
	timeout_ms: number;
	current_status: string;
	last_checked_at_ts: number | null;
	enabled: number;
	created_at: number;
};

const apiBase = import.meta.env.VITE_API_URL;

export const getMonitors = async (orgId: string): Promise<Monitor[]> => {
	const response = await fetch(`${apiBase}/api/monitors/org/${orgId}`);

	if (!response.ok) {
		throw new Error(`Unable to load monitors (${response.status})`);
	}

	return response.json();
};

export const createMonitor = async (monitor: CreateMonitorInput) => {
	const response = await fetch(`${apiBase}/api/monitors`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(monitor),
	});
	if (!response.ok) {
		throw new Error(`Unable to create monitor (${response.status})`);
	}
	return response.json();
};
