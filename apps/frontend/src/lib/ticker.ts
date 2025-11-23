import { z } from "zod";
import { apiBase, withAccessHeader } from "./api";

const reconcileResponseSchema = z.object({
	organizations: z.number(),
	bootstrapped: z.number(),
	failed: z.number(),
});

export type TickerReconcileSummary = z.infer<typeof reconcileResponseSchema>;

export const reconcileTickers = async (): Promise<TickerReconcileSummary> => {
	const response = await fetch(`${apiBase}/api/internal/ticker/reconcile`, {
		method: "POST",
		headers: withAccessHeader({
			"Content-Type": "application/json",
		}),
	});

	if (!response.ok) {
		throw new Error(`Unable to reconcile tickers (${response.status})`);
	}

	const payload = await response.json();
	return reconcileResponseSchema.parse(payload);
};
