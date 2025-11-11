import { createFormHookContexts } from "@tanstack/react-form";

const contexts = createFormHookContexts();

export const { fieldContext, formContext, useFieldContext, useFormContext } =
	contexts;
