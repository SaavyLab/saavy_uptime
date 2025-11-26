const getAccessToken = () => {
	const cookies = document.cookie.split(";");
	const cfCookie = cookies.find((cookie) =>
		cookie.trim().startsWith("CF_Authorization="),
	);
	if (!cfCookie) {
		return import.meta.env.VITE_CF_ACCESS_TOKEN;
	}
	const trimmed = cfCookie.trim();
	const eqIndex = trimmed.indexOf("=");
	return eqIndex >= 0 ? trimmed.substring(eqIndex + 1) : trimmed;
};

export const withAccessHeader = (headers: Record<string, string> = {}) => {
	const accessToken = getAccessToken();
	if (!accessToken) {
		return headers;
	}

	return {
		...headers,
		"Cf-Access-Jwt-Assertion": accessToken,
	};
};
export const apiBase = import.meta.env.VITE_API_URL || "http://localhost:8787";
