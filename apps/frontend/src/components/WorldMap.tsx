import { useQuery } from "@tanstack/react-query";
import {
	ComposableMap,
	Geographies,
	Geography,
	Marker,
} from "react-simple-maps";
import {
	getColoCoordinates,
	type CloudflareColo,
} from "@/lib/cloudflare-colos";

const GEO_URL =
	"https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json";

export interface ColoMarker {
	colo: string;
	count: number;
	status: "up" | "down" | "mixed";
}

interface WorldMapProps {
	markers: ColoMarker[];
	className?: string;
}

export function WorldMap({ markers, className }: WorldMapProps) {
	const { data: coloMap, isLoading } = useQuery({
		queryKey: ["cloudflare-colos"],
		queryFn: getColoCoordinates,
		staleTime: 1000 * 60 * 60, // 1 hour
	});

	const resolvedMarkers = markers
		.map((marker) => {
			const colo = coloMap?.get(marker.colo);
			if (!colo) return null;
			return { ...marker, colo: colo };
		})
		.filter(
			(
				m,
			): m is {
				colo: CloudflareColo;
				count: number;
				status: "up" | "down" | "mixed";
			} => m !== null,
		);

	return (
		<div className={className}>
			<ComposableMap
				projection="geoMercator"
				projectionConfig={{
					scale: 120,
					center: [0, 30],
				}}
				style={{ width: "100%", height: "100%" }}
			>
				<Geographies geography={GEO_URL}>
					{({ geographies }) =>
						geographies.map((geo) => (
							<Geography
								key={geo.rsmKey}
								geography={geo}
								fill="#27272a"
								stroke="#3f3f46"
								strokeWidth={0.3}
								style={{
									default: { outline: "none" },
									hover: { outline: "none", fill: "#3f3f46" },
									pressed: { outline: "none" },
								}}
							/>
						))
					}
				</Geographies>

				{!isLoading &&
					resolvedMarkers.map((marker) => (
						<Marker
							key={marker.colo.iata}
							coordinates={[marker.colo.lon, marker.colo.lat]}
						>
							<circle
								r={Math.min(3 + marker.count * 0.5, 8)}
								fill={
									marker.status === "up"
										? "#34d399"
										: marker.status === "down"
											? "#f87171"
											: "#fbbf24"
								}
								fillOpacity={0.8}
								stroke="#fff"
								strokeWidth={0.5}
								strokeOpacity={0.3}
							/>
						</Marker>
					))}
			</ComposableMap>
		</div>
	);
}

export function aggregateHeartbeatsByLocation(
	heartbeats: Array<{ colo?: string | null; status: string }>,
): ColoMarker[] {
	const byLocation = new Map<string, { up: number; down: number }>();

	for (const hb of heartbeats) {
		const colo = hb.colo;
		if (!colo) continue;

		const existing = byLocation.get(colo) ?? { up: 0, down: 0 };
		if (hb.status === "up") {
			existing.up++;
		} else {
			existing.down++;
		}
		byLocation.set(colo, existing);
	}

	return Array.from(byLocation.entries()).map(([colo, counts]) => ({
		colo,
		count: counts.up + counts.down,
		status: counts.down === 0 ? "up" : counts.up === 0 ? "down" : "mixed",
	}));
}
