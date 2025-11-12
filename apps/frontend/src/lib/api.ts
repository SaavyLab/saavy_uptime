export const withAccessHeader = (headers: Record<string, string> = {}) => {
	if (!import.meta.env.VITE_CF_ACCESS_TOKEN) {
		return headers;
	}

	return {
		...headers,
		CF_Authorization: import.meta.env.VITE_CF_ACCESS_TOKEN,
	};
};
