import { cn } from "@/lib/utils";

type SkeletonProps = {
	className?: string;
};

export function Skeleton({ className }: SkeletonProps) {
	return (
		<div
			className={cn(
				"h-4 animate-pulse rounded-2xl bg-white/10",
				"before:content-[''] before:block before:h-full before:w-full before:bg-white/5",
				className,
			)}
		/>
	);
}
