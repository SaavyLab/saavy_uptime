import { z } from "zod";
import { withAccessHeader } from "./api";

const apiBase = import.meta.env.VITE_API_URL;

export const httpMonitorConfigSchema = z.object({
	url: z.string(),
	interval: z.number(),
	timeout: z.number(),
	verifyTls: z.boolean(),
	followRedirects: z.boolean(),
});

export type HttpMonitorConfig = z.infer<typeof httpMonitorConfigSchema>;

export const monitorStatusSchema = z
	.enum(["up", "down", "degraded", "pending"])
	.transform((status) => status.toUpperCase());

export const monitorKindSchema = z.enum(["http", "tcp", "udp"]);

const monitorSchema = z.object({
	id: z.string(),
	orgId: z.string(),
	name: z.string(),
	kind: monitorKindSchema,
	enabled: z.number(),
	config: z
		.string()
		.transform((config) => JSON.parse(config) as HttpMonitorConfig),
	status: z.string().transform((status) => status.toLowerCase()),
	lastCheckedAt: z.number().nullable(),
	lastFailedAt: z.number().nullable(),
	firstCheckedAt: z.number().nullable(),
	rtMs: z.number().nullable(),
	region: z.string().nullable(),
	lastError: z.string().nullable(),
	nextRunAt: z.number().nullable(),
	createdAt: z.number(),
	updatedAt: z.number(),
});

export type Monitor = z.infer<typeof monitorSchema>;

const createMonitorSchema = z.object({
	name: z.string(),
	config: httpMonitorConfigSchema,
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

	try {
		return monitorSchema.array().parse(await response.json());
	} catch (error) {
		console.error(error);
		throw new Error(`Unable to parse monitors (${error})`);
	}
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
