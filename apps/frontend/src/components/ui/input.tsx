import type * as React from "react";

import { cn } from "@/lib/utils";

function Input({ className, type, ...props }: React.ComponentProps<"input">) {
	return (
		<input
			type={type}
			data-slot="input"
			className={cn(
				"font-mono text-base text-[var(--text-primary)] placeholder:text-[var(--text-soft)] selection:bg-[var(--accent)] selection:text-[#0b0b0b] h-11 w-full min-w-0 rounded-2xl border border-white/15 bg-white/5 px-4 py-2 transition-colors outline-none backdrop-blur",
				"focus-visible:border-[var(--accent)] focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]",
				"aria-invalid:border-[var(--accent-red)] aria-invalid:ring-2 aria-invalid:ring-[var(--accent-red)]/40",
				"file:inline-flex file:h-7 file:rounded-full file:border file:border-white/20 file:bg-white/10 file:px-3 file:text-xs file:font-semibold file:uppercase file:tracking-wide file:text-[var(--text-primary)]",
				"disabled:pointer-events-none disabled:opacity-50",
				className,
			)}
			{...props}
		/>
	);
}

export { Input };
