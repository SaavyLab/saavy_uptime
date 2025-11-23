const getAccessToken = () => {
	const cookies = document.cookie.split(";");
	const cfCookie = cookies.find((cookie) =>
		cookie.trim().startsWith("CF_Authorization="),
	);
	if (!cfCookie) {
		return import.meta.env.VITE_CF_ACCESS_TOKEN;
	}
	return cfCookie.split("=")[1];
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

export const apiBase = "http://localhost:8787";