"use client";

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";
import { AnimatePresence, motion, useReducedMotion } from "motion/react";
import { useTheme } from "next-themes";
import {
  BookOpen,
  Clapperboard,
  History,
  LogOut,
  Menu,
  Moon,
  Search,
  Settings,
  Sun,
  UserRound,
  X,
} from "lucide-react";
import { useAuth } from "@/lib/auth-context";
import { AppLink, navigate, useCurrentRoute } from "@/lib/router";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { LoginView } from "@/features/auth/login-view";
import { DashboardView } from "@/features/dashboard/dashboard-view";
import { SearchView } from "@/features/search/search-view";
import { DetailView } from "@/features/detail/detail-view";
import { LogsView } from "@/features/logs/logs-view";
import { ProfileView } from "@/features/profile/profile-view";

const navigation = [
  { href: "/", label: "我的片库", icon: BookOpen },
  { href: "/search", label: "搜索添加", icon: Search },
  { href: "/logs", label: "变更日志", icon: History },
  { href: "/profile", label: "设置", icon: Settings },
];

function pathOnly(route: string) {
  return route.split("?")[0] || "/";
}

export function AppShell() {
  const route = useCurrentRoute();
  const path = pathOnly(route);
  const { hydrated, token, logout } = useAuth();
  const [menuOpen, setMenuOpen] = useState(false);
  const reduceMotion = useReducedMotion();

  useEffect(() => {
    if (!hydrated) return;
    if (!token && path !== "/login")
      navigate(`/login?next=${encodeURIComponent(route)}`, { replace: true });
    if (token && path === "/login") navigate("/", { replace: true });
  }, [hydrated, token, path, route]);

  if (
    !hydrated ||
    (!token && path !== "/login") ||
    (token && path === "/login")
  )
    return <ShellLoading />;
  if (path === "/login") return <LoginView />;

  const content = <RouteContent path={path} />;
  return (
    <div className="min-h-screen lg:grid lg:grid-cols-[248px_minmax(0,1fr)]">
      <DesktopSidebar
        path={path}
        onLogout={() => {
          logout();
          navigate("/login");
        }}
      />
      <MobileHeader onMenu={() => setMenuOpen(true)} />
      <AnimatePresence>
        {menuOpen ? (
          <MobileDrawer
            path={path}
            onClose={() => setMenuOpen(false)}
            onLogout={() => {
              logout();
              navigate("/login");
            }}
          />
        ) : null}
      </AnimatePresence>
      <main
        id="main-content"
        className="min-w-0 px-4 pb-28 pt-6 sm:px-6 lg:px-10 lg:pb-12 lg:pt-10 xl:px-14"
      >
        <AnimatePresence mode="wait" initial={false}>
          <motion.div
            key={path}
            initial={reduceMotion ? false : { opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={reduceMotion ? undefined : { opacity: 0, y: -4 }}
            transition={{
              duration: reduceMotion ? 0 : 0.2,
              ease: [0.22, 1, 0.36, 1],
            }}
            className="mx-auto max-w-[1320px]"
          >
            {content}
          </motion.div>
        </AnimatePresence>
      </main>
      <MobileNav path={path} />
    </div>
  );
}

function RouteContent({ path }: { path: string }) {
  if (path === "/") return <DashboardView />;
  if (path === "/search") return <SearchView />;
  if (path === "/logs") return <LogsView />;
  if (path === "/profile") return <ProfileView />;
  const custom = path.match(/^\/detail\/custom\/(\d+)\/?$/);
  if (custom?.[1]) return <DetailView source="custom" id={custom[1]} />;
  const imdb = path.match(/^\/detail\/imdb\/([^/]+)\/?$/);
  if (imdb?.[1])
    return <DetailView source="imdb" id={decodeURIComponent(imdb[1])} />;
  const bangumi = path.match(/^\/detail\/(\d+)\/?$/);
  if (bangumi?.[1]) return <DetailView source="bangumi" id={bangumi[1]} />;
  return (
    <div className="grid min-h-[60vh] place-items-center text-center">
      <div>
        <div className="font-mono text-xs tracking-widest text-primary">
          404 / FRAME NOT FOUND
        </div>
        <h1 className="mt-3 font-display text-5xl font-semibold">
          这一帧不存在
        </h1>
        <p className="mt-3 text-muted-foreground">
          地址可能已经变化，返回片库继续浏览。
        </p>
        <Button className="mt-6" onClick={() => navigate("/")}>
          返回我的片库
        </Button>
      </div>
    </div>
  );
}

function Brand() {
  return (
    <AppLink
      href="/"
      className="group flex items-center gap-3 rounded-xl focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
    >
      <span className="relative grid size-11 place-items-center overflow-hidden rounded-xl bg-foreground text-background">
        <Clapperboard className="size-5" />
        <span className="absolute bottom-0 left-0 h-1 w-7 bg-primary transition-all group-hover:w-11" />
      </span>
      <span>
        <strong className="block font-display text-lg leading-none tracking-tight">
          Bangumi
        </strong>
        <span className="mt-1 block font-mono text-[9px] uppercase tracking-[0.22em] text-muted-foreground">
          Recorder / 1.2
        </span>
      </span>
    </AppLink>
  );
}

function DesktopSidebar({
  path,
  onLogout,
}: {
  path: string;
  onLogout: () => void;
}) {
  return (
    <aside className="sticky top-0 hidden h-screen flex-col border-r border-border bg-card/75 px-5 py-6 backdrop-blur lg:flex">
      <Brand />
      <nav aria-label="主导航" className="mt-12 grid gap-1.5">
        {navigation.map((item) => (
          <NavItem
            key={item.href}
            {...item}
            active={
              item.href === "/" ? path === "/" : path.startsWith(item.href)
            }
          />
        ))}
      </nav>
      <div className="mt-auto">
        <div className="mb-5 rounded-xl border border-border bg-background/55 p-4">
          <div className="font-mono text-[10px] uppercase tracking-[0.16em] text-muted-foreground">
            记录原则
          </div>
          <p className="mt-2 text-sm leading-relaxed">
            一集一格，一次更新一条可回看的时间线。
          </p>
        </div>
        <div className="flex items-center justify-between">
          <UserMenu onLogout={onLogout} />
          <ThemeToggle />
        </div>
      </div>
    </aside>
  );
}

function NavItem({
  href,
  label,
  icon: Icon,
  active,
  onClick,
}: {
  href: string;
  label: string;
  icon: typeof Search;
  active: boolean;
  onClick?: () => void;
}) {
  return (
    <AppLink
      href={href}
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      className={cn(
        "group flex min-h-11 items-center gap-3 rounded-xl px-3.5 text-sm font-semibold transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
        active
          ? "bg-foreground text-background"
          : "text-muted-foreground hover:bg-muted hover:text-foreground",
      )}
    >
      <Icon
        className={cn(
          "size-[18px]",
          active ? "text-primary" : "group-hover:text-primary",
        )}
      />
      {label}
    </AppLink>
  );
}

function UserMenu({ onLogout }: { onLogout: () => void }) {
  const { user, username } = useAuth();
  const label = user?.nickname || username || "用户";
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label={`${label}的账户菜单`}>
          <UserRound className="size-5" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <div className="px-3 py-2">
          <div className="text-sm font-semibold">{label}</div>
          <div className="mt-0.5 text-xs text-muted-foreground">
            {user?.is_admin ? "管理员" : "普通用户"}
          </div>
        </div>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => navigate("/profile")}>
          <Settings className="size-4" />
          账户设置
        </DropdownMenuItem>
        <DropdownMenuItem
          className="text-destructive focus:text-destructive"
          onSelect={onLogout}
        >
          <LogOut className="size-4" />
          退出登录
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  const mounted = useSyncExternalStore(
    () => () => undefined,
    () => true,
    () => false,
  );
  if (!mounted) return <div className="size-11" />;
  const dark = resolvedTheme === "dark";
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label={dark ? "切换到亮色模式" : "切换到暗色模式"}
      onClick={() => setTheme(dark ? "light" : "dark")}
    >
      {dark ? <Sun className="size-5" /> : <Moon className="size-5" />}
    </Button>
  );
}

function MobileHeader({ onMenu }: { onMenu: () => void }) {
  return (
    <header className="sticky top-0 z-40 flex h-16 items-center justify-between border-b border-border bg-background/88 px-4 backdrop-blur lg:hidden">
      <Brand />
      <Button
        variant="ghost"
        size="icon"
        aria-label="打开导航菜单"
        onClick={onMenu}
      >
        <Menu className="size-5" />
      </Button>
    </header>
  );
}

function MobileDrawer({
  path,
  onClose,
  onLogout,
}: {
  path: string;
  onClose: () => void;
  onLogout: () => void;
}) {
  return (
    <>
      <motion.button
        aria-label="关闭导航菜单"
        className="fixed inset-0 z-50 bg-foreground/20 backdrop-blur-[2px] lg:hidden"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        onClick={onClose}
      />
      <motion.aside
        role="dialog"
        aria-label="导航菜单"
        className="fixed inset-y-0 right-0 z-[51] flex w-[min(86vw,340px)] flex-col border-l border-border bg-background p-5 shadow-popover lg:hidden"
        initial={{ x: "100%" }}
        animate={{ x: 0 }}
        exit={{ x: "100%" }}
        transition={{ duration: 0.24, ease: [0.22, 1, 0.36, 1] }}
      >
        <div className="flex items-center justify-between">
          <span className="font-display text-2xl font-semibold">导航</span>
          <Button
            variant="ghost"
            size="icon"
            aria-label="关闭导航菜单"
            onClick={onClose}
          >
            <X className="size-5" />
          </Button>
        </div>
        <nav className="mt-8 grid gap-2">
          {navigation.map((item) => (
            <NavItem
              key={item.href}
              {...item}
              active={
                item.href === "/" ? path === "/" : path.startsWith(item.href)
              }
              onClick={onClose}
            />
          ))}
        </nav>
        <div className="mt-auto flex items-center justify-between border-t border-border pt-5">
          <Button variant="ghost" onClick={onLogout}>
            <LogOut className="size-4" />
            退出
          </Button>
          <ThemeToggle />
        </div>
      </motion.aside>
    </>
  );
}

function MobileNav({ path }: { path: string }) {
  const items = useMemo(() => navigation.slice(0, 4), []);
  return (
    <nav
      aria-label="移动端主导航"
      className="fixed inset-x-3 bottom-3 z-40 grid grid-cols-4 rounded-2xl border border-border bg-card/92 p-1.5 shadow-popover backdrop-blur lg:hidden"
    >
      {items.map((item) => {
        const active =
          item.href === "/" ? path === "/" : path.startsWith(item.href);
        const Icon = item.icon;
        return (
          <AppLink
            key={item.href}
            href={item.href}
            aria-current={active ? "page" : undefined}
            className={cn(
              "flex min-h-14 flex-col items-center justify-center gap-1 rounded-xl text-[10px] font-semibold focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
              active
                ? "bg-foreground text-background"
                : "text-muted-foreground",
            )}
          >
            <Icon className={cn("size-[18px]", active && "text-primary")} />
            {item.label.replace("我的", "")}
          </AppLink>
        );
      })}
    </nav>
  );
}

function ShellLoading() {
  return (
    <div className="grid min-h-screen lg:grid-cols-[248px_1fr]">
      <aside className="hidden border-r border-border p-6 lg:block">
        <Skeleton className="h-11 w-40" />
        <div className="mt-12 grid gap-3">
          <Skeleton className="h-11" />
          <Skeleton className="h-11" />
          <Skeleton className="h-11" />
        </div>
      </aside>
      <main className="p-6 lg:p-12">
        <Skeleton className="h-12 w-64" />
        <Skeleton className="mt-7 h-64 w-full" />
      </main>
    </div>
  );
}
