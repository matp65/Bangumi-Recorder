import { Skeleton } from "@/components/ui/skeleton";

export default function Loading() {
  return <main className="mx-auto grid min-h-screen max-w-6xl gap-4 px-4 py-12"><Skeleton className="h-12 w-52" /><Skeleton className="h-44 w-full" /><div className="grid gap-4 md:grid-cols-3"><Skeleton className="h-56" /><Skeleton className="h-56" /><Skeleton className="h-56" /></div></main>;
}
