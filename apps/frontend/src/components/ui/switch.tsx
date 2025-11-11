import * as SwitchPrimitive from "@radix-ui/react-switch";
import type * as React from "react";

import { cn } from "@/lib/utils";

function Switch({
	className,
	...props
}: React.ComponentProps<typeof SwitchPrimitive.Root>) {
	return (
		<SwitchPrimitive.Root
			data-slot="switch"
			className={cn(
				"peer inline-flex h-6 w-11 shrink-0 items-center rounded-full border border-white/20 bg-white/10 transition-all outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] data-[state=checked]:border-[var(--accent)] data-[state=checked]:bg-[var(--accent-muted)] disabled:cursor-not-allowed disabled:opacity-50",
				className,
			)}
			{...props}
		>
			<SwitchPrimitive.Thumb
				data-slot="switch-thumb"
				className={cn(
					"pointer-events-none block size-5 translate-x-1 rounded-full bg-white shadow-[0_8px_15px_rgba(0,0,0,0.35)] transition-transform data-[state=checked]:translate-x-[calc(100%-6px)]",
				)}
			/>
		</SwitchPrimitive.Root>
	);
}

export { Switch };
