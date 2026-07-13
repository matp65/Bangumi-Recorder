"use client";

import { useEffect, useState } from "react";
import { Clapperboard, Eye, EyeOff, LockKeyhole, UserRound } from "lucide-react";
import { toast } from "sonner";
import { api, ApiNetworkError } from "@/lib/api/client";
import { useAuth } from "@/lib/auth-context";
import { navigate, useCurrentRoute } from "@/lib/router";
import { loginSchema } from "@/lib/validation";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function LoginView() {
  const route = useCurrentRoute();
  const { login, register } = useAuth();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [allowRegister, setAllowRegister] = useState(false);
  const [registerNeedToken, setRegisterNeedToken] = useState(false);
  const [showPassword, setShowPassword] = useState(false);
  const [remember, setRemember] = useState(() => typeof window !== "undefined" && window.localStorage.getItem("remember_account") === "true");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [form, setForm] = useState(() => ({ username: typeof window === "undefined" ? "" : window.localStorage.getItem("remembered_username") || "", password: "", registerToken: "" }));

  useEffect(() => {
    api.getConfig().then((response) => {
      if (response.status === 0 && response.data) {
        setAllowRegister(response.data.allow_register);
        setRegisterNeedToken(response.data.register_need_token);
      }
    }).catch(() => setAllowRegister(false));
  }, []);

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    setError("");
    const parsed = loginSchema.safeParse(form);
    if (!parsed.success) { setError(parsed.error.issues[0]?.message || "请检查输入"); return; }
    setLoading(true);
    try {
      const result = mode === "login"
        ? await login(parsed.data.username, parsed.data.password)
        : await register(parsed.data.username, parsed.data.password, form.registerToken || undefined);
      if (!result.ok) { setError(result.message || (mode === "login" ? "用户名或密码不正确" : "注册失败，请检查输入")); return; }
      window.localStorage.setItem("remember_account", String(remember));
      if (remember) window.localStorage.setItem("remembered_username", parsed.data.username);
      else window.localStorage.removeItem("remembered_username");
      toast.success(mode === "login" ? "登录成功" : "账户已创建");
      const next = new URLSearchParams(route.split("?")[1] || "").get("next");
      navigate(next?.startsWith("/") ? next : "/", { replace: true });
    } catch (caught) {
      setError(caught instanceof ApiNetworkError ? caught.message : "登录服务暂时不可用");
    } finally { setLoading(false); }
  }

  return <main className="grid min-h-screen lg:grid-cols-[minmax(0,1.1fr)_minmax(420px,.9fr)]">
    <section className="relative hidden overflow-hidden border-r border-border bg-secondary p-12 text-foreground lg:flex lg:flex-col lg:justify-between xl:p-16">
      <div className="absolute inset-0 opacity-[0.08]" style={{ backgroundImage: "linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)", backgroundSize: "72px 72px" }} />
      <div className="relative flex items-center gap-3"><span className="grid size-11 place-items-center rounded-xl bg-background text-foreground"><Clapperboard className="size-5" /></span><span className="font-display text-xl font-semibold">Bangumi Recorder</span></div>
      <div className="relative max-w-2xl">
        <div className="font-mono text-xs uppercase tracking-[0.2em] text-primary">Now recording</div>
        <h1 className="mt-5 text-pretty font-display text-6xl font-semibold leading-[.95] tracking-[-0.045em] xl:text-7xl">把每一集，留在自己的时间线上。</h1>
        <p className="mt-6 max-w-xl text-lg leading-relaxed text-muted-foreground">从搜索、进度到单集记录，所有变化都清楚可查。打开片库，继续上次停下的位置。</p>
        <div className="mt-10 flex items-center gap-1" aria-hidden>{Array.from({ length: 18 }, (_, index) => <span key={index} className={index < 11 ? "h-2 flex-1 bg-primary" : "h-2 flex-1 bg-primary/10"} />)}</div>
      </div>
      <div className="relative font-mono text-[10px] uppercase tracking-[0.18em] text-muted-foreground">Private library / Same-origin API / Dark ready</div>
    </section>
    <section className="flex items-center justify-center px-4 py-12 sm:px-8">
      <div className="w-full max-w-md">
        <div className="mb-8 lg:hidden"><div className="flex items-center gap-3"><span className="grid size-11 place-items-center rounded-xl bg-foreground text-background"><Clapperboard className="size-5" /></span><span className="font-display text-xl font-semibold">Bangumi Recorder</span></div></div>
        <div className="mb-7"><div className="font-mono text-[11px] uppercase tracking-[0.2em] text-primary">{mode === "login" ? "Welcome back" : "New account"}</div><h2 className="mt-3 font-display text-4xl font-semibold tracking-tight">{mode === "login" ? "继续你的片单" : "建立新的片库"}</h2><p className="mt-2 text-muted-foreground">{mode === "login" ? "登录后同步追踪记录与设置。" : "注册后即可开始添加和记录条目。"}</p></div>
        <Card className="p-5 sm:p-7"><form onSubmit={submit} noValidate className="grid gap-5">
          <div className="grid gap-2"><Label htmlFor="username">用户名</Label><div className="relative"><UserRound className="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input id="username" name="username" autoComplete="username" className="pl-10" value={form.username} onChange={(event) => setForm({ ...form, username: event.target.value })} /></div></div>
          <div className="grid gap-2"><Label htmlFor="password">密码</Label><div className="relative"><LockKeyhole className="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" /><Input id="password" name="password" type={showPassword ? "text" : "password"} autoComplete={mode === "login" ? "current-password" : "new-password"} className="px-10" value={form.password} onChange={(event) => setForm({ ...form, password: event.target.value })} /><button type="button" className="absolute right-1 top-1/2 grid size-9 -translate-y-1/2 place-items-center rounded-md text-muted-foreground hover:bg-muted focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" aria-label={showPassword ? "隐藏密码" : "显示密码"} onClick={() => setShowPassword((value) => !value)}>{showPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}</button></div></div>
          {mode === "register" && registerNeedToken ? <div className="grid gap-2"><Label htmlFor="register-token">注册令牌</Label><Input id="register-token" type="password" autoComplete="off" value={form.registerToken} onChange={(event) => setForm({ ...form, registerToken: event.target.value })} /></div> : null}
          {mode === "login" ? <label className="flex min-h-10 cursor-pointer items-center gap-2.5 text-sm text-muted-foreground"><Checkbox checked={remember} onCheckedChange={(value) => setRemember(value === true)} />记住账号名称</label> : null}
          <div aria-live="polite" className="min-h-5 text-sm font-medium text-destructive">{error}</div>
          <Button type="submit" size="lg" loading={loading}>{mode === "login" ? "登录并打开片库" : "创建账户"}</Button>
        </form></Card>
        {allowRegister ? <button className="mt-6 min-h-11 w-full text-sm font-semibold text-muted-foreground underline-offset-4 hover:text-foreground hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" onClick={() => { setMode((value) => value === "login" ? "register" : "login"); setError(""); }}>{mode === "login" ? "还没有账户？创建一个" : "已有账户？返回登录"}</button> : null}
      </div>
    </section>
  </main>;
}
