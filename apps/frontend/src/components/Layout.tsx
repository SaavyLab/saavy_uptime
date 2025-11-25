import { Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Menu, Activity } from "lucide-react";
import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { Button } from "./ui/button";

export default function Layout() {
	const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

	return (
		<div className="min-h-screen bg-background text-foreground font-sans antialiased flex">
			<Sidebar />

			<div className="flex-1 flex flex-col min-w-0 md:pl-56 transition-all duration-200">
				{/* Mobile Header */}
				<header className="sticky top-0 z-40 flex h-12 items-center gap-3 border-b border-white/[0.04] bg-[#0a0a0c]/95 px-4 backdrop-blur-xl md:hidden">
					<Button
						variant="ghost"
						size="icon"
						className="h-8 w-8"
						onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
					>
						<Menu className="h-4 w-4" />
						<span className="sr-only">Toggle Menu</span>
					</Button>
					<div className="flex items-center gap-2">
						<div className="h-5 w-5 rounded bg-gradient-to-br from-cyan-400 to-cyan-600 flex items-center justify-center">
							<Activity size={10} strokeWidth={2.5} className="text-black" />
						</div>
						<span className="text-sm font-semibold">Saavy</span>
					</div>
				</header>

				{/* Main Content */}
				<main className="flex-1 overflow-y-auto p-5 md:p-8 relative">
					<div className="mx-auto max-w-6xl animate-in fade-in duration-300 relative z-10">
						<Outlet />
					</div>
				</main>
			</div>

			<TanStackRouterDevtools position="bottom-right" />
		</div>
	);
}
