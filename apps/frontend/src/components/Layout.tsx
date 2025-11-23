import { Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Menu } from "lucide-react";
import { useState } from "react";
import { Sidebar } from "./Sidebar";
import { Button } from "./ui/button";

// A simple Mobile Sheet could be added here, but for now we'll just have a placeholder
// or reuse the logic if needed. For this task, we focus on the desktop "platform" look.

export default function Layout() {
	const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

	return (
		<div className="min-h-screen bg-background text-foreground font-sans antialiased flex">
			<Sidebar />

			<div className="flex-1 flex flex-col min-w-0 md:pl-64 transition-all duration-300 ease-in-out">
				{/* Mobile Header */}
				<header className="sticky top-0 z-40 flex h-14 items-center gap-4 border-b border-border bg-background/80 px-4 backdrop-blur-sm md:hidden">
					<Button
						variant="ghost"
						size="icon"
						onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
					>
						<Menu className="h-5 w-5" />
						<span className="sr-only">Toggle Menu</span>
					</Button>
					<span className="font-bold">Saavy Uptime</span>
				</header>

				{/* Main Content */}
				<main className="flex-1 overflow-y-auto p-4 md:p-8 lg:p-10 relative">
					<div className="mx-auto max-w-7xl animate-in fade-in slide-in-from-bottom-4 duration-500 relative z-10">
						<Outlet />
					</div>
				</main>
			</div>

			<TanStackRouterDevtools position="bottom-right" />
		</div>
	);
}
