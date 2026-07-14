"use client";

import { useEffect } from "react";
import { CircleAlert, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function GlobalError({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  useEffect(() => {
    console.error(error);
  }, [error]);
  return (
    <main className="grid min-h-screen place-items-center p-6">
      <div className="max-w-md text-center">
        <CircleAlert className="mx-auto mb-5 size-10 text-destructive" />
        <h1 className="font-display text-3xl font-semibold">
          页面没有正常打开
        </h1>
        <p className="mt-3 text-muted-foreground">
          界面运行时遇到异常。你可以重新加载此区域，未保存的表单内容可能需要重新填写。
        </p>
        <Button className="mt-6" onClick={reset}>
          <RotateCcw className="size-4" />
          重试
        </Button>
      </div>
    </main>
  );
}
