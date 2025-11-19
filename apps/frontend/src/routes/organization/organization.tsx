import { useQuery } from "@tanstack/react-query";
import type { Register, RootRoute } from "@tanstack/react-router";
import { createRoute } from "@tanstack/react-router";
import { Shield, Mail } from "lucide-react";
import { SectionCard } from "@/components/layout/SectionCard";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/Skeleton";
import { getOrganization, getOrganizationMembers } from "@/lib/organizations";
import type { RouterContext } from "@/router-context";

const formatTimestamp = (value?: number) => {
	if (!value) {
		return "—";
	}
	return new Date(value * 1000).toLocaleString();
};

function OrganizationPage() {
	const organizationQuery = useQuery({
		queryKey: ["organization"],
		queryFn: () => getOrganization(),
	});

	const membersQuery = useQuery({
		queryKey: ["organization", "members"],
		queryFn: () => getOrganizationMembers(),
	});

	const organization = organizationQuery.data;
	const members = membersQuery.data ?? [];

	return (
		<div className="space-y-8">
			<div className="flex items-center justify-between">
				<div className="space-y-1">
					<h1 className="text-2xl font-bold tracking-tight">
						{organization?.name ?? "Organization"}
					</h1>
					<p className="text-muted-foreground">
						{organization
							? `Manage settings and members for ${organization.slug}`
							: "Loading organization details..."}
					</p>
				</div>
			</div>

			{organizationQuery.isLoading ? (
				<div className="grid gap-8 lg:grid-cols-2">
					<Skeleton className="h-64 w-full" />
					<Skeleton className="h-64 w-full" />
				</div>
			) : organization ? (
				<div className="grid gap-8 lg:grid-cols-[1fr_1.5fr]">
					<SectionCard title="General Information">
						<dl className="divide-y divide-border">
							<div className="py-4 grid grid-cols-3 gap-4">
								<dt className="text-sm font-medium text-muted-foreground">
									Name
								</dt>
								<dd className="text-sm font-medium text-foreground col-span-2">
									{organization.name}
								</dd>
							</div>
							<div className="py-4 grid grid-cols-3 gap-4">
								<dt className="text-sm font-medium text-muted-foreground">
									Slug
								</dt>
								<dd className="text-sm font-mono text-foreground col-span-2">
									{organization.slug}
								</dd>
							</div>
							<div className="py-4 grid grid-cols-3 gap-4">
								<dt className="text-sm font-medium text-muted-foreground">
									ID
								</dt>
								<dd className="text-xs font-mono text-muted-foreground col-span-2 break-all">
									{organization.id}
								</dd>
							</div>
							<div className="py-4 grid grid-cols-3 gap-4">
								<dt className="text-sm font-medium text-muted-foreground">
									Created
								</dt>
								<dd className="text-sm text-foreground col-span-2">
									{formatTimestamp(organization.createdAt)}
								</dd>
							</div>
						</dl>
					</SectionCard>

					<SectionCard
						title="Members"
						description={`Manage access to ${organization.name}`}
						actions={
							<div className="text-xs font-medium text-muted-foreground">
								{members.length} members
							</div>
						}
						contentClassName="p-0"
					>
						{membersQuery.isLoading ? (
							<div className="p-6 space-y-2">
								<Skeleton className="h-10 w-full" />
								<Skeleton className="h-10 w-full" />
								<Skeleton className="h-10 w-full" />
							</div>
						) : (
							<div className="border-t border-border">
								<Table>
									<TableHeader>
										<TableRow className="hover:bg-transparent border-border">
											<TableHead className="pl-6">User</TableHead>
											<TableHead>Role</TableHead>
										</TableRow>
									</TableHeader>
									<TableBody>
										{members.map((member) => (
											<TableRow
												key={member.email}
												className="border-border hover:bg-muted/30"
											>
												<TableCell className="pl-6">
													<div className="flex items-center gap-3">
														<div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
															<Mail size={14} className="text-primary" />
														</div>
														<span className="font-medium">{member.email}</span>
													</div>
												</TableCell>
												<TableCell>
													<div className="flex items-center gap-2">
														<Shield
															size={14}
															className="text-muted-foreground"
														/>
														<span className="capitalize">{member.role}</span>
													</div>
												</TableCell>
											</TableRow>
										))}
										{members.length === 0 && (
											<TableRow>
												<TableCell
													colSpan={2}
													className="text-center text-muted-foreground h-24 border-border"
												>
													No members found
												</TableCell>
											</TableRow>
										)}
									</TableBody>
								</Table>
							</div>
						)}
					</SectionCard>
				</div>
			) : (
				<div className="p-12 text-center rounded-xl border border-dashed border-border">
					<p className="text-muted-foreground">Organization not found.</p>
				</div>
			)}
		</div>
	);
}

export default (parentRoute: RootRoute<Register, undefined, RouterContext>) =>
	createRoute({
		path: "/organization",
		component: OrganizationPage,
		getParentRoute: () => parentRoute,
	});
