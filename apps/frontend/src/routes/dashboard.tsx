import { useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute, Link } from "@tanstack/react-router";
import { Activity, CheckCircle, XCircle, AlertTriangle, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/Skeleton";
import { getMonitors, type Monitor } from "@/lib/monitors";
import type { RouterContext } from "@/router-context";
import { cn } from "@/lib/utils";

const formatTimestamp = (value?: number | null) => {
  if (!value) return "—";
  return new Date(value * 1000).toLocaleString();
};

function DashboardPage() {
  const { data: monitors = [], isLoading } = useQuery<Monitor[]>({
    queryKey: ["monitors"],
    queryFn: () => getMonitors(),
  });

  const upCount = monitors.filter((m) => m.currentStatus === "up").length;
  const downCount = monitors.filter((m) => m.currentStatus === "down").length;
  const maintenanceCount = monitors.filter((m) => m.currentStatus === "maintenance").length;

  const recentMonitors = [...monitors]
    .sort((a, b) => (b.lastCheckedAtTs ?? 0) - (a.lastCheckedAtTs ?? 0))
    .slice(0, 6);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Overview</h1>
          <p className="text-muted-foreground">Monitor your infrastructure health.</p>
        </div>
        <div className="flex items-center gap-2">
          <Link to="/monitors/new">
            <Button className="gap-2">
              <Plus className="h-4 w-4" />
              New Monitor
            </Button>
          </Link>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Monitors</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? "..." : monitors.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Operational</CardTitle>
            <CheckCircle className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? "..." : upCount}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Down</CardTitle>
            <XCircle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? "..." : downCount}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Maintenance</CardTitle>
            <AlertTriangle className="h-4 w-4 text-amber-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{isLoading ? "..." : maintenanceCount}</div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>
              Latest checks from your monitors.
            </CardDescription>
          </CardHeader>
          <CardContent>
             {isLoading ? (
               <div className="space-y-2">
                 <Skeleton className="h-8 w-full" />
                 <Skeleton className="h-8 w-full" />
                 <Skeleton className="h-8 w-full" />
               </div>
             ) : recentMonitors.length > 0 ? (
               <div className="space-y-4">
                 {recentMonitors.map((monitor) => (
                   <div key={monitor.id} className="flex items-center justify-between border-b border-border last:border-0 pb-4 last:pb-0">
                     <div className="flex items-center gap-4">
                        <div className={cn("w-2 h-2 rounded-full", 
                          monitor.currentStatus === 'up' ? "bg-emerald-500" :
                          monitor.currentStatus === 'down' ? "bg-red-500" :
                          "bg-slate-500"
                        )} />
                        <div>
                          <p className="text-sm font-medium leading-none">{monitor.name}</p>
                          <p className="text-xs text-muted-foreground mt-1">{monitor.url}</p>
                        </div>
                     </div>
                     <div className="text-right">
                        <p className="text-sm font-medium">{monitor.currentStatus?.toUpperCase()}</p>
                        <p className="text-xs text-muted-foreground">{formatTimestamp(monitor.lastCheckedAtTs)}</p>
                     </div>
                   </div>
                 ))}
               </div>
             ) : (
               <p className="text-sm text-muted-foreground">No monitors found.</p>
             )}
          </CardContent>
        </Card>
        <Card className="col-span-3">
          <CardHeader>
             <CardTitle>System Status</CardTitle>
             <CardDescription>Operational log.</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center gap-2 text-sm">
                <div className="w-2 h-2 rounded-full bg-emerald-500" />
                <span className="text-muted-foreground">Worker Scheduler Active</span>
              </div>
               <div className="flex items-center gap-2 text-sm">
                <div className="w-2 h-2 rounded-full bg-emerald-500" />
                <span className="text-muted-foreground">D1 Database Connected</span>
              </div>
               <div className="flex items-center gap-2 text-sm">
                <div className="w-2 h-2 rounded-full bg-emerald-500" />
                <span className="text-muted-foreground">Analytics Engine Ready</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/",
		component: DashboardPage,
		getParentRoute: () => parentRoute,
	});
