import { Link } from "@tanstack/react-router";
import { useState } from "react";
import {
	Activity,
	AlertCircle,
	Home,
	Menu,
	Monitor,
	X,
} from "lucide-react";

export default function Header() {
	const [isOpen, setIsOpen] = useState(false);

	return (
		<>
			<header className="p-4 flex items-center bg-black text-white border-b-4 border-[#ff6633]">
				<button
					type="button"
					onClick={() => setIsOpen(true)}
					className="p-3 bg-[#ff6633] text-black border-2 border-black hover:bg-[#ff5500] transition-colors font-bold"
					aria-label="Open menu"
				>
					<Menu size={24} strokeWidth={3} />
				</button>
				<h1 className="ml-6 text-2xl font-bold uppercase tracking-tight">
					<Link to="/" className="hover:text-[#ff6633] transition-colors">
						SAAVY UPTIME
					</Link>
				</h1>
			</header>

			<aside
				className={`fixed top-0 left-0 h-full w-80 bg-black text-white border-r-4 border-[#ff6633] z-50 transform transition-transform duration-200 ease-linear flex flex-col ${
					isOpen ? "translate-x-0" : "-translate-x-full"
				}`}
			>
				<div className="flex items-center justify-between p-4 border-b-2 border-white">
					<h2 className="text-xl font-bold uppercase tracking-tight">
						NAVIGATION
					</h2>
					<button
						type="button"
						onClick={() => setIsOpen(false)}
						className="p-2 bg-white text-black border-2 border-black hover:bg-[#ff6633] hover:text-black transition-colors"
						aria-label="Close menu"
					>
						<X size={24} strokeWidth={3} />
					</button>
				</div>

				<nav className="flex-1 p-4 overflow-y-auto">
					<Link
						to="/"
						onClick={() => setIsOpen(false)}
						className="flex items-center gap-3 p-3 border-2 border-white hover:bg-white hover:text-black transition-colors mb-2 font-bold uppercase tracking-tight"
						activeProps={{
							className:
								"flex items-center gap-3 p-3 border-2 border-[#ff6633] bg-[#ff6633] text-black transition-colors mb-2 font-bold uppercase tracking-tight",
						}}
					>
						<Home size={20} strokeWidth={2.5} />
						<span>Dashboard</span>
					</Link>

					<Link
						to="/monitors"
						onClick={() => setIsOpen(false)}
						className="flex items-center gap-3 p-3 border-2 border-white hover:bg-white hover:text-black transition-colors mb-2 font-bold uppercase tracking-tight"
						activeProps={{
							className:
								"flex items-center gap-3 p-3 border-2 border-[#ff6633] bg-[#ff6633] text-black transition-colors mb-2 font-bold uppercase tracking-tight",
						}}
					>
						<Monitor size={20} strokeWidth={2.5} />
						<span>Monitors</span>
					</Link>

					<Link
						to="/incidents"
						onClick={() => setIsOpen(false)}
						className="flex items-center gap-3 p-3 border-2 border-white hover:bg-white hover:text-black transition-colors mb-2 font-bold uppercase tracking-tight"
						activeProps={{
							className:
								"flex items-center gap-3 p-3 border-2 border-[#ff6633] bg-[#ff6633] text-black transition-colors mb-2 font-bold uppercase tracking-tight",
						}}
					>
						<AlertCircle size={20} strokeWidth={2.5} />
						<span>Incidents</span>
					</Link>

					<Link
						to="/status"
						onClick={() => setIsOpen(false)}
						className="flex items-center gap-3 p-3 border-2 border-white hover:bg-white hover:text-black transition-colors mb-2 font-bold uppercase tracking-tight"
						activeProps={{
							className:
								"flex items-center gap-3 p-3 border-2 border-[#ff6633] bg-[#ff6633] text-black transition-colors mb-2 font-bold uppercase tracking-tight",
						}}
					>
						<Activity size={20} strokeWidth={2.5} />
						<span>Status Page</span>
					</Link>
				</nav>
			</aside>
		</>
	);
}
