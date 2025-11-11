import { useStore } from "@tanstack/react-form";
import { useId } from "react";

import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

import { useFieldContext } from "./formContexts";

type BooleanSwitchFieldProps = {
	label: string;
	description?: string;
};

export function BooleanSwitchField({
	label,
	description,
}: BooleanSwitchFieldProps) {
	const autoId = useId();
	const field = useFieldContext<boolean>();
	const state = useStore(field.store, (s) => s);
	const switchId = `${field.name}-${autoId}`;
	const checked = Boolean(state.value ?? false);
	const errorMessage = state.meta.errors?.[0];

	return (
		<div className="space-y-2">
			<div className="flex items-center justify-between gap-4">
				<div>
					<Label htmlFor={switchId} className="tracking-[0.3em]">
						{label}
					</Label>
					{description ? (
						<p className="text-sm text-[var(--text-muted)]">{description}</p>
					) : null}
				</div>
				<Switch
					id={switchId}
					checked={checked}
					onCheckedChange={(value) =>
						field.handleChange(Boolean(value) as never)
					}
					onBlur={() => field.handleBlur()}
				/>
			</div>
			{errorMessage ? (
				<p className="text-sm text-destructive" role="alert">
					{String(errorMessage)}
				</p>
			) : null}
		</div>
	);
}
