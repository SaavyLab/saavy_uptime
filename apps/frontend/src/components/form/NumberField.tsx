import { useId } from "react";
import { useStore } from "@tanstack/react-form";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

import { useFieldContext } from "./formContexts";

type NumberFieldProps = {
	label: string;
	placeholder?: string;
	className?: string;
	description?: string;
	min?: number;
	max?: number;
	step?: number;
};

export function NumberField({
	label,
	placeholder,
	className,
	description,
	min,
	max,
	step,
}: NumberFieldProps) {
	const autoId = useId();
	const field = useFieldContext<number>();
	const state = useStore(field.store, (s) => s);
	const inputId = `${field.name}-${autoId}`;
	const errorMessage = state.meta.errors?.[0];

	return (
		<div className="space-y-2">
			<Label htmlFor={inputId} className="font-semibold tracking-wide">
				{label}
			</Label>
			<Input
				id={inputId}
				type="number"
				placeholder={placeholder}
				value={
					typeof state.value === "number" && Number.isFinite(state.value)
						? state.value
						: ""
				}
				min={min}
				max={max}
				step={step}
				onChange={(event) => {
					const raw = event.target.value;
					const nextValue = raw === "" ? undefined : Number(raw);
					field.handleChange(nextValue as never);
				}}
				onBlur={() => field.handleBlur()}
				className={cn(errorMessage && "border-destructive", className)}
			/>
			{description ? (
				<p className="text-sm text-muted-foreground">{description}</p>
			) : null}
			{errorMessage ? (
				<p className="text-sm text-destructive" role="alert">
					{String(errorMessage)}
				</p>
			) : null}
		</div>
	);
}
