import type * as React from "react";

import { cn } from "@/lib/utils";

function Textarea({ className, ...props }: React.ComponentProps<"textarea">) {
	return (
		<textarea
			data-slot="textarea"
			className={cn(
				"font-mono text-[var(--text-primary)] placeholder:text-[var(--text-soft)] selection:bg-[var(--accent)]/80 selection:text-[#0b0b0b] flex min-h-32 w-full rounded-2xl border border-white/15 bg-white/5 px-4 py-3 text-base leading-relaxed transition-colors outline-none backdrop-blur",
				"focus-visible:border-[var(--accent)] focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]",
				"aria-invalid:border-[var(--accent-red)] aria-invalid:ring-2 aria-invalid:ring-[var(--accent-red)]/40",
				"disabled:cursor-not-allowed disabled:opacity-50",
				className,
			)}
			{...props}
		/>
	);
}

export { Textarea };
