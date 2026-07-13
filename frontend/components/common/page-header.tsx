import { cn } from "@/lib/utils";

export function PageHeader({ eyebrow, title, description, actions, className }: { eyebrow?: string; title: string; description?: string; actions?: React.ReactNode; className?: string }) {
  return <header className={cn("mb-7 flex flex-col justify-between gap-5 border-b border-border pb-6 sm:flex-row sm:items-end", className)}><div className="min-w-0">{eyebrow ? <div className="mb-2 font-mono text-[11px] font-semibold uppercase tracking-[0.18em] text-primary">{eyebrow}</div> : null}<h1 className="text-pretty font-display text-4xl font-semibold leading-[1.03] tracking-[-0.035em] sm:text-5xl">{title}</h1>{description ? <p className="mt-3 max-w-2xl text-pretty text-[15px] leading-relaxed text-muted-foreground sm:text-base">{description}</p> : null}</div>{actions ? <div className="flex shrink-0 flex-wrap gap-2">{actions}</div> : null}</header>;
}
