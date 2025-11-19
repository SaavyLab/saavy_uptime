import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type SectionCardProps = {
	title?: ReactNode;
	description?: ReactNode;
	actions?: ReactNode;
	children: ReactNode;
	className?: string;
	contentClassName?: string;
};

export function SectionCard({
	title,
	description,
	actions,
	children,
	className,
	contentClassName,
}: SectionCardProps) {
	return (
		<section
			className={cn(
				"rounded-xl border border-border bg-card shadow-sm",
				className,
			)}
		>
			{title || description || actions ? (
				<div className="flex flex-wrap items-center justify-between gap-4 border-b border-border px-6 py-4">
					<div>
						{title ? (
							<h2 className="text-sm font-medium uppercase tracking-wider text-muted-foreground">
								{title}
							</h2>
						) : null}
						{description ? (
							<p className="text-sm text-muted-foreground">{description}</p>
						) : null}
					</div>
					{actions ? <div className="flex gap-2">{actions}</div> : null}
				</div>
			) : null}
			<div className={cn("px-6 py-5", contentClassName)}>{children}</div>
		</section>
	);
}
