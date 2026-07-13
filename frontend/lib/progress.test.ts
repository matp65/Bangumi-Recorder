import { describe, expect, it } from "vitest";
import { buildProgress, formatSeconds, parseProgress, progressPercent } from "./progress";

describe("recording progress", () => {
  it("round-trips episode and playback time", () => {
    expect(parseProgress(buildProgress(8, "12:04"))).toEqual({ episode: 8, time: "12:04" });
  });

  it("handles empty and episode-only records", () => {
    expect(parseProgress(null)).toEqual({ episode: null, time: null });
    expect(parseProgress("5")).toEqual({ episode: 5, time: null });
  });

  it("clamps visual progress to a valid percentage", () => {
    expect(progressPercent("3", 12)).toBe(25);
    expect(progressPercent("30", 12)).toBe(100);
    expect(progressPercent("3", 0)).toBe(0);
  });

  it("formats episode playback seconds", () => {
    expect(formatSeconds(125)).toBe("2:05");
    expect(formatSeconds(3_725)).toBe("1:02:05");
  });
});
