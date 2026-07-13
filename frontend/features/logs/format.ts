import type { RecordingLogItem, SystemLogItem } from "@/lib/api/types";
import { formatSeconds } from "@/lib/progress";
import { formatUnknown } from "@/lib/utils";

export const ACTION_LABELS: Record<string, string> = {
  recorder_changed: "进度变更",
  status_changed: "状态变更",
  recording_created: "创建记录",
  other_metadata_changed: "自定义条目变更",
  episode_created: "剧集记录创建",
  episode_updated: "剧集记录更新",
  recording_deleted: "删除记录",
  recording_hard_deleted: "硬删除记录",
  other_recording_deleted: "删除自定义条目",
  jwt_issued: "登录",
  api_token_created: "创建 API Token",
  api_token_updated: "修改 API Token",
  api_token_deleted: "删除 API Token",
  recording_logs_read: "读取记录日志",
  system_logs_read: "读取系统日志",
};

export function actionLabel(action: string) {
  return ACTION_LABELS[action] || action;
}
export function targetLabel(record: RecordingLogItem) {
  return (
    record.target_title ||
    `${record.target_type} #${record.target_id || record.recording_id || "—"}`
  );
}

function objectValue(value: unknown, key: string) {
  return value && typeof value === "object" && key in value
    ? (value as Record<string, unknown>)[key]
    : undefined;
}

export function recordingValue(record: RecordingLogItem, value: unknown) {
  if (!["episode_created", "episode_updated"].includes(record.action))
    return compact(formatUnknown(value));
  const ordinal = objectValue(record.metadata, "ordinal");
  const progress = objectValue(value, "progress_seconds");
  const duration = objectValue(value, "duration_seconds");
  const watched = objectValue(value, "watched") === true ? "已看" : "未看";
  const progressText =
    typeof progress === "number" ? formatSeconds(progress) : "0:00";
  const durationText =
    typeof duration === "number" && duration > 0
      ? `/${formatSeconds(duration)}`
      : "";
  return `EP ${ordinal ?? "—"} · ${progressText}${durationText} · ${watched}`;
}

export function recordingMetadata(record: RecordingLogItem) {
  const source = objectValue(record.metadata, "source");
  const ordinal = objectValue(record.metadata, "ordinal");
  const bangumiId = objectValue(record.metadata, "bangumi_id");
  const changes = objectValue(record.metadata, "changes");
  const parts = [
    source ? `来源: ${source}` : "",
    ordinal !== undefined ? `第 ${ordinal} 集` : "",
    bangumiId !== undefined ? `Bangumi: ${bangumiId}` : "",
    Array.isArray(changes) ? `变更 ${changes.length} 项` : "",
  ].filter(Boolean);
  return parts.join("；") || compact(formatUnknown(record.metadata));
}

export function systemMetadata(record: SystemLogItem) {
  const metadata = record.metadata;
  const extra = objectValue(metadata, "extra");
  const parts = [
    objectValue(metadata, "auth_type")
      ? `认证: ${objectValue(metadata, "auth_type")}`
      : "",
    objectValue(metadata, "ip") ? `IP: ${objectValue(metadata, "ip")}` : "",
  ];
  if (extra && typeof extra === "object") {
    const values = extra as Record<string, unknown>;
    if (values.username) parts.push(`用户: ${values.username}`);
    if (values.name) parts.push(`Token: ${values.name}`);
    if (values.token_id) parts.push(`Token ID: ${values.token_id}`);
  }
  return parts.filter(Boolean).join("；") || formatUnknown(metadata);
}

export function compact(text: string, max = 76) {
  return text.length > max ? `${text.slice(0, max)}…` : text;
}
