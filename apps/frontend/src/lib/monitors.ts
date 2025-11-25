import { z } from "zod";
import { apiBase, withAccessHeader } from "./api";

export type HttpMonitorConfig = z.infer<typeof httpMonitorConfigSchema>;

export const monitorStatusSchema = z
	.enum(["up", "down", "degraded", "pending"])
	.transform((status) => status.toUpperCase());

export const monitorKindSchema = z.enum(["http", "tcp", "udp"]);

export const monitorConfigResponseSchema = z.object({
	url: z.string(),
	interval: z.number(),
	timeout: z.number(),
	verify_tls: z.boolean(),
	follow_redirects: z.boolean(),
});

export const httpMonitorConfigSchema = monitorConfigResponseSchema.transform(
	(config) => ({
		url: config.url,
		interval: config.interval,
		timeout: config.timeout,
		verifyTls: config.verify_tls,
		followRedirects: config.follow_redirects,
	}),
);

export type MonitorConfigResponse = z.infer<typeof monitorConfigResponseSchema>;

const monitorSchema = z.object({
	id: z.string(),
	orgId: z.string(),
	name: z.string(),
	kind: monitorKindSchema,
	enabled: z.number(),
	config: monitorConfigResponseSchema.transform((config) =>
		httpMonitorConfigSchema.parse(config),
	),
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

const heartbeatSampleSchema = z.object({
	timestampMs: z.number(),
	status: z.string().transform((s) => s.toLowerCase()),
	latencyMs: z.number(),
	region: z.string().nullable(),
	colo: z.string().nullable(),
	error: z.string().nullable(),
	code: z.number().nullable(),
	sampleRate: z.number(),
	dispatchId: z.string().nullable(),
});

const monitorHeartbeatsResponseSchema = z.object({
	monitorId: z.string(),
	window: z.object({
		sinceMs: z.number(),
		untilMs: z.number(),
		hours: z.number(),
	}),
	items: z.array(heartbeatSampleSchema),
});

export type HeartbeatSample = z.infer<typeof heartbeatSampleSchema>;
export type MonitorHeartbeatsResponse = z.infer<
	typeof monitorHeartbeatsResponseSchema
>;

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

export const seedMonitors = async (quantity: number): Promise<SeedResponse> => {
	const response = await fetch(`${apiBase}/api/internal/seed`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify({ quantity }),
	});

	if (!response.ok) {
		throw new Error(`Unable to seed monitors (${response.status})`);
	}

	return seedResponseSchema.parse(await response.json());
};

export const getMonitorHeartbeats = async (
	monitorId: string,
	options?: { limit?: number; windowHours?: number },
): Promise<MonitorHeartbeatsResponse> => {
	const params = new URLSearchParams();
	const limit = options?.limit ?? 50;
	const windowHours = options?.windowHours ?? 24;
	params.set("limit", limit.toString());
	params.set("windowHours", windowHours.toString());

	const response = await fetch(
		`${apiBase}/api/monitors/${monitorId}/heartbeats?${params.toString()}`,
		{
			headers: withAccessHeader(),
		},
	);

	if (!response.ok) {
		throw new Error(`Unable to load monitor heartbeats (${response.status})`);
	}

	return monitorHeartbeatsResponseSchema.parse(await response.json());
};
