import { Button } from "@/components/ui/button";

import { useFormContext } from "./formContexts";

type SubmitButtonProps = {
	label?: string;
	className?: string;
};

export function SubmitButton({ label = "Submit", className }: SubmitButtonProps) {
	const form = useFormContext();

	return (
		<form.Subscribe selector={(state) => state.isSubmitting}>
			{(isSubmitting) => (
				<Button type="submit" disabled={isSubmitting} className={className}>
					{isSubmitting ? "Submitting..." : label}
				</Button>
			)}
		</form.Subscribe>
	);
}