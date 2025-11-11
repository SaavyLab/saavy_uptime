import { createFormHook } from "@tanstack/react-form";

import { BooleanSwitchField } from "./BooleanSwitchField";
import { fieldContext, formContext } from "./formContexts";
import { NumberField } from "./NumberField";
import { SubmitButton } from "./SubmitButton";
import { TextField } from "./TextField";

export const { useAppForm } = createFormHook({
	fieldComponents: {
		TextField,
		NumberField,
		BooleanSwitchField,
	},
	formComponents: {
		SubmitButton,
	},
	fieldContext,
	formContext,
});
