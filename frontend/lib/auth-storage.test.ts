import { describe, expect, it } from "vitest";
import { decodeJwtExpiration, isTokenExpired } from "./auth-storage";

function token(payload: object) {
  const encode = (value: object) =>
    btoa(JSON.stringify(value))
      .replace(/=/g, "")
      .replace(/\+/g, "-")
      .replace(/\//g, "_");
  return `${encode({ alg: "none" })}.${encode(payload)}.signature`;
}

describe("JWT session validation", () => {
  it("reads a base64url expiration", () => {
    expect(decodeJwtExpiration(token({ exp: 2_000_000_000 }))).toBe(
      2_000_000_000,
    );
  });

  it("expires missing, malformed, and past tokens", () => {
    expect(isTokenExpired("not-a-token", 1_000)).toBe(true);
    expect(isTokenExpired(token({ name: "user" }), 1_000)).toBe(true);
    expect(isTokenExpired(token({ exp: 10 }), 10_001)).toBe(true);
  });

  it("keeps future tokens active", () => {
    expect(isTokenExpired(token({ exp: 20 }), 10_000)).toBe(false);
  });
});
