const API_BASE = "/v1";

function getToken(): string | null {
	return localStorage.getItem("jupiter_token");
}

function setToken(token: string) {
	localStorage.setItem("jupiter_token", token);
}

function clearToken() {
	localStorage.removeItem("jupiter_token");
	localStorage.removeItem("jupiter_user");
}

function getUser(): User | null {
	const u = localStorage.getItem("jupiter_user");
	return u ? JSON.parse(u) : null;
}

interface User {
	id: string;
	username: string;
	email: string;
	display_name: string;
	bio: string;
}

function setUser(user: User) {
	localStorage.setItem("jupiter_user", JSON.stringify(user));
}

async function request(path: string, options: RequestInit = {}) {
	const token = getToken();
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
		...((options.headers as Record<string, string>) || {}),
	};
	if (token) {
		headers["Authorization"] = `Bearer ${token}`;
	}

	const res = await fetch(`${API_BASE}${path}`, {
		...options,
		headers,
	});

	if (res.status === 401) {
		clearToken();
		window.location.reload();
		throw new Error("Unauthorized");
	}

	const data = await res.json();
	if (!res.ok) {
		throw new Error(data.error || "Request failed");
	}
	return data;
}

// Auth
export async function register(
	username: string,
	email: string,
	password: string,
	displayName?: string,
) {
	const data = await request("/auth/register", {
		method: "POST",
		body: JSON.stringify({
			username,
			email,
			password,
			display_name: displayName,
		}),
	});
	setToken(data.token);
	setUser(data.user);
	return data;
}

export async function login(username: string, password: string) {
	const data = await request("/auth/login", {
		method: "POST",
		body: JSON.stringify({ username, password }),
	});
	setToken(data.token);
	setUser(data.user);
	return data;
}

export function logout() {
	clearToken();
	window.location.reload();
}

export function isLoggedIn(): boolean {
	return !!getToken();
}

export function getCurrentUser() {
	return getUser();
}

export async function getProfile() {
	return request("/auth/profile");
}

export async function updateProfile(displayName?: string, bio?: string) {
	return request("/auth/profile", {
		method: "PUT",
		body: JSON.stringify({ display_name: displayName, bio }),
	});
}

// Chat
export async function getChatHistory() {
	return request("/chat");
}

export async function sendMessage(content: string) {
	return request("/chat", {
		method: "POST",
		body: JSON.stringify({ content }),
	});
}

// Agent
export async function getAgentProfile() {
	return request("/agent/profile");
}

export async function triggerProfileUpdate() {
	return request("/agent/profile/update", { method: "POST" });
}

// Matching
export async function triggerMatching() {
	return request("/matching/trigger", { method: "POST" });
}

export async function getMatches() {
	return request("/matches");
}

// Notifications
export async function getNotifications() {
	return request("/notifications");
}

export async function getUnreadCount() {
	return request("/notifications/unread");
}

export async function markNotificationRead(id: number) {
	return request(`/notifications/${id}/read`, { method: "POST" });
}

// Direct Messages
export async function getDirectMessages(matchId: number) {
	return request(`/messages/${matchId}`);
}

export async function sendDirectMessage(matchId: number, content: string) {
	return request(`/messages/${matchId}`, {
		method: "POST",
		body: JSON.stringify({ content }),
	});
}
