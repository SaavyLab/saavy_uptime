import { z } from "zod";
import { withAccessHeader } from "./api";

const apiBase = import.meta.env.VITE_API_URL;

const monitorSchema = z.object({
	id: z.string(),
	orgId: z.string(),
	name: z.string(),
	kind: z.string(),
	url: z.string(),
	intervalS: z.number(),
	timeoutMs: z.number(),
	followRedirects: z.number(),
	verifyTls: z.number(),
	expectStatusLow: z.number().nullable().optional(),
	expectStatusHigh: z.number().nullable().optional(),
	expectSubstring: z.string().nullable().optional(),
	headersJson: z.string().nullable().optional(),
	tagsJson: z.string().nullable().optional(),
	currentStatus: z.string(),
	lastCheckedAtTs: z.number().nullable(),
	enabled: z.number(),
	createdAt: z.number(),
	updatedAt: z.number(),
});

export type Monitor = z.infer<typeof monitorSchema>;

const createMonitorSchema = z.object({
	name: z.string(),
	url: z.string(),
	interval: z.number(),
	timeout: z.number(),
	followRedirects: z.boolean(),
	verifyTls: z.boolean(),
});

export type CreateMonitorInput = z.infer<typeof createMonitorSchema>;
export type UpdateMonitorInput = z.infer<typeof createMonitorSchema>;

const heartbeatSchema = z.object({
	monitorId: z.string(),
	dispatchId: z.string().nullable().optional(),
	ts: z.number(),
	ok: z.number(),
	code: z.number().nullable(),
	rttMs: z.number().nullable(),
	err: z.string().nullable(),
	region: z.string().nullable(),
});

export type Heartbeat = z.infer<typeof heartbeatSchema>;

const seedResponseSchema = z.object({
	created: z.number(),
	failed: z.number(),
});

export type SeedResponse = z.infer<typeof seedResponseSchema>;

export const getMonitors = async (): Promise<Monitor[]> => {
	const response = await fetch(`${apiBase}/api/monitors`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load monitors (${response.status})`);
	}

	return monitorSchema.array().parse(await response.json());
};

export const getMonitor = async (monitorId: string): Promise<Monitor> => {
	const response = await fetch(`${apiBase}/api/monitors/${monitorId}`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load monitor (${response.status})`);
	}

	return monitorSchema.parse(await response.json());
};

export const createMonitor = async (
	monitor: CreateMonitorInput,
): Promise<Monitor> => {
	const response = await fetch(`${apiBase}/api/monitors`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify(monitor),
	});
	if (!response.ok) {
		throw new Error(`Unable to create monitor (${response.status})`);
	}
	return monitorSchema.parse(await response.json());
};

export const updateMonitor = async (
	monitorId: string,
	monitor: UpdateMonitorInput,
): Promise<Monitor> => {
	const response = await fetch(`${apiBase}/api/monitors/${monitorId}`, {
		method: "PATCH",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify(monitor),
	});

	if (!response.ok) {
		throw new Error(`Unable to update monitor (${response.status})`);
	}

	return monitorSchema.parse(await response.json());
};

export const deleteMonitor = async (monitorId: string): Promise<void> => {
	const response = await fetch(`${apiBase}/api/monitors/${monitorId}`, {
		method: "DELETE",
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to delete monitor (${response.status})`);
	}
};

export const seedMonitors = async (): Promise<SeedResponse> => {
	const response = await fetch(`${apiBase}/api/internal/seed`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify({}),
	});

	if (!response.ok) {
		throw new Error(`Unable to seed monitors (${response.status})`);
	}

	return seedResponseSchema.parse(await response.json());
};

export const getMonitorHeartbeats = async (
	monitorId: string,
	limit = 50,
): Promise<Heartbeat[]> => {
	const response = await fetch(
		`${apiBase}/api/monitors/${monitorId}/heartbeats?limit=${limit}`,
		{
			headers: withAccessHeader(),
		},
	);

	if (!response.ok) {
		throw new Error(`Unable to load monitor heartbeats (${response.status})`);
	}

	return heartbeatSchema.array().parse(await response.json());
};
