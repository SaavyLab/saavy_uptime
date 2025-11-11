import { StrictMode } from "react";
import ReactDOM from "react-dom/client";
import {
	Outlet,
	RouterProvider,
	createRootRoute,
	createRouter,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import type { QueryClient } from "@tanstack/react-query";

import Header from "@/components/Header";
import * as TanStackQueryProvider from "@/integrations/tanstack-query/root-provider.tsx";

import DashboardRoute from "@/routes/dashboard.tsx";
import MonitorsRoute from "@/routes/monitors/monitors";
import MonitorNewRoute from "@/routes/monitors/monitor-new";
import IncidentsRoute from "@/routes/incidents.tsx";
import StatusRoute from "@/routes/status.tsx";
import OrganizationRoute from "@/routes/organization/organization";

import "@/styles.css";
import reportWebVitals from "@/reportWebVitals.ts";

interface RouterContext {
	queryClient: QueryClient;
}

const rootRoute = createRootRoute<RouterContext>({
	component: () => (
		<>
			<Header />
			<Outlet />
			<TanStackRouterDevtools />
		</>
	),
});

const routeTree = rootRoute.addChildren([
	DashboardRoute(rootRoute as any), // what the fuck?
	MonitorsRoute(rootRoute as any),
	MonitorNewRoute(rootRoute as any),
	IncidentsRoute(rootRoute as any),
	StatusRoute(rootRoute as any),
	OrganizationRoute(rootRoute as any),
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
