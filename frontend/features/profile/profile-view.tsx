"use client";

import { useEffect, useMemo, useState } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Copy,
  KeyRound,
  Pencil,
  Plus,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";
import { api } from "@/lib/api/client";
import type { ApiTokenItem } from "@/lib/api/types";
import { useAuth } from "@/lib/auth-context";
import { navigate } from "@/lib/router";
import { formatDate } from "@/lib/utils";
import { passwordSchema } from "@/lib/validation";
import { EmptyState, ErrorState } from "@/components/common/async-state";
import { PageHeader } from "@/components/common/page-header";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";

const profileSchema = z.object({
  nickname: z.string().max(255, "昵称过长"),
  avatar: z.union([z.literal(""), z.url("请输入有效的头像 URL")]),
});
type ProfileForm = z.infer<typeof profileSchema>;
type PasswordForm = z.infer<typeof passwordSchema>;

export function ProfileView() {
  const { user, username, refreshUser, updateLocalUser, logout } = useAuth();
  const queryClient = useQueryClient();
  const profileForm = useForm<ProfileForm>({
    resolver: zodResolver(profileSchema),
    defaultValues: { nickname: "", avatar: "" },
  });
  const passwordForm = useForm<PasswordForm>({
    resolver: zodResolver(passwordSchema),
    defaultValues: { oldPassword: "", newPassword: "" },
  });
  const [tokenDialog, setTokenDialog] = useState<{
    mode: "create" | "edit";
    token?: ApiTokenItem;
  } | null>(null);
  const [tokenName, setTokenName] = useState("");
  const [tokenPermissions, setTokenPermissions] = useState(0);
  const [rawToken, setRawToken] = useState("");
  const [deleteToken, setDeleteToken] = useState<ApiTokenItem | null>(null);

  useEffect(() => {
    if (user)
      profileForm.reset({
        nickname: user.nickname || "",
        avatar: user.avatar || "",
      });
  }, [user, profileForm]);

  const tokenQuery = useQuery({
    queryKey: ["tokens"],
    queryFn: async () => {
      const response = await api.listTokens();
      if (response.status !== 0)
        throw new Error(response.message || "Token 列表加载失败");
      return response.data || [];
    },
  });
  const permissionQuery = useQuery({
    queryKey: ["tokens", "permissions"],
    queryFn: async () => {
      const response = await api.getPermissionLabels();
      if (response.status !== 0 || !response.data)
        throw new Error(response.message || "权限列表加载失败");
      return response.data;
    },
  });
  const cleanupQuery = useQuery({
    queryKey: ["settings", "auto-cleanup"],
    queryFn: async () => {
      const response = await api.getAutoCleanupSetting();
      if (response.status !== 0 || !response.data)
        throw new Error(response.message || "清理设置加载失败");
      return response.data.enabled;
    },
  });

  const saveProfile = useMutation({
    mutationFn: async (values: ProfileForm) => {
      const response = await api.updateUserInfo(
        values.nickname || undefined,
        values.avatar || undefined,
      );
      if (response.status !== 0)
        throw new Error(response.message || "个人信息保存失败");
      return values;
    },
    onSuccess: async (values) => {
      updateLocalUser(values);
      await refreshUser();
      toast.success("个人信息已保存");
    },
    onError: (error) => toast.error(error.message),
  });
  const changePassword = useMutation({
    mutationFn: async (values: PasswordForm) => {
      const response = await api.updatePassword(
        values.oldPassword,
        values.newPassword,
      );
      if (response.status !== 0)
        throw new Error(response.message || "密码修改失败");
    },
    onSuccess: () => {
      toast.success("密码已修改，请重新登录");
      passwordForm.reset();
      window.setTimeout(() => {
        logout();
        navigate("/login");
      }, 900);
    },
    onError: (error) => toast.error(error.message),
  });
  const updateCleanup = useMutation({
    mutationFn: async (enabled: boolean) => {
      const response = await api.updateAutoCleanupSetting(enabled);
      if (response.status !== 0)
        throw new Error(response.message || "设置保存失败");
      return enabled;
    },
    onMutate: async (enabled) => {
      await queryClient.cancelQueries({
        queryKey: ["settings", "auto-cleanup"],
      });
      const previous = queryClient.getQueryData<boolean>([
        "settings",
        "auto-cleanup",
      ]);
      queryClient.setQueryData(["settings", "auto-cleanup"], enabled);
      return { previous };
    },
    onError: (error, _enabled, context) => {
      queryClient.setQueryData(["settings", "auto-cleanup"], context?.previous);
      toast.error(error.message);
    },
    onSuccess: (enabled) =>
      toast.success(enabled ? "已启用自动清理" : "已关闭自动清理"),
  });
  const saveToken = useMutation({
    mutationFn: async () => {
      if (!tokenDialog || !tokenName.trim())
        throw new Error("请输入 Token 名称");
      if (!tokenPermissions) throw new Error("请至少选择一项权限");
      if (tokenDialog.mode === "create") {
        const response = await api.createToken(
          tokenName.trim(),
          tokenPermissions,
        );
        if (response.status !== 0 || !response.data)
          throw new Error(response.message || "Token 创建失败");
        return response.data.raw_token;
      }
      const response = await api.updateToken(tokenDialog.token?.id || 0, {
        name: tokenName.trim(),
        permissions: tokenPermissions,
      });
      if (response.status !== 0)
        throw new Error(response.message || "Token 保存失败");
      return "";
    },
    onSuccess: (token) => {
      void queryClient.invalidateQueries({ queryKey: ["tokens"] });
      if (token) setRawToken(token);
      else {
        setTokenDialog(null);
        toast.success("Token 已更新");
      }
    },
    onError: (error) => toast.error(error.message),
  });
  const toggleToken = useMutation({
    mutationFn: async (token: ApiTokenItem) => {
      const response = await api.updateToken(token.id, {
        is_active: !token.is_active,
      });
      if (response.status !== 0)
        throw new Error(response.message || "Token 状态更新失败");
    },
    onSuccess: () =>
      void queryClient.invalidateQueries({ queryKey: ["tokens"] }),
    onError: (error) => toast.error(error.message),
  });
  const removeToken = useMutation({
    mutationFn: async (token: ApiTokenItem) => {
      const response = await api.deleteToken(token.id);
      if (response.status !== 0)
        throw new Error(response.message || "Token 删除失败");
    },
    onSuccess: () => {
      setDeleteToken(null);
      toast.success("Token 已删除");
      void queryClient.invalidateQueries({ queryKey: ["tokens"] });
    },
    onError: (error) => toast.error(error.message),
  });

  const allValue = permissionQuery.data?.all_value || 255;
  const permissions = useMemo(
    () => permissionQuery.data?.labels || [],
    [permissionQuery.data?.labels],
  );
  function openTokenDialog(mode: "create" | "edit", token?: ApiTokenItem) {
    setTokenDialog({ mode, token });
    setRawToken("");
    setTokenName(token?.name || "");
    setTokenPermissions(token?.permissions || permissions[0]?.value || 0);
  }
  function togglePermission(value: number) {
    setTokenPermissions((current) =>
      value === allValue
        ? (current & allValue) === allValue
          ? 0
          : allValue
        : (current & value) !== 0
          ? current ^ value
          : current | value,
    );
  }
  const permissionText = useMemo(
    () => (value: number) =>
      (value & allValue) === allValue
        ? "全部权限"
        : permissions
            .filter((item) => (value & item.value) !== 0)
            .map((item) => item.label)
            .join("、") || "无权限",
    [allValue, permissions],
  );

  return (
    <>
      <PageHeader
        eyebrow="Account / 账户"
        title="设置"
        description="管理个人资料、安全设置、记录清理与开放 API Token。"
      />
      <div className="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        <Card>
          <CardHeader>
            <CardTitle>个人资料</CardTitle>
            <CardDescription>
              这些信息只在登录后的界面和 API 中显示。
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="mb-6 flex items-center gap-4">
              <Avatar className="size-16">
                <AvatarImage
                  src={user?.avatar || undefined}
                  alt={user?.nickname || username || "用户头像"}
                />
                <AvatarFallback>
                  {(user?.nickname || username || "U")
                    .slice(0, 1)
                    .toUpperCase()}
                </AvatarFallback>
              </Avatar>
              <div className="min-w-0">
                <div className="truncate text-lg font-semibold">
                  {user?.nickname || username}
                </div>
                <div className="mt-1 truncate font-mono text-xs text-muted-foreground">
                  {user?.uuid || "正在载入用户信息"}
                </div>
                <div className="mt-2">
                  <Badge variant={user?.is_admin ? "default" : "outline"}>
                    {user?.is_admin ? "管理员" : "普通用户"}
                  </Badge>
                </div>
              </div>
            </div>
            <form
              className="grid gap-4"
              onSubmit={profileForm.handleSubmit((values) =>
                saveProfile.mutate(values),
              )}
            >
              <FormField
                label="昵称"
                error={profileForm.formState.errors.nickname?.message}
              >
                <Input
                  {...profileForm.register("nickname")}
                  placeholder="设置显示昵称"
                />
              </FormField>
              <FormField
                label="头像 URL"
                error={profileForm.formState.errors.avatar?.message}
              >
                <Input
                  {...profileForm.register("avatar")}
                  type="url"
                  placeholder="https://"
                />
              </FormField>
              <Button type="submit" loading={saveProfile.isPending}>
                保存资料
              </Button>
            </form>
          </CardContent>
        </Card>
        <div className="grid gap-5">
          <Card>
            <CardHeader>
              <CardTitle>修改密码</CardTitle>
              <CardDescription>
                保存后当前 JWT 会退出，需要重新登录。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form
                className="grid gap-4"
                onSubmit={passwordForm.handleSubmit((values) =>
                  changePassword.mutate(values),
                )}
              >
                <FormField
                  label="原密码"
                  error={passwordForm.formState.errors.oldPassword?.message}
                >
                  <Input
                    {...passwordForm.register("oldPassword")}
                    type="password"
                    autoComplete="current-password"
                  />
                </FormField>
                <FormField
                  label="新密码"
                  error={passwordForm.formState.errors.newPassword?.message}
                >
                  <Input
                    {...passwordForm.register("newPassword")}
                    type="password"
                    autoComplete="new-password"
                  />
                </FormField>
                <Button
                  type="submit"
                  variant="outline"
                  loading={changePassword.isPending}
                >
                  <KeyRound className="size-4" />
                  更新密码
                </Button>
              </form>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>记录清理</CardTitle>
              <CardDescription>
                软删除超过 30
                天的追踪记录会在服务器每日清理时永久移除；日志不会被清理。
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between gap-4">
                <div>
                  <Label htmlFor="cleanup-switch">自动清理软删除记录</Label>
                  <p className="mt-1 text-xs text-muted-foreground">
                    可随时关闭，不影响现有数据。
                  </p>
                </div>
                {cleanupQuery.isLoading ? (
                  <Skeleton className="h-6 w-11" />
                ) : (
                  <Switch
                    id="cleanup-switch"
                    checked={cleanupQuery.data || false}
                    disabled={updateCleanup.isPending}
                    onCheckedChange={(checked) => updateCleanup.mutate(checked)}
                  />
                )}
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
      <section className="mt-8">
        <div className="mb-4 flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="font-display text-3xl font-semibold">API Token</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              每个 Token 独立授权，可随时停用或删除。
            </p>
          </div>
          <Button onClick={() => openTokenDialog("create")}>
            <Plus className="size-4" />
            新建 Token
          </Button>
        </div>
        {tokenQuery.isLoading || permissionQuery.isLoading ? (
          <div className="grid gap-3">
            <Skeleton className="h-20" />
            <Skeleton className="h-20" />
          </div>
        ) : tokenQuery.isError ? (
          <ErrorState
            message={tokenQuery.error.message}
            onRetry={() => void tokenQuery.refetch()}
          />
        ) : !tokenQuery.data?.length ? (
          <EmptyState
            title="还没有 API Token"
            description="为脚本、客户端或同步工具创建一个最小权限 Token。"
          />
        ) : (
          <div className="grid gap-3">
            {tokenQuery.data.map((token) => (
              <Card
                key={token.id}
                className="grid gap-4 p-4 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center"
              >
                <div className="min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="font-semibold">{token.name}</span>
                    <Badge variant={token.is_active ? "success" : "outline"}>
                      {token.is_active ? "启用" : "停用"}
                    </Badge>
                  </div>
                  <p className="mt-2 truncate text-sm text-muted-foreground">
                    {permissionText(token.permissions)}
                  </p>
                  <div className="mt-2 font-mono text-[10px] text-muted-foreground">
                    创建 {formatDate(token.created_at)} · 最后使用{" "}
                    {formatDate(token.last_used_at, true)}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <Switch
                    checked={token.is_active}
                    aria-label={`${token.name}${token.is_active ? "停用" : "启用"}`}
                    disabled={toggleToken.isPending}
                    onCheckedChange={() => toggleToken.mutate(token)}
                  />
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={`编辑 ${token.name}`}
                    onClick={() => openTokenDialog("edit", token)}
                  >
                    <Pencil className="size-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-destructive hover:text-destructive"
                    aria-label={`删除 ${token.name}`}
                    onClick={() => setDeleteToken(token)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        )}
      </section>
      <Dialog
        open={Boolean(tokenDialog)}
        onOpenChange={(open) => {
          if (!open && !saveToken.isPending) setTokenDialog(null);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              {tokenDialog?.mode === "create"
                ? "新建 API Token"
                : "编辑 API Token"}
            </DialogTitle>
            <DialogDescription>
              {rawToken
                ? "这是完整 Token 唯一一次显示，请立即保存。"
                : "遵循最小权限原则，只开启实际需要的能力。"}
            </DialogDescription>
          </DialogHeader>
          {rawToken ? (
            <div>
              <div className="rounded-xl border border-warning/30 bg-warning/10 p-4 text-sm text-warning-foreground">
                关闭窗口后无法再次查看完整 Token。
              </div>
              <div className="mt-4 flex gap-2">
                <Input
                  readOnly
                  className="font-mono text-xs"
                  value={rawToken}
                />
                <Button
                  variant="outline"
                  size="icon"
                  aria-label="复制 Token"
                  onClick={() => copyText(rawToken)}
                >
                  <Copy className="size-4" />
                </Button>
              </div>
              <DialogFooter className="mt-5">
                <Button onClick={() => setTokenDialog(null)}>我已保存</Button>
              </DialogFooter>
            </div>
          ) : (
            <>
              <div className="grid gap-4">
                <FormField label="Token 名称">
                  <Input
                    value={tokenName}
                    onChange={(event) => setTokenName(event.target.value)}
                    placeholder="例如 Animeko 同步"
                  />
                </FormField>
                <div>
                  <Label>权限</Label>
                  <div className="mt-3 grid gap-2">
                    <PermissionRow
                      label="全部权限"
                      description="授予当前所有开放 API 权限"
                      checked={(tokenPermissions & allValue) === allValue}
                      onChange={() => togglePermission(allValue)}
                    />
                    {permissions.map((permission) => (
                      <PermissionRow
                        key={permission.value}
                        label={permission.label}
                        description={permission.description}
                        checked={(tokenPermissions & permission.value) !== 0}
                        onChange={() => togglePermission(permission.value)}
                      />
                    ))}
                  </div>
                </div>
              </div>
              <DialogFooter>
                <Button variant="outline" onClick={() => setTokenDialog(null)}>
                  取消
                </Button>
                <Button
                  loading={saveToken.isPending}
                  onClick={() => saveToken.mutate()}
                >
                  <ShieldCheck className="size-4" />
                  {tokenDialog?.mode === "create" ? "创建 Token" : "保存权限"}
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
      <AlertDialog
        open={Boolean(deleteToken)}
        onOpenChange={(open) => !open && setDeleteToken(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>删除 API Token？</AlertDialogTitle>
            <AlertDialogDescription>
              依赖「{deleteToken?.name}
              」的客户端会立即失去访问权限。此操作不可恢复。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={(event) => {
                event.preventDefault();
                if (deleteToken) removeToken.mutate(deleteToken);
              }}
            >
              {removeToken.isPending ? "删除中…" : "删除 Token"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

function FormField({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {children}
      {error ? (
        <p className="text-xs font-medium text-destructive">{error}</p>
      ) : null}
    </div>
  );
}
function PermissionRow({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description: string;
  checked: boolean;
  onChange: () => void;
}) {
  return (
    <label className="flex min-h-14 cursor-pointer items-start gap-3 rounded-xl border border-border p-3 transition hover:bg-muted/50">
      <Checkbox checked={checked} onCheckedChange={onChange} />
      <span>
        <span className="block text-sm font-semibold">{label}</span>
        {description ? (
          <span className="mt-1 block text-xs leading-relaxed text-muted-foreground">
            {description}
          </span>
        ) : null}
      </span>
    </label>
  );
}
async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    toast.success("已复制到剪贴板");
  } catch {
    toast.error("复制失败，请手动选择 Token");
  }
}
