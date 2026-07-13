import { describe, expect, it } from "vitest";
import type { RecordingLogItem } from "@/lib/api/types";
import {
  actionLabel,
  recordingMetadata,
  recordingValue,
  targetLabel,
} from "./format";

const episodeLog: RecordingLogItem = {
  id: 1,
  recording_id: 9,
  user_id: 2,
  target_type: "bangumi",
  target_id: 42,
  target_title: "测试番剧",
  action: "episode_updated",
  field_name: "episode",
  old_value: null,
  new_value: { watched: true, progress_seconds: 125, duration_seconds: 1_500 },
  metadata: { ordinal: 3, source: "bangumi", changes: ["watched"] },
  created_at: "2026-07-12 12:00:00",
};

describe("log transforms", () => {
  it("maps backend actions to user-facing labels", () => {
    expect(actionLabel("episode_updated")).toBe("剧集记录更新");
    expect(actionLabel("future_action")).toBe("future_action");
  });

  it("summarizes episode payloads and metadata", () => {
    expect(recordingValue(episodeLog, episodeLog.new_value)).toBe(
      "EP 3 · 2:05/25:00 · 已看",
    );
    expect(recordingMetadata(episodeLog)).toContain("变更 1 项");
    expect(targetLabel(episodeLog)).toBe("测试番剧");
  });
});
