import type * as React from "react";

import { cn } from "@/lib/utils";

function Input({ className, type, ...props }: React.ComponentProps<"input">) {
	return (
		<input
			type={type}
			data-slot="input"
			className={cn(
				"font-mono file:text-foreground placeholder:text-muted-foreground selection:bg-[#ff6633] selection:text-black border-black dark:border-white h-10 w-full min-w-0 border-2 bg-white dark:bg-black px-3 py-2 text-base transition-colors outline-none file:inline-flex file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-bold disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50",
				"focus-visible:border-[#ff6633] focus-visible:ring-2 focus-visible:ring-[#ff6633]",
				"aria-invalid:ring-2 aria-invalid:ring-[#ff0000] aria-invalid:border-[#ff0000]",
				className,
			)}
			{...props}
		/>
	);
}

export { Input };
