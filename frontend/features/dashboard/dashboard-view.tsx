"use client";

import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { motion } from "motion/react";
import { ArrowDownAZ, ArrowUpAZ, ChevronRight, MoreHorizontal, Search, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { api } from "@/lib/api/client";
import type { DetailListItem } from "@/lib/api/types";
import { STATUS_LABELS, STATUS_OPTIONS, TYPE_LABELS, statusVariant } from "@/lib/constants";
import { progressPercent } from "@/lib/progress";
import { AppLink, navigate } from "@/lib/router";
import { formatDate } from "@/lib/utils";
import { CoverImage } from "@/components/common/cover-image";
import { EmptyState, ErrorState } from "@/components/common/async-state";
import { PageHeader } from "@/components/common/page-header";
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { type DashboardFilters, filterRecords, recordHref } from "./model";

const DEFAULT_FILTERS: DashboardFilters = { status: -1, keyword: "", sortBy: "time", sortOrder: "desc" };

export function DashboardView() {
  const queryClient = useQueryClient();
  const [filters, setFilters] = useState<DashboardFilters>(() => typeof window === "undefined" ? DEFAULT_FILTERS : ({ status: Number(localStorage.getItem("dashboard.filterStatus") ?? -1), keyword: localStorage.getItem("dashboard.searchKeyword") ?? "", sortBy: localStorage.getItem("dashboard.sortBy") === "name" ? "name" : "time", sortOrder: localStorage.getItem("dashboard.sortOrder") === "asc" ? "asc" : "desc" }));
  const [deleting, setDeleting] = useState<{ item: DetailListItem; hard: boolean } | null>(null);
  useEffect(() => {
    localStorage.setItem("dashboard.filterStatus", String(filters.status));
    localStorage.setItem("dashboard.searchKeyword", filters.keyword);
    localStorage.setItem("dashboard.sortBy", filters.sortBy);
    localStorage.setItem("dashboard.sortOrder", filters.sortOrder);
  }, [filters]);

  const recordsQuery = useQuery({ queryKey: ["records", "detail"], queryFn: async ({ signal }) => { const response = await api.getDetailList(signal); if (response.status !== 0) throw new Error(response.message || "获取追踪列表失败"); return response.data || []; } });
  const records = useMemo(() => recordsQuery.data || [], [recordsQuery.data]);
  const filtered = useMemo(() => filterRecords(records, filters), [records, filters]);
  const counts = useMemo(() => Object.fromEntries([-1, ...STATUS_OPTIONS.map((item) => item.value)].map((status) => [status, status === -1 ? records.filter((item) => !item.is_delete).length : records.filter((item) => !item.is_delete && item.user_status === status).length])), [records]);

  const deleteMutation = useMutation({
    mutationFn: async ({ item, hard }: { item: DetailListItem; hard: boolean }) => {
      const response = item.bangumi_id ? await api.deleteRecordByBangumi(Number(item.bangumi_id), hard) : item.imdb_id ? await api.deleteRecordByImdb(item.imdb_id, hard) : await api.deleteRecordByCustom(item.other_id || 0, hard);
      if (response.status !== 0) throw new Error(response.message || "删除失败");
    },
    onSuccess: () => { toast.success("追踪记录已删除"); setDeleting(null); void queryClient.invalidateQueries({ queryKey: ["records"] }); },
    onError: (error) => toast.error(error.message),
  });

  return <><PageHeader eyebrow="Library / 私人片库" title="我的追踪" description="按状态扫看片单，沿着进度刻度回到上次停下的位置。" actions={<Button onClick={() => navigate("/search")}><Search className="size-4" />搜索并添加</Button>} />
    {recordsQuery.isLoading ? <DashboardSkeleton /> : recordsQuery.isError ? <ErrorState message={recordsQuery.error.message} onRetry={() => void recordsQuery.refetch()} /> : <>
      <section aria-label="追踪状态筛选" className="mb-5 flex gap-2 overflow-x-auto pb-2">{[{ value: -1, label: "全部" }, ...STATUS_OPTIONS].map((item) => <button key={item.value} aria-pressed={filters.status === item.value} className={filters.status === item.value ? "flex min-h-11 shrink-0 items-center gap-2 rounded-xl bg-foreground px-4 text-sm font-semibold text-background" : "flex min-h-11 shrink-0 items-center gap-2 rounded-xl border border-border bg-card/65 px-4 text-sm font-semibold text-muted-foreground transition hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"} onClick={() => setFilters({ ...filters, status: item.value })}><span>{item.label}</span><span className={filters.status === item.value ? "font-mono text-[10px] text-primary" : "font-mono text-[10px]"}>{counts[item.value] || 0}</span></button>)}</section>
      <section aria-label="片库筛选工具" className="mb-6 grid gap-3 rounded-2xl border border-border bg-card/70 p-3 sm:grid-cols-[minmax(220px,1fr)_160px_48px]">
        <div className="relative"><Search className="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input aria-label="搜索标题或 ID" placeholder="搜索标题、Bangumi ID 或 IMDb ID" className="pl-10" value={filters.keyword} onChange={(event) => setFilters({ ...filters, keyword: event.target.value })} /></div>
        <Select value={filters.sortBy} onValueChange={(value: "name" | "time") => setFilters({ ...filters, sortBy: value })}><SelectTrigger aria-label="排序字段"><SelectValue /></SelectTrigger><SelectContent><SelectItem value="time">按更新时间</SelectItem><SelectItem value="name">按标题</SelectItem></SelectContent></Select>
        <Button variant="outline" size="icon" aria-label={filters.sortOrder === "asc" ? "当前升序，切换为降序" : "当前降序，切换为升序"} onClick={() => setFilters({ ...filters, sortOrder: filters.sortOrder === "asc" ? "desc" : "asc" })}>{filters.sortOrder === "asc" ? <ArrowUpAZ className="size-4" /> : <ArrowDownAZ className="size-4" />}</Button>
      </section>
      <div className="mb-4 flex items-center justify-between"><p className="text-sm text-muted-foreground">显示 <strong className="font-mono text-foreground">{filtered.length}</strong> 个条目</p>{filters.keyword || filters.status !== -1 ? <Button variant="ghost" size="sm" onClick={() => setFilters(DEFAULT_FILTERS)}>清除筛选</Button> : null}</div>
      {filtered.length === 0 ? <EmptyState title={records.length ? "没有符合条件的条目" : "片库还是空的"} description={records.length ? "调整状态或搜索词，看看其他记录。" : "从 Bangumi、IMDb 或自定义条目开始建立你的追踪时间线。"} action={!records.length ? <Button onClick={() => navigate("/search")}>添加第一个条目</Button> : undefined} /> : <motion.div layout className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{filtered.map((item) => <RecordCard key={item.id} item={item} onDelete={(hard) => setDeleting({ item, hard })} />)}</motion.div>}
    </>}
    <AlertDialog open={Boolean(deleting)} onOpenChange={(open) => { if (!open && !deleteMutation.isPending) setDeleting(null); }}><AlertDialogContent><AlertDialogHeader><AlertDialogTitle>{deleting?.hard ? "永久删除这条记录？" : "从片库移除这条记录？"}</AlertDialogTitle><AlertDialogDescription>{deleting?.hard ? `「${deleting.item.title || "未命名条目"}」的追踪记录将被永久删除，此操作无法恢复。` : `「${deleting?.item.title || "未命名条目"}」会被软删除，之后重新添加时可以恢复。`}</AlertDialogDescription></AlertDialogHeader><AlertDialogFooter><AlertDialogCancel disabled={deleteMutation.isPending}>取消</AlertDialogCancel><AlertDialogAction disabled={deleteMutation.isPending} onClick={(event) => { event.preventDefault(); if (deleting) deleteMutation.mutate(deleting); }}>{deleteMutation.isPending ? "正在删除…" : deleting?.hard ? "永久删除" : "移除记录"}</AlertDialogAction></AlertDialogFooter></AlertDialogContent></AlertDialog>
  </>;
}

function RecordCard({ item, onDelete }: { item: DetailListItem; onDelete: (hard: boolean) => void }) {
  const href = recordHref(item);
  const percent = progressPercent(item.recorder, item.episodes);
  const source = item.imdb_id ? "IMDb" : item.other_id ? "自定义" : "Bangumi";
  return <motion.article layout initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.22 }} className="group relative overflow-hidden rounded-2xl border border-border bg-card shadow-card transition duration-200 hover:-translate-y-0.5 hover:border-foreground/20 hover:shadow-popover">
    {href ? <AppLink href={href} aria-label={`查看${item.title || "未命名条目"}详情`} className="absolute inset-0 z-0 rounded-2xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-ring" /> : null}
    <div className="relative z-[1] flex gap-4 p-4 pointer-events-none"><CoverImage src={item.cover_url} alt={item.title || "未命名条目"} className="w-24 shrink-0 rounded-xl sm:w-28" sizes="112px" /><div className="min-w-0 flex-1 py-1"><div className="flex items-start justify-between gap-2"><div className="min-w-0"><div className="mb-2 flex flex-wrap gap-1.5"><Badge variant={statusVariant(item.user_status)}>{STATUS_LABELS[item.user_status ?? -1] || "未设置"}</Badge><Badge variant="outline">{source}</Badge></div><h2 className="line-clamp-2 text-pretty text-base font-semibold leading-snug">{item.title || "未命名条目"}</h2></div><DropdownMenu><DropdownMenuTrigger asChild><Button className="pointer-events-auto -mr-2 -mt-2" variant="ghost" size="icon-sm" aria-label="条目操作"><MoreHorizontal className="size-4" /></Button></DropdownMenuTrigger><DropdownMenuContent align="end"><DropdownMenuItem onSelect={() => href && navigate(href)}>查看详情<ChevronRight className="ml-auto size-4" /></DropdownMenuItem><DropdownMenuSeparator /><DropdownMenuItem className="text-destructive focus:text-destructive" onSelect={() => onDelete(false)}><Trash2 className="size-4" />软删除</DropdownMenuItem><DropdownMenuItem className="text-destructive focus:text-destructive" onSelect={() => onDelete(true)}><Trash2 className="size-4" />永久删除</DropdownMenuItem></DropdownMenuContent></DropdownMenu></div><div className="mt-3 text-xs text-muted-foreground">{TYPE_LABELS[item.type || 0] || "未知类型"}{item.episodes ? ` · ${item.episodes} 话` : ""}</div><div className="mt-4"><div className="mb-2 flex items-center justify-between gap-2 text-xs"><span className="font-mono text-foreground">{item.recorder ? `EP ${item.recorder.replace("|", " · ")}` : "尚未记录"}</span>{item.episodes ? <span className="text-muted-foreground">{percent}%</span> : null}</div><div className="flex gap-1" aria-label={item.episodes ? `观看进度 ${percent}%` : "总集数未知"}>{Array.from({ length: 12 }, (_, index) => <span key={index} className={index < Math.round(percent / 8.34) ? "h-1.5 flex-1 rounded-full bg-accent" : "h-1.5 flex-1 rounded-full bg-muted"} />)}</div></div><div className="mt-3 font-mono text-[10px] text-muted-foreground">更新于 {formatDate(item.updated_at)}</div></div></div>
  </motion.article>;
}

function DashboardSkeleton() { return <div><div className="mb-5 flex gap-2"><Skeleton className="h-11 w-24" /><Skeleton className="h-11 w-24" /><Skeleton className="h-11 w-24" /></div><Skeleton className="mb-6 h-16 w-full" /><div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{Array.from({ length: 6 }, (_, index) => <Card key={index} className="flex gap-4 p-4"><Skeleton className="h-36 w-28 shrink-0" /><div className="flex-1"><Skeleton className="h-5 w-2/3" /><Skeleton className="mt-3 h-4 w-full" /><Skeleton className="mt-10 h-2 w-full" /></div></Card>)}</div></div>; }
