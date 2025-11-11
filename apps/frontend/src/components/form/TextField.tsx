import { useStore } from "@tanstack/react-form";
import { useId } from "react";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

import { useFieldContext } from "./formContexts";

type TextFieldProps = {
	label: string;
	placeholder?: string;
	className?: string;
	description?: string;
	type?: React.ComponentProps<typeof Input>["type"];
};

export function TextField({
	label,
	placeholder,
	className,
	description,
	type = "text",
}: TextFieldProps) {
	const autoId = useId();
	const field = useFieldContext<string>();
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
				type={type}
				placeholder={placeholder}
				value={(state.value as string | undefined) ?? ""}
				onChange={(event) => field.handleChange(event.target.value as never)}
				onBlur={() => field.handleBlur()}
				className={cn(errorMessage && "border-destructive", className)}
				autoComplete="off"
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
