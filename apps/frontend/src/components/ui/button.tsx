import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@/lib/utils";

const buttonVariants = cva(
	"relative inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full border border-white/10 px-5 py-2.5 text-sm font-semibold tracking-tight text-[var(--text-primary)] transition duration-200 ease-out shadow-[0_12px_30px_rgba(0,0,0,0.35)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)] disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 [&_svg]:shrink-0",
	{
		variants: {
			variant: {
				default:
					"border-transparent bg-gradient-to-r from-[var(--accent)] via-[var(--accent-strong)] to-[var(--accent)] text-[#0b0905] shadow-[0_18px_45px_rgba(239,111,46,0.35)] hover:brightness-[1.05]",
				secondary:
					"border-white/15 bg-white/5 text-[var(--text-primary)] hover:border-white/30 hover:bg-white/10",
				outline:
					"border-white/30 bg-transparent text-[var(--text-primary)] hover:border-white/50 hover:bg-white/5",
				ghost:
					"border-transparent bg-transparent text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-white/5",
				destructive:
					"border-transparent bg-gradient-to-r from-[var(--accent-red)] to-[#ff3a55] text-white shadow-[0_18px_45px_rgba(255,71,120,0.35)] hover:brightness-[1.05]",
				link: "border-none px-0 py-0 text-[var(--accent)] underline-offset-8 hover:underline",
			},
			size: {
				default: "h-11 px-6",
				sm: "h-9 px-4 text-xs",
				lg: "h-12 px-7 text-base",
				icon: "size-10 rounded-full p-0",
				"icon-sm": "size-9 rounded-full p-0",
				"icon-lg": "size-12 rounded-full p-0",
			},
		},
		defaultVariants: {
			variant: "default",
			size: "default",
		},
	},
);

function Button({
	className,
	variant,
	size,
	asChild = false,
	...props
}: React.ComponentProps<"button"> &
	VariantProps<typeof buttonVariants> & {
		asChild?: boolean;
	}) {
	const Comp = asChild ? Slot : "button";

	return (
		<Comp
			data-slot="button"
			className={cn(buttonVariants({ variant, size, className }))}
			{...props}
		/>
	);
}

export { Button, buttonVariants };
