"use client";

import Image from "next/image";
import { useState } from "react";
import { Clapperboard } from "lucide-react";
import { cn } from "@/lib/utils";

export function CoverImage({ src, alt, className, sizes = "160px", priority = false }: { src?: string | null; alt: string; className?: string; sizes?: string; priority?: boolean }) {
  const [failed, setFailed] = useState(false);
  if (!src || failed) return <div role="img" aria-label={`${alt}暂无封面`} className={cn("grid aspect-[3/4] place-items-center bg-muted text-muted-foreground", className)}><Clapperboard aria-hidden className="size-8 opacity-55" /></div>;
  return <Image unoptimized priority={priority} src={src} alt={alt} width={480} height={640} sizes={sizes} onError={() => setFailed(true)} className={cn("aspect-[3/4] object-cover", className)} />;
}
