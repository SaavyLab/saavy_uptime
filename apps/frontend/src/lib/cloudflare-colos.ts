import { z } from "zod";

const coloSchema = z.object({
	iata: z.string(),
	lat: z.number(),
	lon: z.number(),
	cca2: z.string(),
	region: z.string(),
	city: z.string(),
});

export type CloudflareColo = z.infer<typeof coloSchema>;

let coloCache: Map<string, CloudflareColo> | null = null;

export async function getColoCoordinates(): Promise<
	Map<string, CloudflareColo>
> {
	if (coloCache) return coloCache;

	const response = await fetch("https://speed.cloudflare.com/locations");
	if (!response.ok) {
		throw new Error(`Failed to fetch colo locations: ${response.status}`);
	}

	const data = await response.json();
	const colos = z.array(coloSchema).parse(data);

	coloCache = new Map(colos.map((colo) => [colo.iata, colo]));
	return coloCache;
}

export function getColoFromCache(iata: string): CloudflareColo | undefined {
	return coloCache?.get(iata);
}
