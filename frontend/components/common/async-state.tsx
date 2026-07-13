import { CircleAlert, Inbox, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

export function ErrorState({ title = "内容加载失败", message, onRetry }: { title?: string; message?: string; onRetry?: () => void }) {
  return <Card role="alert" className="grid min-h-64 place-items-center border-dashed p-8 text-center"><div><CircleAlert className="mx-auto mb-4 size-9 text-destructive" /><h2 className="font-display text-2xl font-semibold">{title}</h2><p className="mx-auto mt-2 max-w-md text-sm leading-relaxed text-muted-foreground">{message || "后端暂时没有返回可用内容，请检查服务状态后重试。"}</p>{onRetry ? <Button variant="outline" className="mt-5" onClick={onRetry}><RotateCcw className="size-4" />重新加载</Button> : null}</div></Card>;
}

export function EmptyState({ title, description, action }: { title: string; description: string; action?: React.ReactNode }) {
  return <Card className="grid min-h-64 place-items-center border-dashed bg-card/60 p-8 text-center"><div><Inbox className="mx-auto mb-4 size-9 text-muted-foreground/65" /><h2 className="font-display text-2xl font-semibold">{title}</h2><p className="mx-auto mt-2 max-w-md text-sm leading-relaxed text-muted-foreground">{description}</p>{action ? <div className="mt-5">{action}</div> : null}</div></Card>;
}
