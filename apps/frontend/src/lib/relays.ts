import { z } from "zod";
import { apiBase, withAccessHeader } from "./api";

const relaySchema = z.object({
	id: z.string(),
	slug: z.string(),
	name: z.string(),
	locationHint: z.string(),
	jurisdiction: z.string(),
	durableObjectId: z.string(),
	enabled: z.boolean(),
	lastBootstrappedAt: z.number().nullable(),
	lastError: z.string().nullable(),
	createdAt: z.number(),
	updatedAt: z.number(),
});

export type Relay = z.infer<typeof relaySchema>;

const relayListSchema = relaySchema.array();

const registerRelaySchema = z.object({
	slug: z.string(),
	name: z.string(),
	locationHint: z.string(),
});

export type RegisterRelayInput = z.infer<typeof registerRelaySchema>;

export const getRelays = async (): Promise<Relay[]> => {
	const response = await fetch(`${apiBase}/api/internal/relays`, {
		headers: withAccessHeader(),
	});

	if (!response.ok) {
		throw new Error(`Unable to load relays (${response.status})`);
	}

	return relayListSchema.parse(await response.json());
};

export const createRelay = async (
	input: RegisterRelayInput,
): Promise<Relay> => {
	const payload = registerRelaySchema.parse(input);
	const response = await fetch(`${apiBase}/api/internal/relays`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
		body: JSON.stringify(payload),
	});

	if (!response.ok) {
		throw new Error(`Unable to create relay (${response.status})`);
	}

	return relaySchema.parse(await response.json());
};
