import { createFormHook } from "@tanstack/react-form";

import { BooleanSwitchField } from "./BooleanSwitchField";
import { NumberField } from "./NumberField";
import { SubmitButton } from "./SubmitButton";
import { TextField } from "./TextField";
import { fieldContext, formContext } from "./formContexts";

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
