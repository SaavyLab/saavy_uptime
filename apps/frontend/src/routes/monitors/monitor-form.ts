import { z } from "zod";
import { httpMonitorConfigSchema } from "@/lib/monitors";

export const monitorFormSchema = z.object({
	name: z.string(),
	config: httpMonitorConfigSchema,
	relayId: z.string(),
});

export type MonitorFormValues = z.infer<typeof monitorFormSchema>;

export const defaultMonitorFormValues: MonitorFormValues = {
	name: "",
	config: {
		url: "",
		interval: 60,
		timeout: 5000,
		followRedirects: true,
		verifyTls: true,
	},
	relayId: "",
};
