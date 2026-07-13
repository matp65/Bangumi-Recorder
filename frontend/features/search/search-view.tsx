"use client";

import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { Database, Globe2, Plus, Search, SlidersHorizontal } from "lucide-react";
import { toast } from "sonner";
import { api } from "@/lib/api/client";
import type { BangumiSearchItem, ImdbSearchItem, LocalSearchItem } from "@/lib/api/types";
import { STATUS_OPTIONS, TYPE_LABELS } from "@/lib/constants";
import { customItemSchema } from "@/lib/validation";
import { navigate, useCurrentRoute } from "@/lib/router";
import { CoverImage } from "@/components/common/cover-image";
import { EmptyState } from "@/components/common/async-state";
import { PageHeader } from "@/components/common/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";

type SearchItem = BangumiSearchItem | ImdbSearchItem | LocalSearchItem;
type Scope = "online" | "local";
type Source = "bangumi" | "imdb";

export function SearchView() {
  const route = useCurrentRoute();
  const [initial] = useState(() => new URLSearchParams(route.split("?")[1] || ""));
  const [tab, setTab] = useState(initial.get("tab") === "custom" ? "custom" : "search");
  const [scope, setScope] = useState<Scope>(initial.get("scope") === "local" ? "local" : "online");
  const [source, setSource] = useState<Source>(initial.get("source") === "imdb" ? "imdb" : "bangumi");
  const [useOmdb, setUseOmdb] = useState(false);
  const [keyword, setKeyword] = useState(initial.get("q") || "");
  const [idSearch, setIdSearch] = useState("");
  const [page, setPage] = useState(Number(initial.get("page")) || 1);
  const [results, setResults] = useState<SearchItem[]>([]);
  const [hasSearched, setHasSearched] = useState(false);
  const [total, setTotal] = useState(0);
  const [addingKey, setAddingKey] = useState<string | null>(null);
  const [custom, setCustom] = useState({ title: "", description: "", cover: "", maxNumber: "", status: "2", recorder: "" });
  const [customError, setCustomError] = useState("");

  const searchMutation = useMutation({
    mutationFn: async ({ nextPage, byId }: { nextPage: number; byId?: string }) => {
      if (byId) {
        if (scope === "local") {
          const numericId = Number(byId); if (!Number.isInteger(numericId) || numericId <= 0) throw new Error("请输入有效的本地条目 ID");
          const response = await api.searchLocal(undefined, numericId, 1, 20); if (response.status !== 0) throw new Error(response.message || "本地搜索失败");
          return { items: response.data?.items || [], total: response.data?.total || 0 };
        }
        if (source === "imdb") {
          const response = await api.searchImdbById(byId, false, useOmdb); if (response.status !== 0 || !response.data) throw new Error(response.message || "未找到该 IMDb 条目");
          const item: ImdbSearchItem = { source: "imdb", imdb_id: response.data.imdb_id, external_id: response.data.external_id, title: response.data.title, year: response.data.release_date?.slice(0, 4) || null, cover: response.data.cover_url, info: [TYPE_LABELS[response.data.type], response.data.author].filter(Boolean).join(" · "), type: response.data.type };
          return { items: [item], total: 1 };
        }
        const id = Number(byId); if (!Number.isInteger(id) || id <= 0) throw new Error("请输入有效的 Bangumi ID");
        const response = await api.searchBangumiById(id); if (response.status !== 0 || !response.data) throw new Error(response.message || "未找到该 Bangumi 条目");
        const item: BangumiSearchItem = { source: "bangumi", bangumi_id: response.data.bangumi_id, title: response.data.title, alias: "", cover: response.data.cover_url, info: `${TYPE_LABELS[response.data.type] || "其他"} · ${response.data.episodes || "?"} 话`, type: response.data.type };
        return { items: [item], total: 1 };
      }
      const query = keyword.trim(); if (!query) throw new Error("请输入搜索关键词");
      if (scope === "local") { const response = await api.searchLocal(query, undefined, nextPage, 20); if (response.status !== 0) throw new Error(response.message || "本地搜索失败"); return { items: response.data?.items || [], total: response.data?.total || 0 }; }
      const response = source === "imdb" ? await api.searchImdb(query, nextPage, useOmdb) : await api.searchBangumi(query, nextPage);
      if (response.status !== 0) throw new Error(response.message || "在线搜索失败");
      return { items: response.data || [], total: 0 };
    }, onSuccess: (data) => { setResults(data.items); setTotal(data.total); setHasSearched(true); if (!data.items.length) toast.info("没有找到匹配条目"); }, onError: (error) => { setResults([]); setHasSearched(true); toast.error(error.message); }
  });

  function runSearch(nextPage = 1) {
    setPage(nextPage);
    navigate(`/search?scope=${scope}&source=${source}&q=${encodeURIComponent(keyword.trim())}&page=${nextPage}`, { replace: true });
    searchMutation.mutate({ nextPage });
  }

  const addMutation = useMutation({
    mutationFn: async (item: SearchItem) => {
      setAddingKey(itemKey(item));
      const ids = itemIds(item);
      const response = ids.bangumi ? await api.addRecord({ bangumi_id: Number(ids.bangumi), user_status: 2 }) : ids.imdb ? await api.addRecord({ source: "imdb", external_id: ids.imdb, user_status: 2, use_api: useOmdb }) : await api.addRecord({ other_id: ids.custom || 0, user_status: 2 });
      if (response.status !== 0) throw new Error(response.message?.includes("already exists") ? "该条目已经在片库中" : response.message || "添加失败");
      return item.title;
    }, onSuccess: (title) => toast.success(`已添加「${title}」`), onError: (error) => toast.error(error.message), onSettled: () => setAddingKey(null)
  });

  const createMutation = useMutation({
    mutationFn: async () => {
      const parsed = customItemSchema.safeParse({ title: custom.title, description: custom.description, cover: custom.cover, maxNumber: custom.maxNumber ? Number(custom.maxNumber) : undefined });
      if (!parsed.success) throw new Error(parsed.error.issues[0]?.message || "请检查输入");
      const response = await api.addRecord({ other_title: parsed.data.title, other_description: parsed.data.description || undefined, other_cover: parsed.data.cover || undefined, other_max_number: parsed.data.maxNumber, other_status: Number(custom.status), user_status: Number(custom.status), recorder: custom.recorder || undefined });
      if (response.status !== 0) throw new Error(response.message || "自定义条目创建失败");
      return parsed.data.title;
    }, onSuccess: (title) => { toast.success(`已创建「${title}」`); setCustom({ title: "", description: "", cover: "", maxNumber: "", status: "2", recorder: "" }); setCustomError(""); }, onError: (error) => setCustomError(error.message)
  });

  return <><PageHeader eyebrow="Discovery / 发现" title="搜索与添加" description="在线发现 Bangumi 或 IMDb 条目，也可以从本地缓存和自己的自定义记录中查找。" />
    <Tabs value={tab} onValueChange={(value) => { setTab(value); navigate(`/search${value === "custom" ? "?tab=custom" : ""}`, { replace: true }); }}>
      <TabsList className="grid w-full grid-cols-2 sm:w-[360px]"><TabsTrigger value="search">搜索条目</TabsTrigger><TabsTrigger value="custom">自定义条目</TabsTrigger></TabsList>
      <TabsContent value="search">
        <Card className="overflow-hidden"><div className="grid lg:grid-cols-[minmax(0,1fr)_280px]"><div className="p-5 sm:p-7"><div className="flex items-center gap-2 font-mono text-[11px] uppercase tracking-[0.17em] text-primary">{scope === "local" ? <Database className="size-4" /> : <Globe2 className="size-4" />}{scope === "local" ? "Local index" : `${source} source`}</div><h2 className="mt-3 font-display text-3xl font-semibold tracking-tight">{scope === "local" ? "搜索本地索引" : source === "imdb" ? "搜索 IMDb" : "搜索 Bangumi"}</h2><p className="mt-2 text-sm text-muted-foreground">{scope === "local" ? "本地搜索会同时返回 Bangumi、IMDb 和自定义条目。" : "输入标题关键词，或使用准确 ID 直接定位。"}</p>
          <form className="mt-6 flex flex-col gap-3 sm:flex-row" onSubmit={(event) => { event.preventDefault(); runSearch(); }}><div className="relative flex-1"><Search className="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input aria-label="搜索关键词" className="pl-10" placeholder={scope === "local" ? "搜索本地标题或信息" : source === "imdb" ? "例如 Interstellar" : "例如 葬送的芙莉莲"} value={keyword} onChange={(event) => setKeyword(event.target.value)} /></div><Button type="submit" loading={searchMutation.isPending}>搜索</Button></form>
          <form className="mt-3 flex flex-col gap-3 sm:flex-row" onSubmit={(event) => { event.preventDefault(); searchMutation.mutate({ nextPage: 1, byId: idSearch.trim() }); }}><Input aria-label="条目 ID" placeholder={scope === "local" ? "本地 ID" : source === "imdb" ? "IMDb ID，如 tt0816692" : "Bangumi ID，如 425998"} value={idSearch} onChange={(event) => setIdSearch(event.target.value)} /><Button type="submit" variant="outline" loading={searchMutation.isPending}>按 ID 查找</Button></form></div>
          <aside className="border-t border-border bg-muted/45 p-5 lg:border-l lg:border-t-0"><div className="flex items-center gap-2 font-semibold"><SlidersHorizontal className="size-4" />搜索范围</div><div className="mt-5 grid gap-4"><div className="flex items-center justify-between gap-4"><Label htmlFor="scope-switch">使用本地缓存</Label><Switch id="scope-switch" checked={scope === "local"} onCheckedChange={(checked) => setScope(checked ? "local" : "online")} /></div>{scope === "online" ? <><div className="grid gap-2"><Label>在线来源</Label><Select value={source} onValueChange={(value: Source) => setSource(value)}><SelectTrigger><SelectValue /></SelectTrigger><SelectContent><SelectItem value="bangumi">Bangumi</SelectItem><SelectItem value="imdb">IMDb</SelectItem></SelectContent></Select></div>{source === "imdb" ? <div className="flex items-center justify-between gap-4"><Label htmlFor="omdb-switch">使用 OMDb API</Label><Switch id="omdb-switch" checked={useOmdb} onCheckedChange={setUseOmdb} /></div> : null}</> : null}</div></aside></div></Card>
        <section aria-live="polite" aria-busy={searchMutation.isPending} className="mt-7">{searchMutation.isPending ? <SearchSkeleton /> : hasSearched && !results.length ? <EmptyState title="没有找到匹配条目" description="尝试更短的关键词、切换在线来源，或直接输入条目 ID。" /> : results.length ? <><div className="mb-4 flex items-center justify-between"><h2 className="font-display text-2xl font-semibold">搜索结果</h2><span className="font-mono text-xs text-muted-foreground">{total ? `${total} 条` : `第 ${page} 页`}</span></div><div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{results.map((item) => <SearchResultCard key={itemKey(item)} item={item} adding={addingKey === itemKey(item)} onAdd={() => addMutation.mutate(item)} />)}</div><div className="mt-6 flex justify-center gap-2"><Button variant="outline" disabled={page <= 1} onClick={() => runSearch(page - 1)}>上一页</Button><Button variant="outline" disabled={scope === "local" ? page * 20 >= total : results.length < 20} onClick={() => runSearch(page + 1)}>下一页</Button></div></> : <div className="grid min-h-56 place-items-center text-center"><div><Search className="mx-auto size-8 text-muted-foreground/60" /><p className="mt-3 text-sm text-muted-foreground">输入关键词，开始建立你的下一条记录。</p></div></div>}</section>
      </TabsContent>
      <TabsContent value="custom"><Card className="mx-auto max-w-2xl"><CardHeader><CardTitle>创建自定义条目</CardTitle><CardDescription>记录不在公开资料库中的作品、清单或个人进度。</CardDescription></CardHeader><CardContent><form className="grid gap-5" onSubmit={(event) => { event.preventDefault(); createMutation.mutate(); }}><Field label="条目名称" htmlFor="custom-title"><Input id="custom-title" value={custom.title} maxLength={255} onChange={(event) => setCustom({ ...custom, title: event.target.value })} /></Field><Field label="描述" htmlFor="custom-description"><Textarea id="custom-description" value={custom.description} maxLength={2000} onChange={(event) => setCustom({ ...custom, description: event.target.value })} /></Field><div className="grid gap-5 sm:grid-cols-2"><Field label="封面 URL" htmlFor="custom-cover"><Input id="custom-cover" type="url" placeholder="https://" value={custom.cover} onChange={(event) => setCustom({ ...custom, cover: event.target.value })} /></Field><Field label="总数" htmlFor="custom-total"><Input id="custom-total" type="number" min="0" inputMode="numeric" value={custom.maxNumber} onChange={(event) => setCustom({ ...custom, maxNumber: event.target.value })} /></Field></div><div className="grid gap-5 sm:grid-cols-2"><Field label="状态" htmlFor="custom-status"><Select value={custom.status} onValueChange={(value) => setCustom({ ...custom, status: value })}><SelectTrigger id="custom-status"><SelectValue /></SelectTrigger><SelectContent>{STATUS_OPTIONS.map((item) => <SelectItem key={item.value} value={String(item.value)}>{item.label}</SelectItem>)}</SelectContent></Select></Field><Field label="初始进度" htmlFor="custom-progress"><Input id="custom-progress" placeholder="如 5|2:12" value={custom.recorder} onChange={(event) => setCustom({ ...custom, recorder: event.target.value })} /></Field></div><p aria-live="polite" className="min-h-5 text-sm font-medium text-destructive">{customError}</p><Button type="submit" loading={createMutation.isPending}><Plus className="size-4" />创建并加入片库</Button></form></CardContent></Card></TabsContent>
    </Tabs></>;
}

function Field({ label, htmlFor, children }: { label: string; htmlFor: string; children: React.ReactNode }) { return <div className="grid gap-2"><Label htmlFor={htmlFor}>{label}</Label>{children}</div>; }

function isBangumi(item: SearchItem): item is BangumiSearchItem { return "alias" in item; }
function isImdb(item: SearchItem): item is ImdbSearchItem { return "external_id" in item && item.source === "imdb"; }
function itemIds(item: SearchItem) { return { bangumi: isBangumi(item) ? item.bangumi_id : "bangumi_id" in item ? item.bangumi_id : null, imdb: isImdb(item) ? item.imdb_id : "imdb_id" in item ? item.imdb_id : null, custom: "other_id" in item ? item.other_id : null }; }
function itemKey(item: SearchItem) { const ids = itemIds(item); return ids.bangumi ? `bangumi-${ids.bangumi}` : ids.imdb ? `imdb-${ids.imdb}` : `custom-${ids.custom || item.title}`; }
function itemCover(item: SearchItem) { return item.cover || null; }
function itemInfo(item: SearchItem) { return isBangumi(item) ? item.info || item.alias : item.info || "暂无附加信息"; }
function itemSource(item: SearchItem) { return isBangumi(item) ? "Bangumi" : isImdb(item) ? "IMDb" : item.source === "custom" ? "自定义" : item.source; }
function itemHref(item: SearchItem) { const ids = itemIds(item); return ids.bangumi ? `/detail/${ids.bangumi}` : ids.imdb ? `/detail/imdb/${encodeURIComponent(ids.imdb)}` : ids.custom ? `/detail/custom/${ids.custom}` : null; }

function SearchResultCard({ item, adding, onAdd }: { item: SearchItem; adding: boolean; onAdd: () => void }) {
  const href = itemHref(item);
  return <Card className="flex min-w-0 gap-4 p-4 transition hover:border-foreground/20"><button type="button" className="shrink-0 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" disabled={!href} onClick={() => href && navigate(href)}><CoverImage src={itemCover(item)} alt={item.title} className="w-24 rounded-xl" sizes="96px" /></button><div className="min-w-0 flex-1 py-1"><div className="mb-2 flex flex-wrap gap-1.5"><Badge variant="outline">{itemSource(item)}</Badge>{isBangumi(item) || isImdb(item) ? <Badge>{TYPE_LABELS[item.type] || "其他"}</Badge> : null}</div><button type="button" disabled={!href} onClick={() => href && navigate(href)} className="line-clamp-2 text-left font-semibold leading-snug hover:text-primary disabled:cursor-default disabled:hover:text-foreground">{item.title}</button><p className="mt-2 line-clamp-2 text-xs leading-relaxed text-muted-foreground">{itemInfo(item)}</p><Button className="mt-4" size="sm" loading={adding} onClick={onAdd}><Plus className="size-3.5" />添加追踪</Button></div></Card>;
}

function SearchSkeleton() { return <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{Array.from({ length: 6 }, (_, index) => <Card key={index} className="flex gap-4 p-4"><Skeleton className="h-32 w-24 shrink-0" /><div className="flex-1"><Skeleton className="h-5 w-4/5" /><Skeleton className="mt-3 h-4 w-full" /><Skeleton className="mt-8 h-9 w-24" /></div></Card>)}</div>; }
