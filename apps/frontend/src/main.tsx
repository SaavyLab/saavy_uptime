import type { QueryClient } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import {
	createRootRoute,
	createRouter,
	Outlet,
	RouterProvider,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import Header from "@/components/Header";
import BootstrapGate from "@/components/BootstrapGate";
import * as TanStackQueryProvider from "@/integrations/tanstack-query/root-provider.tsx";
import DashboardRoute from "@/routes/dashboard.tsx";
import IncidentsRoute from "@/routes/incidents.tsx";
import MonitorNewRoute from "@/routes/monitors/monitor-new";
import MonitorsRoute from "@/routes/monitors/monitors";
import OrganizationRoute from "@/routes/organization/organization";
import StatusRoute from "@/routes/status.tsx";
import "@/styles.css";
import reportWebVitals from "@/reportWebVitals.ts";

interface RouterContext {
	queryClient: QueryClient;
}

const rootRoute = createRootRoute<RouterContext>({
	component: () => (
		<BootstrapGate>
			<Header />
			<Outlet />
			<TanStackRouterDevtools />
		</BootstrapGate>
	),
});

const typedRootRoute = rootRoute as unknown as RootRoute<
	Register,
	undefined,
	RouterContext
>;

const routeTree = typedRootRoute.addChildren([
	DashboardRoute(typedRootRoute),
	MonitorsRoute(typedRootRoute),
	MonitorNewRoute(typedRootRoute),
	IncidentsRoute(typedRootRoute),
	StatusRoute(typedRootRoute),
	OrganizationRoute(typedRootRoute),
]);

const TanStackQueryProviderContext = TanStackQueryProvider.getContext();
const router = createRouter({
	routeTree,
	context: {
		...TanStackQueryProviderContext,
	},
	defaultPreload: "intent",
	scrollRestoration: true,
	defaultStructuralSharing: true,
	defaultPreloadStaleTime: 0,
});

declare module "@tanstack/react-router" {
	interface Register {
		router: typeof router;
	}
}

const rootElement = document.getElementById("app");
if (rootElement && !rootElement.innerHTML) {
	const root = ReactDOM.createRoot(rootElement);
	root.render(
		<StrictMode>
			<TanStackQueryProvider.Provider {...TanStackQueryProviderContext}>
				<RouterProvider router={router} />
			</TanStackQueryProvider.Provider>
		</StrictMode>,
	);
}

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
