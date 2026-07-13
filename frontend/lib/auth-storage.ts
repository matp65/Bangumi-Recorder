const TOKEN_KEY = "token";
const USERNAME_KEY = "username";

export function getStoredToken() {
  return typeof window === "undefined" ? null : window.localStorage.getItem(TOKEN_KEY);
}

export function getStoredUsername() {
  return typeof window === "undefined" ? null : window.localStorage.getItem(USERNAME_KEY);
}

export function storeSession(token: string, username: string) {
  window.localStorage.setItem(TOKEN_KEY, token);
  window.localStorage.setItem(USERNAME_KEY, username);
}

export function clearStoredSession() {
  if (typeof window === "undefined") return;
  window.localStorage.removeItem(TOKEN_KEY);
  window.localStorage.removeItem(USERNAME_KEY);
}

export function decodeJwtExpiration(token: string): number | null {
  try {
    const payload = token.split(".")[1];
    if (!payload) return null;
    const base64 = payload.replace(/-/g, "+").replace(/_/g, "/").padEnd(Math.ceil(payload.length / 4) * 4, "=");
    const value: unknown = JSON.parse(atob(base64));
    if (!value || typeof value !== "object" || !("exp" in value) || typeof value.exp !== "number") return null;
    return value.exp;
  } catch {
    return null;
  }
}

export function isTokenExpired(token: string, now = Date.now()) {
  const expiration = decodeJwtExpiration(token);
  return expiration === null || now >= expiration * 1000;
}
