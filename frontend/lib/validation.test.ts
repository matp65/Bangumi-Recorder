import { describe, expect, it } from "vitest";
import {
  customItemSchema,
  loginSchema,
  passwordSchema,
  progressSchema,
} from "./validation";

describe("form validation", () => {
  it("requires credentials and a six-character replacement password", () => {
    expect(loginSchema.safeParse({ username: "", password: "" }).success).toBe(
      false,
    );
    expect(
      passwordSchema.safeParse({ oldPassword: "old", newPassword: "12345" })
        .success,
    ).toBe(false);
    expect(
      passwordSchema.safeParse({ oldPassword: "old", newPassword: "123456" })
        .success,
    ).toBe(true);
  });

  it("accepts mm:ss progress and rejects invalid seconds", () => {
    expect(
      progressSchema.safeParse({ episode: 3, time: "12:59" }).success,
    ).toBe(true);
    expect(
      progressSchema.safeParse({ episode: 3, time: "12:89" }).success,
    ).toBe(false);
  });

  it("validates custom item URLs without requiring a cover", () => {
    expect(
      customItemSchema.safeParse({
        title: "自定义条目",
        description: "",
        cover: "",
      }).success,
    ).toBe(true);
    expect(
      customItemSchema.safeParse({
        title: "自定义条目",
        description: "",
        cover: "not-a-url",
      }).success,
    ).toBe(false);
  });
});
