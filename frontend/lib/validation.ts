import { z } from "zod";

export const loginSchema = z.object({
  username: z.string().trim().min(1, "请输入用户名").max(255, "用户名过长"),
  password: z.string().min(1, "请输入密码"),
});

export const passwordSchema = z.object({
  oldPassword: z.string().min(1, "请输入原密码"),
  newPassword: z.string().min(6, "新密码至少需要 6 位"),
});

export const progressSchema = z.object({
  episode: z.number().int("进度必须是整数").min(0, "进度不能小于 0"),
  time: z.string().regex(/^$|^\d{1,3}:[0-5]\d$/, "时间格式应为 mm:ss"),
});

export const customItemSchema = z.object({
  title: z
    .string()
    .trim()
    .min(1, "请输入条目名称")
    .max(255, "名称不能超过 255 个字符"),
  description: z.string().max(2000, "描述不能超过 2000 个字符"),
  cover: z.union([z.literal(""), z.url("请输入有效的封面 URL")]),
  maxNumber: z.number().int().min(0).optional(),
});
