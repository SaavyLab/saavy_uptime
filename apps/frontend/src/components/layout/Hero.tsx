import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type HeroProps = {
	eyebrow?: string;
	title: ReactNode;
	description?: ReactNode;
	actions?: ReactNode;
	sideContent?: ReactNode;
	className?: string;
	titleClassName?: string;
	descriptionClassName?: string;
};

export function Hero({
	eyebrow,
	title,
	description,
	actions,
	sideContent,
	className,
	titleClassName,
	descriptionClassName,
}: HeroProps) {
	return (
		<section
			className={cn(
				"grid gap-8 rounded-[34px] border border-white/10 bg-gradient-to-br from-white/10 via-white/5 to-white/[0.04] p-6 sm:p-8 shadow-[var(--shadow-card)]",
				sideContent ? "lg:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]" : "",
				className,
			)}
		>
			<div className="space-y-6">
				{eyebrow ? (
					<span className="inline-flex items-center gap-2 rounded-full border border-white/20 bg-white/10 px-4 py-1 text-[0.65rem] font-semibold uppercase tracking-[0.5em] text-[var(--text-soft)]">
						{eyebrow}
					</span>
				) : null}
				<div className="space-y-4">
					<h1
						className={cn(
							"text-balance text-3xl font-medium leading-tight md:text-4xl",
							sideContent ? "text-4xl md:text-5xl" : "",
							titleClassName,
						)}
					>
						{title}
					</h1>
					{description ? (
						<p
							className={cn(
								"text-sm text-[var(--text-muted)] md:text-base",
								descriptionClassName,
							)}
						>
							{description}
						</p>
					) : null}
				</div>
				{actions ? <div className="flex flex-wrap gap-3">{actions}</div> : null}
			</div>
			{sideContent ? <div className="space-y-4">{sideContent}</div> : null}
		</section>
	);
}
