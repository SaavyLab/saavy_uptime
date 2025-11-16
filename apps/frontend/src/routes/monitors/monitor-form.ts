import { z } from "zod";

export const monitorFormSchema = z.object({
	name: z.string(),
	url: z.string(),
	interval: z.number(),
	timeout: z.number(),
	followRedirects: z.boolean(),
	verifyTls: z.boolean(),
});

export type MonitorFormValues = z.infer<typeof monitorFormSchema>;

export const defaultMonitorFormValues: MonitorFormValues = {
	name: "",
	url: "",
	interval: 60,
	timeout: 5000,
	followRedirects: true,
	verifyTls: true,
};
