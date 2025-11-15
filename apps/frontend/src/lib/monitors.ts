import { z } from "zod";
import { withAccessHeader } from "./api";

const apiBase = import.meta.env.VITE_API_URL;

const createMonitorSchema = z.object({
	name: z.string(),
	url: z.string(),
	interval: z.number(),
	timeout: z.number(),
	followRedirects: z.boolean(),
	verifyTls: z.boolean(),
});

export type CreateMonitorInput = z.infer<typeof createMonitorSchema>;

const monitorSchema = z.object({
	id: z.string(),
	name: z.string(),
	url: z.string(),
	intervalS: z.number(),
	timeoutMs: z.number(),
	currentStatus: z.string(),
	lastCheckedAtTs: z.number().nullable(),
	enabled: z.number(),
	createdAt: z.number(),
	updatedAt: z.number(),
});

export type Monitor = z.infer<typeof monitorSchema>;

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
