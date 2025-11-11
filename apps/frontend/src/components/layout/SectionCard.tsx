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
				"rounded-[32px] border border-white/10 bg-white/[0.02] shadow-[var(--shadow-soft)]",
				className,
			)}
		>
			{title || description || actions ? (
				<div className="flex flex-wrap items-center justify-between gap-4 border-b border-white/10 px-6 py-5">
					<div>
						{title ? (
							<h2 className="text-sm font-semibold uppercase tracking-[0.4em] text-[var(--text-soft)]">
								{title}
							</h2>
						) : null}
						{description ? (
							<p className="text-sm text-[var(--text-muted)]">{description}</p>
						) : null}
					</div>
					{actions ? <div className="flex gap-2">{actions}</div> : null}
				</div>
			) : null}
			<div className={cn("px-6 py-5", contentClassName)}>{children}</div>
		</section>
	);
}
