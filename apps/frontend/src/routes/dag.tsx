import { useCallback, useEffect, useState, useRef } from "react";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import {
	ReactFlow,
	Background,
	Controls,
	MiniMap,
	type Node,
	type Edge,
	useNodesState,
	useEdgesState,
	MarkerType,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import {
	Activity,
	Database,
	Play,
	Clock,
	CheckCircle2,
	XCircle,
} from "lucide-react";
import { SectionCard } from "@/components/layout/SectionCard";
import { Button } from "@/components/ui/button";
import type { RouterContext } from "@/router-context";

// Custom node component with consistent styling
function CustomNode({
	data,
}: {
	data: {
		label: string;
		icon: React.ReactNode;
		type: string;
		active?: boolean;
		success?: boolean;
		error?: boolean;
		ticking?: boolean;
	};
}) {
	return (
		<div
			className={`
				rounded-2xl border-2 px-6 py-4 shadow-lg transition-all duration-300
				${
					data.active
						? "border-[var(--accent)] bg-[var(--accent)]/20 scale-105"
						: data.success
							? "border-green-500 bg-green-500/20"
							: data.error
								? "border-red-500 bg-red-500/20"
								: "border-white/20 bg-[var(--surface-card)]"
				}
				${data.ticking ? "animate-pulse" : ""}
			`}
		>
			<div className="flex items-center gap-3">
				<div
					className={`
					rounded-full p-2
					${data.active ? "bg-[var(--accent)]/30" : data.success ? "bg-green-500/30" : data.error ? "bg-red-500/30" : "bg-white/10"}
				`}
				>
					{data.icon}
				</div>
				<div>
					<div className="font-semibold text-sm text-[var(--text-primary)]">
						{data.label}
					</div>
					<div className="text-xs text-[var(--text-soft)] uppercase tracking-wider">
						{data.type}
					</div>
				</div>
			</div>
		</div>
	);
}

const nodeTypes = {
	custom: CustomNode,
};

// Static node configuration (persistent infrastructure)
const staticNodes: Node[] = [
	{
		id: "ticker-do",
		type: "custom",
		position: { x: 250, y: 50 },
		data: {
			label: "Ticker DO",
			icon: <Clock size={20} className="text-[var(--accent)]" />,
			type: "Scheduler",
		},
	},
	{
		id: "d1",
		type: "custom",
		position: { x: 50, y: 450 },
		data: {
			label: "D1 Database",
			icon: <Database size={20} className="text-[var(--accent)]" />,
			type: "Hot Storage",
		},
	},
	{
		id: "ae",
		type: "custom",
		position: { x: 250, y: 450 },
		data: {
			label: "Analytics Engine",
			icon: <Activity size={20} className="text-[var(--accent)]" />,
			type: "Metrics",
		},
	},
	{
		id: "r2",
		type: "custom",
		position: { x: 475, y: 450 },
		data: {
			label: "R2 Bucket",
			icon: <Database size={20} className="text-[var(--accent)]" />,
			type: "Cold Storage",
		},
	},
];

// POPs for runner distribution
const POPS = ["SJC", "AMS", "SIN", "SYD", "GRU", "NRT"];

type CheckJob = {
	id: string;
	runnerId: string;
	pop: string;
	monitorId: string;
	startTime: number;
	phase: "dispatch" | "executing" | "writing" | "complete";
	success: boolean;
};

function DAGVisualizerPage() {
	const [nodes, setNodes, onNodesChange] = useNodesState(staticNodes);
	const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
	const [isSimulating, setIsSimulating] = useState(false);
	const [activeJobs, setActiveJobs] = useState<CheckJob[]>([]);
	const [tickCount, setTickCount] = useState(0);
	const [stats, setStats] = useState({ total: 0, success: 0, failed: 0 });
	const jobIdCounter = useRef(0);

	// Simulate ticker alarm (every 2 seconds during simulation)
	useEffect(() => {
		if (!isSimulating) return;

		const tickInterval = setInterval(() => {
			setTickCount((prev) => prev + 1);

			// Pulse the ticker node
			setNodes((nds) =>
				nds.map((node) =>
					node.id === "ticker-do"
						? { ...node, data: { ...node.data, ticking: true } }
						: node,
				),
			);

			setTimeout(() => {
				setNodes((nds) =>
					nds.map((node) =>
						node.id === "ticker-do"
							? { ...node, data: { ...node.data, ticking: false } }
							: node,
					),
				);
			}, 5000);

			// Dispatch 2-4 checks on each tick
			const numChecks = Math.floor(Math.random() * 3) + 2;
			const newJobs: CheckJob[] = [];

			for (let i = 0; i < numChecks; i++) {
				const pop = POPS[Math.floor(Math.random() * POPS.length)];
				const runnerId = `runner-${jobIdCounter.current}`;
				const success = Math.random() > 0.15; // 85% success rate

				newJobs.push({
					id: `job-${jobIdCounter.current}`,
					runnerId,
					pop,
					monitorId: `monitor-${Math.floor(Math.random() * 10)}`,
					startTime: Date.now(),
					phase: "dispatch",
					success,
				});

				jobIdCounter.current++;
			}

			setActiveJobs((jobs) => [...jobs, ...newJobs]);
		}, 15000); // Tick every 15 seconds

		return () => clearInterval(tickInterval);
	}, [isSimulating, setNodes]);

	// Process active jobs through their lifecycle
	useEffect(() => {
		if (activeJobs.length === 0) return;

		const interval = setInterval(() => {
			setActiveJobs((jobs) => {
				const now = Date.now();
				const updatedJobs = jobs.map((job) => {
					const elapsed = now - job.startTime;

					// Phase transitions:
					// 0-500ms: dispatch (spawn runner)
					// 500-1500ms: executing (runner active)
					// 1500-2000ms: writing (write to D1/AE)
					// 2000ms+: complete (cleanup)

					if (elapsed < 1000) {
						return { ...job, phase: "dispatch" as const };
					}
					if (elapsed < 2000) {
						return { ...job, phase: "executing" as const };
					}
					if (elapsed < 3000) {
						return { ...job, phase: "writing" as const };
					}
					return { ...job, phase: "complete" as const };
				});

				// Update stats for completed jobs
				const completed = updatedJobs.filter((j) => j.phase === "complete");
				if (completed.length > 0) {
					setStats((prev) => ({
						total: prev.total + completed.length,
						success: prev.success + completed.filter((j) => j.success).length,
						failed: prev.failed + completed.filter((j) => !j.success).length,
					}));
				}

				// Remove completed jobs
				return updatedJobs.filter((j) => j.phase !== "complete");
			});
		}, 100);

		return () => clearInterval(interval);
	}, [activeJobs]);

	// Sync jobs to nodes and edges
	useEffect(() => {
		// Create runner nodes for active jobs with better spacing
		const runnerNodes: Node[] = activeJobs.map((job, idx) => {
			// Spread runners in a grid pattern with better spacing
			const cols = 5;
			const colWidth = 350;
			const rowHeight = 80;
			const startX = 20;
			const startY = 200;

			const col = idx % cols;
			const row = Math.floor(idx / cols);

			const x = startX + col * colWidth;
			const y = startY + row * rowHeight;

			return {
				id: job.runnerId,
				type: "custom",
				position: { x, y },
				data: {
					label: `Runner (${job.pop})`,
					icon:
						job.phase === "writing" ? (
							job.success ? (
								<CheckCircle2 size={20} className="text-green-500" />
							) : (
								<XCircle size={20} className="text-red-500" />
							)
						) : (
							<Play size={20} className="text-[var(--accent)]" />
						),
					type: "Check Executor",
					active: job.phase === "executing",
					success: job.phase === "writing" && job.success,
					error: job.phase === "writing" && !job.success,
				},
			};
		});

		setNodes([...staticNodes, ...runnerNodes]);

		// Create edges for active jobs
		const jobEdges: Edge[] = activeJobs.flatMap((job) => {
			const edges: Edge[] = [];

			// Ticker → Runner (always show during dispatch and executing)
			if (
				job.phase === "dispatch" ||
				job.phase === "executing" ||
				job.phase === "writing"
			) {
				edges.push({
					id: `ticker-${job.runnerId}`,
					source: "ticker-do",
					target: job.runnerId,
					animated: job.phase === "dispatch",
					markerEnd: { type: MarkerType.ArrowClosed },
					style: {
						stroke:
							job.phase === "writing"
								? job.success
									? "#22c55e"
									: "#ef4444"
								: "var(--accent)",
						strokeWidth: 2,
						opacity: job.phase === "writing" ? 0.4 : 1,
					},
				});
			}

			// Runner → D1 (writing phase)
			if (job.phase === "writing") {
				edges.push({
					id: `${job.runnerId}-d1`,
					source: job.runnerId,
					target: "d1",
					animated: true,
					markerEnd: { type: MarkerType.ArrowClosed },
					style: {
						stroke: job.success ? "#22c55e" : "#ef4444",
						strokeWidth: 2,
					},
				});
			}

			// Runner → AE (writing phase)
			if (job.phase === "writing") {
				edges.push({
					id: `${job.runnerId}-ae`,
					source: job.runnerId,
					target: "ae",
					animated: true,
					markerEnd: { type: MarkerType.ArrowClosed },
					style: {
						stroke: job.success ? "#22c55e" : "#ef4444",
						strokeWidth: 2,
					},
				});
			}

			return edges;
		});

		// Add archival edge (static)
		jobEdges.push({
			id: "d1-r2",
			source: "d1",
			target: "r2",
			animated: false,
			markerEnd: { type: MarkerType.ArrowClosed },
			style: {
				stroke: "var(--text-soft)",
				strokeWidth: 2,
				strokeDasharray: "5,5",
			},
			label: "Archive",
		});

		setEdges(jobEdges);
	}, [activeJobs, setNodes, setEdges]);

	const startSimulation = useCallback(() => {
		setIsSimulating(true);
		setTickCount(0);
		setStats({ total: 0, success: 0, failed: 0 });
		jobIdCounter.current = 0;
	}, []);

	const stopSimulation = useCallback(() => {
		setIsSimulating(false);
		setActiveJobs([]);
		setNodes(staticNodes);
		setEdges([]);
	}, [setNodes, setEdges]);

	return (
		<div className="space-y-8">
      <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div className="space-y-1">
          <h1 className="text-2xl font-bold tracking-tight">DAG Execution</h1>
          <p className="text-muted-foreground">
            Visualize real-time health check dispatch and execution flow.
          </p>
        </div>
        <Button
          type="button"
          onClick={isSimulating ? stopSimulation : startSimulation}
          className="gap-2"
        >
          {isSimulating ? (
            <>
              <XCircle size={16} />
              Stop Simulation
            </>
          ) : (
            <>
              <Play size={16} />
              Start Simulation
            </>
          )}
        </Button>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="rounded-xl border border-border bg-muted/20 p-4">
          <div className="mb-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Ticker Stats
          </div>
          <div className="space-y-2 font-mono text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Alarm Cycles</span>
              <span className="text-foreground font-medium">{tickCount}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Active Jobs</span>
              <span className="text-primary font-medium">{activeJobs.length}</span>
            </div>
          </div>
        </div>

        <div className="rounded-xl border border-border bg-muted/20 p-4">
          <div className="mb-3 text-xs font-medium uppercase tracking-wider text-muted-foreground">
            Check Results
          </div>
          <div className="space-y-2 font-mono text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total</span>
              <span className="text-foreground font-medium">{stats.total}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Success</span>
              <span className="text-emerald-500 font-medium">{stats.success}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Failed</span>
              <span className="text-red-500 font-medium">{stats.failed}</span>
            </div>
          </div>
        </div>
      </div>

			<SectionCard
				title="Architecture Flow"
				description="Ticker DO dispatches checks → Runner Workers execute → Results stream to D1/AE, archive to R2"
			>
				<div
					className="relative h-[650px] w-full rounded-xl border border-border bg-card"
					style={{
						background:
							"radial-gradient(circle at 50% 50%, rgba(255,255,255,0.03) 0%, transparent 100%)",
					}}
				>
					<ReactFlow
						nodes={nodes}
						edges={edges}
						onNodesChange={onNodesChange}
						onEdgesChange={onEdgesChange}
						nodeTypes={nodeTypes}
						fitView
						attributionPosition="bottom-left"
						proOptions={{ hideAttribution: true }}
					>
						<Background
							color="#71717a"
							gap={16}
							size={1}
							style={{ opacity: 0.1 }}
						/>
						<Controls
							style={{
								background: "var(--card)",
								border: "1px solid var(--border)",
								borderRadius: "12px",
							}}
						/>
						<MiniMap
							nodeColor={(node) => {
								if (node.data.error) return "#ef4444";
								if (node.data.success) return "#22c55e";
								if (node.data.active) return "var(--primary)";
								return "#71717a";
							}}
							maskColor="rgba(0, 0, 0, 0.8)"
							style={{
								background: "var(--card)",
								border: "1px solid var(--border)",
								borderRadius: "12px",
							}}
						/>
					</ReactFlow>
				</div>

				<div className="mt-6 space-y-4">
					<div className="rounded-xl border border-border bg-muted/20 p-4">
						<h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
							Simulation Details
						</h3>
						<div className="space-y-3 text-sm text-muted-foreground">
							<div>
								<strong className="text-foreground">
									Ticker DO alarm:
								</strong>{" "}
								Fires every 2 seconds, queries D1 for monitors due, and
								dispatches 2-4 checks per cycle.
							</div>
							<div>
								<strong className="text-foreground">
									Runner Workers (ephemeral):
								</strong>{" "}
								Spawn dynamically from 6 global POPs (SJC, AMS, SIN, SYD, GRU,
								NRT), execute the check, then disappear after writing results.
							</div>
							<div>
								<strong className="text-foreground">
									Check lifecycle:
								</strong>{" "}
								Dispatch (0.5s) → Execute (1s) → Write to D1/AE (0.5s) →
								Complete. Green edges indicate success, red edges indicate
								failures.
							</div>
							<div>
								<strong className="text-foreground">
									Data archival (D1→R2):
								</strong>{" "}
								Dashed line represents the janitor cron that moves old
								heartbeats from D1 to R2 for long-term storage.
							</div>
						</div>
					</div>

					<div className="rounded-xl border border-border bg-muted/20 p-4">
						<h3 className="mb-3 text-sm font-semibold uppercase tracking-wider text-muted-foreground">
							Future Enhancements
						</h3>
						<ul className="space-y-2 text-sm text-muted-foreground">
							<li>
								•{" "}
								<strong className="text-foreground">
									Live SSE/WebSocket stream
								</strong>{" "}
								from backend with real dispatch metadata
							</li>
							<li>
								•{" "}
								<strong className="text-foreground">
									Per-POP latency heatmap
								</strong>{" "}
								showing geographic distribution of checks
							</li>
							<li>
								•{" "}
								<strong className="text-foreground">
									Incident correlation
								</strong>{" "}
								highlighting which checks triggered incidents
							</li>
							<li>
								•{" "}
								<strong className="text-foreground">
									Response payload inspection
								</strong>{" "}
								showing HTTP status, headers, and body snippets
							</li>
						</ul>
					</div>
				</div>
			</SectionCard>
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/dag",
		component: DAGVisualizerPage,
		getParentRoute: () => parentRoute,
	});
