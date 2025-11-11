function App() {
	return (
		<div className="min-h-screen bg-white text-black">
			<div className="max-w-7xl mx-auto p-8">
				<div className="border-4 border-black p-8 mb-8 bg-[#ff6633]">
					<h1 className="text-6xl font-black mb-4">SAAVY UPTIME MONITOR</h1>
					<p className="text-xl font-bold uppercase tracking-tight">
						CLOUDFLARE-FIRST • BRUTALIST DESIGN • ZERO-SERVER DEPLOYMENT
					</p>
				</div>

				<div className="grid grid-cols-1 md:grid-cols-3 gap-6">
					<div className="border-4 border-black p-6 bg-white hover:bg-black hover:text-white transition-colors">
						<h2 className="mb-3">DURABLE OBJECTS</h2>
						<p className="font-mono text-sm normal-case">
							Stateful sub-minute scheduling with persistent alarm() API for
							reliable check execution
						</p>
					</div>

					<div className="border-4 border-black p-6 bg-white hover:bg-black hover:text-white transition-colors">
						<h2 className="mb-3">ANALYTICS ENGINE</h2>
						<p className="font-mono text-sm normal-case">
							High-performance time-series metrics for instant p50/p90/p99
							latency queries
						</p>
					</div>

					<div className="border-4 border-black p-6 bg-white hover:bg-black hover:text-white transition-colors">
						<h2 className="mb-3">D1 + R2 LIFECYCLE</h2>
						<p className="font-mono text-sm normal-case">
							Hot data in D1 (30d retention), cold archives in R2 for
							cost-effective long-term storage
						</p>
					</div>
				</div>

				<div className="mt-8 border-4 border-black p-6 bg-black text-white">
					<h3 className="mb-4 text-[#ff6633]">{"// STATUS: SYSTEM READY"}</h3>
					<pre className="font-mono text-xs bg-transparent p-0">
						{`> wrangler deploy
✓ Worker deployed to Cloudflare Edge
✓ Durable Objects configured
✓ D1 database initialized
✓ Analytics Engine streaming
✓ R2 bucket mounted
✓ Access policies active

[OK] All systems operational`}
					</pre>
				</div>
			</div>
		</div>
	);
}

export default App;
