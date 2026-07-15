"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Download, Eye, FilterX, RefreshCcw } from "lucide-react";
import { toast } from "sonner";
import { api } from "@/lib/api/client";
import type {
  RecordingLogFilters,
  RecordingLogItem,
  SystemLogFilters,
  SystemLogItem,
} from "@/lib/api/types";
import { useAuth } from "@/lib/auth-context";
import { formatDate, formatUnknown } from "@/lib/utils";
import { EmptyState, ErrorState } from "@/components/common/async-state";
import { PageHeader } from "@/components/common/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  actionLabel,
  compact,
  recordingValue,
  systemMetadata,
  targetLabel,
} from "./format";

const PAGE_SIZE = 50;
const recordingActions = [
  "recorder_changed",
  "status_changed",
  "recording_created",
  "recording_restored",
  "other_metadata_changed",
  "episode_created",
  "episode_updated",
  "recording_deleted",
  "recording_hard_deleted",
];
const systemActions = [
  "jwt_issued",
  "api_token_created",
  "api_token_updated",
  "api_token_deleted",
  "recording_logs_read",
  "system_logs_read",
  "stale_episode_metadata_cleaned",
  "animeko_episode_ordinals_repaired",
  "episode_ordinal_compatibility_mapped",
];

export function LogsView() {
  const { user } = useAuth();
  const [tab, setTab] = useState<"recording" | "system">("recording");
  const [page, setPage] = useState(1);
  const [recordingDraft, setRecordingDraft] = useState({
    start_time: "",
    end_time: "",
    target: "",
    action: "all",
  });
  const [systemDraft, setSystemDraft] = useState({
    start_time: "",
    end_time: "",
    category: "all",
    action: "all",
    username: "",
  });
  const [recordingFilters, setRecordingFilters] = useState<RecordingLogFilters>(
    {},
  );
  const [systemFilters, setSystemFilters] = useState<SystemLogFilters>({});
  const [detail, setDetail] = useState<RecordingLogItem | SystemLogItem | null>(
    null,
  );
  const [exporting, setExporting] = useState(false);

  const recordingQuery = useQuery({
    queryKey: ["logs", "recording", page, recordingFilters],
    enabled: tab === "recording",
    queryFn: async () => {
      const response = await api.listRecordingLogs(
        page,
        PAGE_SIZE,
        recordingFilters,
      );
      if (response.status !== 0)
        throw new Error(response.message || "记录日志加载失败");
      return response.data?.items || [];
    },
  });
  const systemQuery = useQuery({
    queryKey: ["logs", "system", page, systemFilters],
    enabled: tab === "system" && Boolean(user?.is_admin),
    queryFn: async () => {
      const response = await api.listSystemLogs(page, PAGE_SIZE, systemFilters);
      if (response.status !== 0)
        throw new Error(response.message || "系统日志加载失败");
      return response.data?.items || [];
    },
  });
  const activeQuery = tab === "recording" ? recordingQuery : systemQuery;
  const items = activeQuery.data || [];

  function applyFilters() {
    setPage(1);
    if (tab === "recording")
      setRecordingFilters(normalizeFilters(recordingDraft));
    else setSystemFilters(normalizeFilters(systemDraft));
  }
  function resetFilters() {
    setPage(1);
    if (tab === "recording") {
      setRecordingDraft({
        start_time: "",
        end_time: "",
        target: "",
        action: "all",
      });
      setRecordingFilters({});
    } else {
      setSystemDraft({
        start_time: "",
        end_time: "",
        category: "all",
        action: "all",
        username: "",
      });
      setSystemFilters({});
    }
  }
  async function exportCsv() {
    setExporting(true);
    try {
      const all: RecordingLogItem[] = [];
      for (let current = 1; ; current += 1) {
        const response = await api.listRecordingLogs(
          current,
          100,
          recordingFilters,
        );
        if (response.status !== 0 || !response.data)
          throw new Error(response.message || "日志导出失败");
        all.push(...response.data.items);
        if (response.data.items.length < 100) break;
      }
      if (!all.length) {
        toast.info("当前筛选条件下没有可导出的日志");
        return;
      }
      const rows = [
        ["时间", "对象", "动作", "字段", "旧值", "新值", "扩展"],
        ...all.map((item) => [
          item.created_at,
          targetLabel(item),
          actionLabel(item.action),
          item.field_name || "",
          formatUnknown(item.old_value),
          formatUnknown(item.new_value),
          formatUnknown(item.metadata),
        ]),
      ];
      const csv = rows.map((row) => row.map(csvCell).join(",")).join("\n");
      const url = URL.createObjectURL(
        new Blob([`\ufeff${csv}`], { type: "text/csv;charset=utf-8" }),
      );
      const link = document.createElement("a");
      link.href = url;
      link.download = `recording-logs-${new Date().toISOString().slice(0, 10)}.csv`;
      link.click();
      URL.revokeObjectURL(url);
      toast.success("日志 CSV 已生成");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "日志导出失败");
    } finally {
      setExporting(false);
    }
  }

  return (
    <>
      <PageHeader
        eyebrow="Timeline / 时间线"
        title="变更日志"
        description="每次进度、状态与条目修改都会留下可追溯记录；管理员还可以查看系统审计日志。"
        actions={
          <>
            <Button
              variant="outline"
              onClick={() => void activeQuery.refetch()}
            >
              <RefreshCcw className="size-4" />
              刷新
            </Button>
            {tab === "recording" ? (
              <Button loading={exporting} onClick={exportCsv}>
                <Download className="size-4" />
                导出 CSV
              </Button>
            ) : null}
          </>
        }
      />
      <Tabs
        value={tab}
        onValueChange={(value) => {
          setTab(value as "recording" | "system");
          setPage(1);
        }}
      >
        <TabsList>
          <TabsTrigger value="recording">追踪记录</TabsTrigger>
          {user?.is_admin ? (
            <TabsTrigger value="system">系统审计</TabsTrigger>
          ) : null}
        </TabsList>
      </Tabs>
      <Card className="mt-5">
        <CardContent className="p-4 sm:p-5">
          <FilterBar
            tab={tab}
            recording={recordingDraft}
            system={systemDraft}
            setRecording={setRecordingDraft}
            setSystem={setSystemDraft}
            onApply={applyFilters}
            onReset={resetFilters}
          />
        </CardContent>
      </Card>
      <section
        aria-live="polite"
        aria-busy={activeQuery.isLoading}
        className="mt-5"
      >
        {activeQuery.isLoading ? (
          <LogSkeleton />
        ) : activeQuery.isError ? (
          <ErrorState
            message={activeQuery.error.message}
            onRetry={() => void activeQuery.refetch()}
          />
        ) : !items.length ? (
          <EmptyState
            title="没有匹配日志"
            description="修改筛选条件或等待新的记录产生。"
          />
        ) : (
          <>
            {tab === "recording" ? (
              <RecordingTable
                items={items as RecordingLogItem[]}
                onOpen={setDetail}
              />
            ) : (
              <SystemTable
                items={items as SystemLogItem[]}
                onOpen={setDetail}
              />
            )}
            <div className="mt-5 flex items-center justify-center gap-3">
              <Button
                variant="outline"
                disabled={page <= 1}
                onClick={() => setPage((value) => value - 1)}
              >
                上一页
              </Button>
              <span className="font-mono text-xs text-muted-foreground">
                PAGE {page}
              </span>
              <Button
                variant="outline"
                disabled={items.length < PAGE_SIZE}
                onClick={() => setPage((value) => value + 1)}
              >
                下一页
              </Button>
            </div>
          </>
        )}
      </section>
      <LogDetail
        item={detail}
        onOpenChange={(open) => !open && setDetail(null)}
      />
    </>
  );
}

function FilterBar({
  tab,
  recording,
  system,
  setRecording,
  setSystem,
  onApply,
  onReset,
}: {
  tab: "recording" | "system";
  recording: {
    start_time: string;
    end_time: string;
    target: string;
    action: string;
  };
  system: {
    start_time: string;
    end_time: string;
    category: string;
    action: string;
    username: string;
  };
  setRecording: React.Dispatch<React.SetStateAction<typeof recording>>;
  setSystem: React.Dispatch<React.SetStateAction<typeof system>>;
  onApply: () => void;
  onReset: () => void;
}) {
  return (
    <form
      className="grid gap-3 md:grid-cols-2 xl:grid-cols-6"
      onSubmit={(event) => {
        event.preventDefault();
        onApply();
      }}
    >
      <Input
        aria-label="开始时间"
        type="datetime-local"
        value={tab === "recording" ? recording.start_time : system.start_time}
        onChange={(event) =>
          tab === "recording"
            ? setRecording({ ...recording, start_time: event.target.value })
            : setSystem({ ...system, start_time: event.target.value })
        }
      />
      <Input
        aria-label="结束时间"
        type="datetime-local"
        value={tab === "recording" ? recording.end_time : system.end_time}
        onChange={(event) =>
          tab === "recording"
            ? setRecording({ ...recording, end_time: event.target.value })
            : setSystem({ ...system, end_time: event.target.value })
        }
      />
      {tab === "recording" ? (
        <>
          <Input
            aria-label="日志对象"
            placeholder="标题 / 类型 / ID"
            value={recording.target}
            onChange={(event) =>
              setRecording({ ...recording, target: event.target.value })
            }
          />
          <Select
            value={recording.action}
            onValueChange={(value) =>
              setRecording({ ...recording, action: value })
            }
          >
            <SelectTrigger aria-label="记录动作">
              <SelectValue placeholder="全部动作" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部动作</SelectItem>
              {recordingActions.map((action) => (
                <SelectItem key={action} value={action}>
                  {actionLabel(action)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </>
      ) : (
        <>
          <Select
            value={system.category}
            onValueChange={(value) => setSystem({ ...system, category: value })}
          >
            <SelectTrigger aria-label="系统类别">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部类别</SelectItem>
              {["auth", "api_token", "logs", "settings", "cleanup"].map(
                (category) => (
                  <SelectItem key={category} value={category}>
                    {category}
                  </SelectItem>
                ),
              )}
            </SelectContent>
          </Select>
          <Select
            value={system.action}
            onValueChange={(value) => setSystem({ ...system, action: value })}
          >
            <SelectTrigger aria-label="系统动作">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部动作</SelectItem>
              {systemActions.map((action) => (
                <SelectItem key={action} value={action}>
                  {actionLabel(action)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Input
            aria-label="操作用户"
            placeholder="用户名 / 用户 ID"
            value={system.username}
            onChange={(event) =>
              setSystem({ ...system, username: event.target.value })
            }
          />
        </>
      )}
      <div className="flex gap-2">
        <Button type="submit" className="flex-1">
          筛选
        </Button>
        <Button
          type="button"
          variant="outline"
          size="icon"
          aria-label="清除筛选"
          onClick={onReset}
        >
          <FilterX className="size-4" />
        </Button>
      </div>
    </form>
  );
}

function RecordingTable({
  items,
  onOpen,
}: {
  items: RecordingLogItem[];
  onOpen: (item: RecordingLogItem) => void;
}) {
  return (
    <>
      <div className="hidden overflow-x-auto rounded-2xl border border-border bg-card md:block">
        <table className="w-full min-w-[900px] text-left text-sm">
          <thead className="bg-muted/60 font-mono text-[10px] uppercase tracking-[0.14em] text-muted-foreground">
            <tr>
              <th className="px-4 py-3">时间</th>
              <th className="px-4 py-3">对象</th>
              <th className="px-4 py-3">动作</th>
              <th className="px-4 py-3">旧值</th>
              <th className="px-4 py-3">新值</th>
              <th className="w-16 px-4 py-3">
                <span className="sr-only">操作</span>
              </th>
            </tr>
          </thead>
          <tbody>
            {items.map((item) => (
              <tr
                key={item.id}
                className="border-t border-border transition hover:bg-muted/35"
              >
                <td className="whitespace-nowrap px-4 py-3 font-mono text-xs text-muted-foreground">
                  {formatDate(item.created_at, true)}
                </td>
                <td className="max-w-56 truncate px-4 py-3 font-medium">
                  {targetLabel(item)}
                </td>
                <td className="px-4 py-3">
                  <Badge variant="outline">{actionLabel(item.action)}</Badge>
                </td>
                <td className="max-w-52 truncate px-4 py-3 font-mono text-xs text-muted-foreground">
                  {recordingValue(item, item.old_value)}
                </td>
                <td className="max-w-52 truncate px-4 py-3 font-mono text-xs">
                  {recordingValue(item, item.new_value)}
                </td>
                <td className="px-3">
                  <Button
                    variant="ghost"
                    size="icon-sm"
                    aria-label={`查看日志 ${item.id}`}
                    onClick={() => onOpen(item)}
                  >
                    <Eye className="size-4" />
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="grid gap-3 md:hidden">
        {items.map((item) => (
          <button
            key={item.id}
            className="rounded-xl border border-border bg-card p-4 text-left shadow-card focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onClick={() => onOpen(item)}
          >
            <div className="flex items-start justify-between gap-3">
              <span className="font-semibold">{targetLabel(item)}</span>
              <Badge variant="outline">{actionLabel(item.action)}</Badge>
            </div>
            <div className="mt-3 font-mono text-xs text-muted-foreground">
              {recordingValue(item, item.old_value)} →{" "}
              {recordingValue(item, item.new_value)}
            </div>
            <div className="mt-3 text-xs text-muted-foreground">
              {formatDate(item.created_at, true)}
            </div>
          </button>
        ))}
      </div>
    </>
  );
}

function SystemTable({
  items,
  onOpen,
}: {
  items: SystemLogItem[];
  onOpen: (item: SystemLogItem) => void;
}) {
  return (
    <div className="grid gap-3">
      {items.map((item) => (
        <button
          key={item.id}
          className="grid min-h-20 gap-2 rounded-xl border border-border bg-card p-4 text-left shadow-card transition hover:border-foreground/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring md:grid-cols-[170px_120px_160px_minmax(0,1fr)] md:items-center"
          onClick={() => onOpen(item)}
        >
          <span className="font-mono text-xs text-muted-foreground">
            {formatDate(item.created_at, true)}
          </span>
          <Badge variant={item.level === "error" ? "destructive" : "outline"}>
            {item.category}
          </Badge>
          <span className="text-sm font-semibold">
            {actionLabel(item.action)}
          </span>
          <span className="min-w-0">
            <span className="block truncate text-sm">{item.message}</span>
            <span className="mt-1 block truncate font-mono text-xs text-muted-foreground">
              {compact(systemMetadata(item), 100)}
            </span>
          </span>
        </button>
      ))}
    </div>
  );
}

function LogDetail({
  item,
  onOpenChange,
}: {
  item: RecordingLogItem | SystemLogItem | null;
  onOpenChange: (open: boolean) => void;
}) {
  const recording = item && "target_type" in item ? item : null;
  return (
    <Dialog open={Boolean(item)} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl grid-rows-[auto_minmax(0,1fr)] gap-0 overflow-hidden p-0">
        <DialogHeader className="border-b border-border px-5 py-4 sm:px-6 sm:py-5">
          <DialogTitle>日志详情</DialogTitle>
          <DialogDescription>
            {item ? `${formatDate(item.created_at, true)} · #${item.id}` : ""}
          </DialogDescription>
        </DialogHeader>
        {item ? (
          <div
            data-slot="log-detail-scroll"
            className="min-h-0 touch-pan-y overflow-y-auto overscroll-contain px-5 py-4 [scrollbar-gutter:stable] [-webkit-overflow-scrolling:touch] sm:px-6 sm:py-5"
          >
            <div className="grid gap-4 pb-[env(safe-area-inset-bottom)]">
              <div className="grid gap-3 rounded-xl border border-border bg-muted/35 p-4 text-sm sm:grid-cols-2">
                <Meta label="动作" value={actionLabel(item.action)} />
                <Meta
                  label={recording ? "对象" : "类别"}
                  value={
                    recording
                      ? targetLabel(recording)
                      : (item as SystemLogItem).category
                  }
                />
                {recording ? (
                  <Meta label="字段" value={recording.field_name || "—"} />
                ) : (
                  <Meta
                    label="用户"
                    value={(item as SystemLogItem).username || "—"}
                  />
                )}
              </div>
              {recording ? (
                <>
                  <RawBlock title="旧值" value={recording.old_value} />
                  <RawBlock title="新值" value={recording.new_value} />
                  <RawBlock title="扩展数据" value={recording.metadata} />
                </>
              ) : (
                <>
                  <p className="text-sm leading-relaxed">
                    {(item as SystemLogItem).message}
                  </p>
                  <RawBlock title="扩展数据" value={item.metadata} />
                </>
              )}
            </div>
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}

function RawBlock({ title, value }: { title: string; value: unknown }) {
  return (
    <div>
      <div className="mb-2 text-sm font-semibold">{title}</div>
      <pre className="overflow-x-auto rounded-xl border border-border bg-muted/70 p-4 font-mono text-xs leading-relaxed text-foreground whitespace-pre-wrap break-words">
        {formatUnknown(value, true)}
      </pre>
    </div>
  );
}
function Meta({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <div className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
        {label}
      </div>
      <div className="mt-1 font-medium">{value}</div>
    </div>
  );
}
function normalizeFilters<T extends Record<string, string>>(values: T) {
  return Object.fromEntries(
    Object.entries(values)
      .filter(([, value]) => value && value !== "all")
      .map(([key, value]) => [
        key,
        key.endsWith("time") ? value.replace("T", " ") : value,
      ]),
  );
}
function csvCell(value: string) {
  return `"${value.replace(/"/g, '""')}"`;
}
function LogSkeleton() {
  return (
    <div className="grid gap-3">
      {Array.from({ length: 7 }, (_, index) => (
        <Skeleton key={index} className="h-20" />
      ))}
    </div>
  );
}
