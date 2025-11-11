import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type DescriptionItem = {
	label: ReactNode;
	value: ReactNode;
};

type DescriptionListProps = {
	items: DescriptionItem[];
	className?: string;
	itemClassName?: string;
};

export function DescriptionList({
	items,
	className,
	itemClassName,
}: DescriptionListProps) {
	return (
		<dl
			className={cn("grid gap-4 text-sm text-[var(--text-muted)]", className)}
		>
			{items.map((item) => (
				<div
					key={String(item.label)}
					className={cn("space-y-1", itemClassName)}
				>
					<dt className="text-xs uppercase tracking-[0.3em] text-[var(--text-soft)]">
						{item.label}
					</dt>
					<dd className="font-mono text-[var(--text-primary)]">{item.value}</dd>
				</div>
			))}
		</dl>
	);
}
